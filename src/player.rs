// SPDX-FileCopyrightText: David Cabot <d-k-bo@mailbox.org>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;

use adw::{glib, gtk, prelude::*, subclass::prelude::*};

use crate::{
    application::TvApplication,
    config::{APP_ID, APP_NAME},
    settings::TvPlayerSettings,
};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/player.blp")]
    #[properties(wrapper_type=super::TvPlayer)]
    pub struct TvPlayer {
        #[template_child]
        pub(super) video: TemplateChild<clapper_gtk::Video>,

        #[property(get, set)]
        title: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TvPlayer {
        const NAME: &'static str = "TvPlayer";
        type Type = super::TvPlayer;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for TvPlayer {
        fn constructed(&self) {
            self.parent_constructed();

            self.video
                .player()
                .expect("should not be nullable")
                .add_feature(&clapper::Mpris::new(
                    &format!("org.mpris.MediaPlayer2.{APP_ID}"),
                    APP_NAME,
                    Some(APP_ID),
                ));

            let slf = self.obj();

            let taginject = gst::ElementFactory::make("taginject")
                .property("scope", gst::TagScope::Global)
                .build()
                .expect("failed to create `taginject` element");
            self.video
                .player()
                .expect("should not be nullable")
                .set_video_filter(Some(&taginject));
            slf.connect_title_notify(glib::clone!(
                #[weak]
                taginject,
                move |slf| {
                    let mut tags = gst::TagList::new();
                    tags.make_mut()
                        .add::<gst::tags::Title>(&&*slf.title(), gst::TagMergeMode::Replace);
                    taginject.set_property(
                        "tags",
                        tags.to_string()
                            .strip_prefix("taglist, ")
                            .expect("serialized GstTagList should start with `taglist, `"),
                    )
                }
            ));

            self.video.connect_toggle_fullscreen(glib::clone!(
                #[weak]
                slf,
                move |_| slf.set_fullscreened(!slf.is_fullscreen())
            ));

            let settings = TvPlayerSettings::get();

            settings.bind_width(&*slf, "default-width").build();
            settings.bind_height(&*slf, "default-height").build();
            settings.bind_is_maximized(&*slf, "maximized").build();
            settings.bind_is_fullscreen(&*slf, "fullscreened").build();
        }
    }
    impl WidgetImpl for TvPlayer {}
    impl WindowImpl for TvPlayer {}
    impl AdwWindowImpl for TvPlayer {}
}

glib::wrapper! {
    pub struct TvPlayer(ObjectSubclass<imp::TvPlayer>)
        @extends gtk::Widget, gtk::Window, adw::Window;
}

impl TvPlayer {
    pub fn new(application: &TvApplication) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }
    pub fn play(&self, title: &str, uri: &str, subtitle_uri: Option<&str>) {
        let player = self.imp().video.player().expect("should not be nullable");
        let queue = player.queue().expect("should not be nullable");

        let item = clapper::MediaItem::new(uri);

        if let Some(subtitle_uri) = subtitle_uri {
            item.set_suburi(subtitle_uri);
        }

        queue.add_item(&item);
        queue.select_item(Some(&item));
        player.play();
        self.set_title(title);
    }
}
