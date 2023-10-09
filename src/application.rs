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

use std::cell::OnceCell;

use adw::{gio, glib, prelude::*, subclass::prelude::*};

use crate::config::{APP_NAME, AUTHOR};
use crate::{
    config::{APP_ID, VERSION},
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

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([quit_action, about_action]);
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
