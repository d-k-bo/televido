// Copyright 2023 David Cabot
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    cell::OnceCell,
    rc::Rc,
    sync::{Arc, OnceLock},
};

use adw::{gio, glib, prelude::*, subclass::prelude::*};
use eyre::WrapErr;
use gettextrs::gettext;
use smart_default::SmartDefault;

use crate::{
    config::{APP_ID, APP_NAME, AUTHOR, ISSUE_URL, PROJECT_URL, VERSION},
    launcher::{ExternalProgram, ExternalProgramType, ProgramSelector},
    preferences::TvPreferencesDialog,
    settings::TvSettings,
    utils::{show_error, spawn, spawn_clone, tokio, AsyncResource},
    window::TvWindow,
    zapp::Zapp,
};

mod imp {
    use super::*;

    #[derive(Debug, SmartDefault)]
    pub struct TvApplication {
        #[default(Arc::new(Zapp::new().expect("failed to initialize Zapp client")))]
        pub(super) zapp: Arc<Zapp>,
        pub(super) live_channels: AsyncResource<Rc<crate::zapp::ChannelInfoList>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TvApplication {
        const NAME: &'static str = "TvApplication";
        type Type = super::TvApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for TvApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
        }
    }

    impl ApplicationImpl for TvApplication {
        fn activate(&self) {
            self.obj().window().present();
        }
    }

    impl GtkApplicationImpl for TvApplication {}
    impl AdwApplicationImpl for TvApplication {}
}

glib::wrapper! {
    pub struct TvApplication(ObjectSubclass<imp::TvApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

thread_local! {
    static APPLICATION: OnceCell<TvApplication> = const { OnceCell::new() };
}

impl TvApplication {
    pub fn new() -> Self {
        let slf: Self = glib::Object::builder()
            .property("application-id", APP_ID)
            .property("flags", gio::ApplicationFlags::FLAGS_NONE)
            .property("resource-base-path", "/de/k_bo/televido")
            .build();

        APPLICATION.with(|app| {
            app.set(slf.clone())
                .expect("TvApplication may only be created once")
        });

        let live_channels = slf.live_channels();
        live_channels.set_load_fn({
            let zapp = slf.zapp();
            move || {
                let zapp = zapp.clone();
                Box::pin(async move {
                    match tokio(async move {
                        zapp.channel_info_list()
                            .await
                            .wrap_err("Failed to load channel info list")
                    })
                    .await
                    {
                        Ok(channels) => Rc::new(channels),
                        Err(e) => {
                            show_error(e.wrap_err(gettext("Failed to load livestream channels")));
                            Default::default()
                        }
                    }
                })
            }
        });
        spawn(async move { live_channels.load() });

        slf
    }
    pub fn get() -> Self {
        APPLICATION.with(|app| {
            app.get()
                .expect("TvApplication::get() may only be called from the main thread")
                .clone()
        })
    }

    pub async fn dbus() -> zbus::Connection {
        static DBUS: OnceLock<zbus::Connection> = OnceLock::new();
        match DBUS.get() {
            Some(dbus) => dbus.clone(),
            None => {
                let conn = tokio(async { zbus::Connection::session().await })
                    .await
                    .expect("Failed to acquire a connection to the session D-Bus.");
                DBUS.set(conn.clone())
                    .expect("D-Bus connection has already been acquired.");
                conn
            }
        }
    }

    pub fn window(&self) -> TvWindow {
        self.active_window().and_downcast().unwrap_or_else(|| {
            let win = TvWindow::new(self);
            win.present();
            win
        })
    }

    pub fn zapp(&self) -> Arc<Zapp> {
        self.imp().zapp.clone()
    }

    pub fn live_channels(&self) -> AsyncResource<Rc<crate::zapp::ChannelInfoList>> {
        self.imp().live_channels.clone()
    }

    pub async fn play(&self, uri: String) {
        let settings = TvSettings::get();
        let player_name = settings.video_player_name();
        let player_id = settings.video_player_id();

        let player = if player_id.is_empty() {
            None
        } else {
            match ExternalProgram::find(player_name, player_id.clone()).await {
                Ok(player) => player,
                Err(e) => {
                    show_error(e);
                    None
                }
            }
        };

        let player = match player {
            Some(player) => player,
            None => {
                match ProgramSelector::select_program(ExternalProgramType::Player, player_id).await
                {
                    Some(player) => {
                        settings.set_video_player_name(&player.name);
                        settings.set_video_player_id(&player.id);

                        player
                    }
                    None => return,
                }
            }
        };

        match player.open(uri).await {
            Ok(()) => (),
            Err(e) => show_error(e.wrap_err(gettext("Failed to play video stream"))),
        }
    }

    pub async fn download(&self, uri: String) {
        let settings = TvSettings::get();
        let downloader_name = settings.video_downloader_name();
        let downloader_id = settings.video_downloader_id();

        let downloader = if downloader_id.is_empty() {
            None
        } else {
            match ExternalProgram::find(downloader_name, downloader_id.clone()).await {
                Ok(downloader) => downloader,
                Err(e) => {
                    show_error(e);
                    None
                }
            }
        };

        let downloader = match downloader {
            Some(downloader) => downloader,
            None => {
                match ProgramSelector::select_program(
                    ExternalProgramType::Downloader,
                    downloader_id,
                )
                .await
                {
                    Some(downloader) => {
                        settings.set_video_downloader_name(&downloader.name);
                        settings.set_video_downloader_id(&downloader.id);

                        downloader
                    }
                    None => return,
                }
            }
        };

        match downloader.open(uri).await {
            Ok(()) => (),
            Err(e) => show_error(e.wrap_err(gettext("Failed to launch video downloader"))),
        }
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        let preferences_action = gio::ActionEntry::builder("preferences")
            .activate(move |app: &Self, _, _| TvPreferencesDialog::new().present(&app.window()))
            .build();
        let play_action = gio::ActionEntry::builder("play")
            .parameter_type(Some(glib::VariantTy::STRING))
            .activate(move |app: &Self, _, variant| {
                if let Some(stream_url) = variant.and_then(|v| v.get()) {
                    spawn_clone!(app => app.play(stream_url))
                }
            })
            .build();
        let download_action = gio::ActionEntry::builder("download")
            .parameter_type(Some(glib::VariantTy::STRING))
            .activate(move |app: &Self, _, variant| {
                if let Some(stream_url) = variant.and_then(|v| v.get()) {
                    spawn_clone!(app => app.download(stream_url))
                }
            })
            .build();

        self.add_action_entries([
            quit_action,
            about_action,
            preferences_action,
            play_action,
            download_action,
        ]);

        self.set_accels_for_action("app.quit", &["<primary>q"]);
        self.set_accels_for_action("window.close", &["<primary>w"]);
        self.set_accels_for_action("window.reload", &["F5", "Reload"]);
        self.set_accels_for_action("window.show-live", &["<primary>l"]);
        self.set_accels_for_action("window.show-mediathek", &["<primary>m"]);
    }

    fn show_about(&self) {
        let about = adw::AboutDialog::builder()
            .application_name(APP_NAME)
            .application_icon(APP_ID)
            .developer_name(AUTHOR)
            .version(VERSION)
            .website(PROJECT_URL)
            .issue_url(ISSUE_URL)
            .license_type(gtk::License::Gpl30)
            .build();

        about.present(&self.window());
    }
}
