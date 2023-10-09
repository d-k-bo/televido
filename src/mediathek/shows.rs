use std::cell::OnceCell;

use adw::{glib, prelude::*, subclass::prelude::*};
use mediathekviewweb::models::Item;
use once_cell::sync::Lazy;

use crate::utils::{format_duration, format_timestamp_full};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ShowObject(pub(super) OnceCell<Item>);

    #[glib::object_subclass]
    impl ObjectSubclass for ShowObject {
        const NAME: &'static str = "ShowObject";
        type Type = super::ShowObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for ShowObject {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::builder("channel")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("topic").read_only().build(),
                    glib::ParamSpecString::builder("title").read_only().build(),
                    glib::ParamSpecString::builder("description")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("date").read_only().build(),
                    glib::ParamSpecString::builder("duration")
                        .read_only()
                        .build(),
                ]
            });
            &PROPERTIES
        }
        fn set_property(&self, _id: usize, _value: &glib::Value, _pspec: &glib::ParamSpec) {}
        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let Some(item) = self.0.get() else {
                return None::<String>.to_value();
            };
            match pspec.name() {
                "channel" => item.channel.to_value(),
                "topic" => item.topic.to_value(),
                "title" => item.title.to_value(),
                "description" => item.description.to_value(),
                "date" => format_timestamp_full(item.timestamp).to_value(),
                "duration" => format_duration(&item.duration).to_value(),
                _ => None::<String>.to_value(),
            }
        }
    }
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
            .0
            .set(item)
            .expect("ShowObject has already been initialized.");
        slf
    }
    pub fn channel(&self) -> String {
        self.property("channel")
    }
}
