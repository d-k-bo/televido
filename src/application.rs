// Copyright 2023 David Cabot
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{cell::OnceCell, sync::OnceLock};

use adw::{gio, glib, prelude::*, subclass::prelude::*};
use tracing::error;

use crate::{
    config::{APP_ID, APP_NAME, AUTHOR, ISSUE_URL, PROJECT_URL, VERSION},
    launcher::{ExternalProgramType, ProgramSelector},
    preferences::TvPreferencesWindow,
    settings::TvSettings,
    utils::{spawn_clone, tokio},
    window::TvWindow,
};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct TvApplication {}

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
            let application = self.obj();
            application
                .active_window()
                .unwrap_or_else(|| TvWindow::new(&application).upcast())
                .present();
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

impl TvApplication {
    pub fn get() -> Self {
        thread_local! {
            static APPLICATION: OnceCell<TvApplication> = OnceCell::new();
        }
        APPLICATION.with(|app| {
            app.get_or_init(|| {
                glib::Object::builder()
                    .property("application-id", APP_ID)
                    .property("flags", gio::ApplicationFlags::FLAGS_NONE)
                    .property("resource-base-path", "/de/k_bo/televido")
                    .build()
            })
            .clone()
        })
    }
    pub fn new() -> Self {
        Self::get()
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

    pub async fn play(&self, uri: String) {
        let settings = TvSettings::get();
        let player_id = settings.video_player_id();
        let player = if player_id.is_empty() {
            let Some(program) = ProgramSelector::select_program(ExternalProgramType::Player).await
            else {
                return;
            };

            settings.set_video_player_name(program.name);
            settings.set_video_player_id(program.id);

            program
        } else {
            let Some(program) = ExternalProgramType::Player.find(&player_id) else {
                return;
            };
            program
        };

        match player.play(uri).await {
            Ok(()) => (),
            Err(e) => error!("{}", e.wrap_err("failed to play video stream")),
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
            .activate(move |app: &Self, _, _| {
                TvPreferencesWindow::new(app.active_window().as_ref()).present()
            })
            .build();
        let play_action = gio::ActionEntry::builder("play")
            .parameter_type(Some(glib::VariantTy::STRING))
            .activate(move |app: &Self, _, variant| {
                if let Some(stream_url) = variant.and_then(|v| v.get()) {
                    spawn_clone!(app => app.play(stream_url))
                }
            })
            .build();
        self.add_action_entries([quit_action, about_action, preferences_action, play_action]);

        self.set_accels_for_action("app.quit", &["<primary>q"]);
        self.set_accels_for_action("window.close", &["<primary>w"]);
        self.set_accels_for_action("window.reload", &["F5", "Reload"]);
        self.set_accels_for_action("window.show-live", &["<primary>l"]);
        self.set_accels_for_action("window.show-mediathek", &["<primary>m"]);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutWindow::builder()
            .transient_for(&window)
            .application_name(APP_NAME)
            .application_icon(APP_ID)
            .developer_name(AUTHOR)
            .version(VERSION)
            .website(PROJECT_URL)
            .issue_url(ISSUE_URL)
            .license_type(gtk::License::Gpl30)
            .build();

        about.present();
    }
}
