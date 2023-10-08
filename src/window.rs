// Copyright 2023 David Cabot
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::{gio, glib, gtk, prelude::*, subclass::prelude::*};

use crate::{
    application::MdkApplication, config::PROFILE, live::MdkLiveView, mediathek::MdkMediathekView,
    settings::MdkSettings,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate /* , glib::Properties */)]
    #[template(file = "src/window.blp")]
    // #[properties(wrapper_type = super::MdkWindow)]
    pub struct MdkWindow {
        #[template_child]
        stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        live_view: TemplateChild<MdkLiveView>,
        #[template_child]
        mediathek_view: TemplateChild<MdkMediathekView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MdkWindow {
        const NAME: &'static str = "MdkWindow";
        type Type = super::MdkWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // #[glib::derived_properties]
    impl ObjectImpl for MdkWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let slf = self.obj();
            let settings = MdkSettings::get();

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
    impl WidgetImpl for MdkWindow {}
    impl WindowImpl for MdkWindow {}
    impl ApplicationWindowImpl for MdkWindow {}
    impl AdwApplicationWindowImpl for MdkWindow {}
}

glib::wrapper! {
    pub struct MdkWindow(ObjectSubclass<imp::MdkWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl MdkWindow {
    pub fn new(application: &MdkApplication) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .build();

        if PROFILE == "Devel" {
            win.add_css_class("devel");
        }

        win
    }
}
