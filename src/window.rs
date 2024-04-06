// SPDX-FileCopyrightText: David Cabot <d-k-bo@mailbox.org>
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::{gio, glib, gtk, prelude::*, subclass::prelude::*};

use crate::{
    application::TvApplication, config::PROFILE, live::TvLiveView, mediathek::TvMediathekView,
    settings::TvSettings,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(file = "src/window.blp")]
    pub struct TvWindow {
        #[template_child]
        pub(super) toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        live_view: TemplateChild<TvLiveView>,
        #[template_child]
        mediathek_view: TemplateChild<TvMediathekView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TvWindow {
        const NAME: &'static str = "TvWindow";
        type Type = super::TvWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("window.reload", None, |slf, _, _| {
                let slf = slf.imp();
                match slf.stack.visible_child_name().as_deref() {
                    Some("live") => slf.live_view.reload(),
                    Some("mediathek") => slf.mediathek_view.reload(),
                    _ => (),
                }
            });
            klass.install_action("window.show-live", None, |slf, _, _| {
                slf.imp().stack.set_visible_child_name("live")
            });
            klass.install_action("window.show-mediathek", None, |slf, _, _| {
                slf.imp().stack.set_visible_child_name("mediathek")
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TvWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let slf = self.obj();
            let settings = TvSettings::get();

            settings
                .bind_width(&*slf, "default-width")
                .flags(gio::SettingsBindFlags::DEFAULT)
                .build();
            settings
                .bind_height(&*slf, "default-height")
                .flags(gio::SettingsBindFlags::DEFAULT)
                .build();
            settings
                .bind_is_maximized(&*slf, "maximized")
                .flags(gio::SettingsBindFlags::DEFAULT)
                .build();
            settings
                .bind_is_fullscreen(&*slf, "fullscreened")
                .flags(gio::SettingsBindFlags::DEFAULT)
                .build();
            settings
                .bind_visible_view(&*self.stack, "visible-child-name")
                .flags(gio::SettingsBindFlags::DEFAULT)
                .build();
        }
    }
    impl WidgetImpl for TvWindow {}
    impl WindowImpl for TvWindow {}
    impl ApplicationWindowImpl for TvWindow {}
    impl AdwApplicationWindowImpl for TvWindow {}
}

glib::wrapper! {
    pub struct TvWindow(ObjectSubclass<imp::TvWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Native;
}

impl TvWindow {
    pub fn new(application: &TvApplication) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .build();

        if PROFILE == "Devel" {
            win.add_css_class("devel");
        }

        win
    }
    pub fn add_toast(&self, toast: adw::Toast) {
        self.imp().toast_overlay.add_toast(toast)
    }
}
