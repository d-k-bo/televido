// Copyright 2023 David Cabot
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;

use adw::{glib, gtk, prelude::*, subclass::prelude::*};

use crate::{
    launcher::{ExternalProgramType, ProgramSelector},
    settings::TvSettings,
};

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/preferences.blp")]
    #[properties(wrapper_type = super::TvPreferencesWindow)]
    pub struct TvPreferencesWindow {
        #[template_child]
        video_player_row: TemplateChild<adw::ActionRow>,

        #[property(get)]
        video_player_display_name: RefCell<String>,

        settings: TvSettings,
    }

    #[gtk::template_callbacks]
    impl TvPreferencesWindow {
        #[template_callback]
        async fn select_video_player(&self, #[rest] _: &[glib::Value]) {
            if let Some(program) =
                ProgramSelector::select_program(ExternalProgramType::Player).await
            {
                self.settings.set_video_player_name(program.name);
                self.settings.set_video_player_id(program.id);
            }
        }
    }

    impl TvPreferencesWindow {
        fn update_video_player_display_name(&self) {
            *self.video_player_display_name.borrow_mut() = format!(
                "{} ({})",
                self.settings.video_player_name(),
                self.settings.video_player_id()
            );
            self.obj().notify_video_player_display_name();
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TvPreferencesWindow {
        const NAME: &'static str = "TvPreferencesWindow";
        type Type = super::TvPreferencesWindow;
        type ParentType = adw::PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for TvPreferencesWindow {
        fn constructed(&self) {
            self.parent_constructed();

            self.update_video_player_display_name();
            self.settings.connect_video_player_name_changed(
                glib::clone!(@weak self as slf => move |_| slf.update_video_player_display_name()),
            );
            self.settings.connect_video_player_id_changed(
                glib::clone!(@weak self as slf => move |_| slf.update_video_player_display_name()),
            );
        }
    }
    impl WidgetImpl for TvPreferencesWindow {}
    impl WindowImpl for TvPreferencesWindow {}
    impl AdwWindowImpl for TvPreferencesWindow {}
    impl PreferencesWindowImpl for TvPreferencesWindow {}
}

glib::wrapper! {
    pub struct TvPreferencesWindow(ObjectSubclass<imp::TvPreferencesWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow;
}

impl TvPreferencesWindow {
    pub fn new(parent: Option<&impl IsA<gtk::Window>>) -> Self {
        match parent {
            Some(parent) => glib::Object::builder()
                .property("modal", true)
                .property("transient-for", parent)
                .build(),
            None => glib::Object::new(),
        }
    }
}
