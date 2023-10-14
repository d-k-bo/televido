use std::cell::RefCell;

use adw::{gio, glib, gtk, prelude::*, subclass::prelude::*};

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

        #[property(get, set)]
        video_player_name: RefCell<String>,
        #[property(get, set)]
        video_player_id: RefCell<String>,
    }

    #[gtk::template_callbacks]
    impl TvPreferencesWindow {
        #[template_callback]
        async fn select_video_player(&self, #[rest] _: &[glib::Value]) {
            let slf = self.obj();

            if let Some(program) =
                ProgramSelector::select_program(ExternalProgramType::Player).await
            {
                slf.set_video_player_name(program.name);
                slf.set_video_player_id(program.id);
            }
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

            let slf = self.obj();
            let settings = TvSettings::get();

            settings
                .bind_video_player_name(&*slf, "video-player-name")
                .flags(gio::SettingsBindFlags::DEFAULT)
                .build();
            settings
                .bind_video_player_id(&*slf, "video-player-id")
                .flags(gio::SettingsBindFlags::DEFAULT)
                .build();
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
