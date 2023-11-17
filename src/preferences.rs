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
        #[template_child]
        video_downloader_row: TemplateChild<adw::ActionRow>,

        #[property(get)]
        video_player_display_name: RefCell<String>,
        #[property(get)]
        video_downloader_display_name: RefCell<String>,

        settings: TvSettings,
    }

    #[gtk::template_callbacks]
    impl TvPreferencesWindow {
        #[template_callback]
        async fn select_video_player(&self, #[rest] _: &[glib::Value]) {
            if let Some(player) = ProgramSelector::select_program(
                ExternalProgramType::Player,
                self.settings.video_player_id(),
            )
            .await
            {
                self.settings.set_video_player_name(&player.name);
                self.settings.set_video_player_id(&player.id);
            }
        }
        #[template_callback]
        async fn select_video_downloader(&self, #[rest] _: &[glib::Value]) {
            if let Some(downloader) = ProgramSelector::select_program(
                ExternalProgramType::Downloader,
                self.settings.video_downloader_id(),
            )
            .await
            {
                self.settings.set_video_downloader_name(&downloader.name);
                self.settings.set_video_downloader_id(&downloader.id);
            }
        }
    }

    impl TvPreferencesWindow {
        fn update_video_player_display_name(&self) {
            let name = self.settings.video_player_name();
            let id = self.settings.video_player_id();

            *self.video_player_display_name.borrow_mut() = if name.is_empty() {
                id
            } else {
                format!("{name} ({id})",)
            };

            self.obj().notify_video_player_display_name();
        }
        fn update_video_downloader_display_name(&self) {
            let name = self.settings.video_downloader_name();
            let id = self.settings.video_downloader_id();

            *self.video_downloader_display_name.borrow_mut() = if name.is_empty() {
                id
            } else {
                format!("{name} ({id})",)
            };

            self.obj().notify_video_downloader_display_name();
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

            self.update_video_downloader_display_name();
            self.settings.connect_video_downloader_name_changed(
                glib::clone!(@weak self as slf => move |_| slf.update_video_downloader_display_name()),
            );
            self.settings.connect_video_downloader_id_changed(
                glib::clone!(@weak self as slf => move |_| slf.update_video_downloader_display_name()),
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
