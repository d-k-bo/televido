// Copyright 2023 David Cabot
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::OnceCell;

use adw::{glib, prelude::*, subclass::prelude::*};
use mediathekviewweb::models::Item;

use crate::{utils::{format_duration, format_timestamp_full}, settings::VideoQuality};

mod imp {
    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::ShowObject)]
    pub struct ShowObject {
        #[property(
            name = "channel",
            type = String,
            get,
            member = channel,
        )]
        #[property(
            name = "topic",
            type = String,
            get,
            member = topic,
        )]
        #[property(
            name = "title",
            type = String,
            get,
            member = title,
        )]
        #[property(
            name = "description",
            type = Option<String>,
            get,
            member = description,
        )]
        #[property(
            name = "date",
            type = Option<glib::GString>,
            get = |show: &ShowObject| format_timestamp_full(show.inner.get().unwrap().timestamp),
        )]
        #[property(
            name = "duration",
            type = String,
            get = |show: &ShowObject| format_duration(&show.inner.get().unwrap().duration), 
        )]
        #[property(
            name = "video-url-high",
            type = Option<String>,
            get,
            member = url_video_hd,
        )]
        #[property(
            name = "video-url-medium",
            type = Option<String>,
            get,
            member = url_video,
        )]
        #[property(
            name = "video-url-low",
            type = Option<String>,
            get,
            member = url_video_low
        )]
        #[property(
            name = "website-url",
            type = Option<String>,
            get,
            member = url_website
        )]
        pub(super) inner: OnceCell<Item>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ShowObject {
        const NAME: &'static str = "ShowObject";
        type Type = super::ShowObject;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ShowObject {}
}

glib::wrapper! {
    pub struct ShowObject(ObjectSubclass<imp::ShowObject>);
}

impl Default for ShowObject {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl ShowObject {
    pub fn new(item: Item) -> Self {
        let slf: Self = glib::Object::new();
        slf.imp()
            .inner
            .set(item)
            .expect("ShowObject has already been initialized.");
        slf
    }
    pub fn video_url(&self, quality: VideoQuality) -> Option<String> {
        match quality {
            VideoQuality::High => self.video_url_high(),
            VideoQuality::Medium => self.video_url_medium(),
            VideoQuality::Low => self.video_url_low(),
        }
    }
}
