use std::cell::{Cell, RefCell};

use adw::{gio, glib, gtk, prelude::*, subclass::prelude::*};
use tracing::error;

use crate::{
    settings::{MdkSettings, VideoQuality},
    utils::{load_channel_icon, spawn},
};

use super::{channels::Channel, shows::ShowObject};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/mediathek/card.blp")]
    #[properties(wrapper_type=super::MdkMediathekCard)]
    pub struct MdkMediathekCard {
        #[template_child]
        icon: TemplateChild<gtk::Image>,
        #[template_child]
        revealer: TemplateChild<gtk::Revealer>,

        #[property(get, construct_only)]
        show: RefCell<Option<ShowObject>>,

        #[property(get, set)]
        expanded: Cell<bool>,
    }
    impl MdkMediathekCard {
        fn set_icon(&self) {
            let icon_name = self
                .obj()
                .show()
                .and_then(|c| c.channel().parse::<Channel>().ok())
                .map(|c| c.icon_name());

            load_channel_icon(&self.icon, icon_name);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MdkMediathekCard {
        const NAME: &'static str = "MdkMediathekCard";
        type Type = super::MdkMediathekCard;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MdkMediathekCard {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().setup_actions();
            self.obj().connect_show_notify(|slf| slf.imp().set_icon());

            self.revealer.connect_child_revealed_notify(|revealer| {
                revealer.set_visible(revealer.is_child_revealed())
            });
        }
    }
    impl WidgetImpl for MdkMediathekCard {}
    impl ListBoxRowImpl for MdkMediathekCard {}
}

glib::wrapper! {
    pub struct MdkMediathekCard(ObjectSubclass<imp::MdkMediathekCard>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl MdkMediathekCard {
    fn play(&self, quality: VideoQuality) {
        self.activate_action(
            "app.play",
            Some(
                &self
                    .show()
                    .and_then(|show| show.video_url(quality))
                    .expect("action must only be enabled if url is not None")
                    .to_variant(),
            ),
        )
        .unwrap()
    }
    fn setup_actions(&self) {
        let actions = gio::SimpleActionGroup::new();

        let play_default = gio::SimpleAction::new("play-default", None);
        play_default.connect_activate(
            glib::clone!(@weak self as slf => move |_,_| slf.play(VideoQuality::default_playback())),
        );
        self.connect_show_notify(glib::clone!(@weak play_default => move |slf| {
            play_default.set_enabled(
                slf.show()
                    .and_then(|show| show.video_url(VideoQuality::default_playback()))
                    .is_some()
                );
        }));
        MdkSettings::get().connect_default_playback_quality_changed(
            glib::clone!(@weak self as slf, @weak play_default => move |_| {
                play_default.set_enabled(
                    slf.show()
                        .and_then(|show| show.video_url(VideoQuality::default_playback()))
                        .is_some()
                    );
            }),
        );
        actions.add_action(&play_default);

        let play_high = gio::SimpleAction::new("play-high", None);
        play_high.connect_activate(
            glib::clone!(@weak self as slf => move |_,_| slf.play(VideoQuality::High)),
        );
        self.connect_show_notify(glib::clone!(@weak play_high => move |slf| {
            play_high.set_enabled(
                slf.show()
                    .and_then(|show| show.video_url(VideoQuality::High))
                    .is_some()
                );
        }));
        actions.add_action(&play_high);

        let play_medium = gio::SimpleAction::new("play-medium", None);
        play_medium.connect_activate(
            glib::clone!(@weak self as slf => move |_,_| slf.play(VideoQuality::Medium)),
        );
        self.connect_show_notify(glib::clone!(@weak play_medium => move |slf| {
            play_medium.set_enabled(
                slf.show()
                    .and_then(|show| show.video_url(VideoQuality::Medium))
                    .is_some()
                );
        }));
        actions.add_action(&play_medium);

        let play_low = gio::SimpleAction::new("play-low", None);
        play_low.connect_activate(
            glib::clone!(@weak self as slf => move |_,_| slf.play(VideoQuality::Low)),
        );
        self.connect_show_notify(glib::clone!(@weak play_low => move |slf| {
            play_low.set_enabled(
                slf.show()
                    .and_then(|show| show.video_url(VideoQuality::Low))
                    .is_some()
                );
        }));
        actions.add_action(&play_low);

        let open_website = gio::SimpleAction::new("open-website", None);
        open_website.connect_activate(glib::clone!(@weak self as slf => move |_,_| spawn(async move {
           let url = slf
               .show()
               .and_then(|show| show.website_url())
               .expect("action must only be enabled if url is not None");
            if let Err(e) = gtk::UriLauncher::new(&url).launch_future(slf.root().and_downcast_ref::<adw::Window>()).await {
                error!("{e}");
            }
        })));
        self.connect_show_notify(glib::clone!(@weak open_website => move |slf| {
            open_website.set_enabled(
                slf.show()
                    .and_then(|show| show.website_url())
                    .is_some()
                );
        }));
        actions.add_action(&open_website);

        self.insert_action_group("card", Some(&actions));
    }
}
