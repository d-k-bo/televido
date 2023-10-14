/* application.rs
 *
 * Copyright 2023 David Cabot
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::{cell::OnceCell, sync::OnceLock};

use adw::{gio, glib, prelude::*, subclass::prelude::*};
use tracing::error;

use crate::{
    config::{APP_ID, APP_NAME, AUTHOR, VERSION},
    launcher::{ExternalProgramType, ProgramSelector},
    preferences::MdkPreferencesWindow,
    settings::MdkSettings,
    utils::{spawn_clone, tokio},
    window::MdkWindow,
};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct MdkApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for MdkApplication {
        const NAME: &'static str = "MdkApplication";
        type Type = super::MdkApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for MdkApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
        }
    }

    impl ApplicationImpl for MdkApplication {
        fn activate(&self) {
            let application = self.obj();
            application
                .active_window()
                .unwrap_or_else(|| MdkWindow::new(&application).upcast())
                .present();
        }
    }

    impl GtkApplicationImpl for MdkApplication {}
    impl AdwApplicationImpl for MdkApplication {}
}

glib::wrapper! {
    pub struct MdkApplication(ObjectSubclass<imp::MdkApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl MdkApplication {
    pub fn get() -> Self {
        thread_local! {
            static APPLICATION: OnceCell<MdkApplication> = OnceCell::new();
        }
        APPLICATION.with(|app| {
            app.get_or_init(|| {
                glib::Object::builder()
                    .property("application-id", APP_ID)
                    .property("flags", gio::ApplicationFlags::FLAGS_NONE)
                    .property("resource-base-path", "/de/k_bo/mediathek")
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
        let settings = MdkSettings::get();
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
                MdkPreferencesWindow::new(app.active_window().as_ref()).present()
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
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutWindow::builder()
            .transient_for(&window)
            .application_name(APP_NAME)
            .application_icon(APP_ID)
            .developer_name(AUTHOR)
            .version(VERSION)
            .build();

        about.present();
    }
}
