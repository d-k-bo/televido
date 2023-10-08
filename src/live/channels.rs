use std::cell::{Cell, RefCell};

use adw::{glib, prelude::*, subclass::prelude::*};

use crate::{
    application::MdkApplication,
    utils::{channel_mapping, format_timestamp_time},
};

mod imp {
    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::ChannelObject)]
    pub struct ChannelObject {
        #[property(get, construct_only)]
        id: RefCell<String>,
        #[property(get, construct_only)]
        name: RefCell<String>,
        #[property(get, set, nullable)]
        title: RefCell<Option<String>>,
        #[property(get, set, nullable)]
        subtitle: RefCell<Option<String>>,
        #[property(get, set, nullable)]
        description: RefCell<Option<String>>,
        #[property(get, set)]
        start_time: Cell<i64>,
        #[property(get, set)]
        end_time: Cell<i64>,
        #[property(get)]
        duration: Cell<i64>,
        #[property(get)]
        timespan: RefCell<Option<String>>,
        #[property(get, construct_only)]
        stream_url: RefCell<String>,
    }

    impl ChannelObject {
        fn update_time(&self) {
            self.update_timespan();
            self.update_duration();
        }
        fn update_timespan(&self) {
            let obj = self.obj();
            let start_time = obj.start_time();
            let end_time = obj.end_time();

            let timespan = if start_time == 0 || end_time == 0 {
                None
            } else if let (Some(start_time), Some(end_time)) = (
                format_timestamp_time(start_time),
                format_timestamp_time(end_time),
            ) {
                Some(format!("{start_time} - {end_time}",))
            } else {
                None
            };

            self.timespan.replace(timespan);
            obj.notify_timespan();
        }
        fn update_duration(&self) {
            let obj = self.obj();
            let start_time = obj.start_time();
            let end_time = obj.end_time();

            let duration = if start_time == 0 || end_time == 0 {
                0
            } else {
                end_time - start_time
            };

            self.duration.replace(duration);
            obj.notify_duration();
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChannelObject {
        const NAME: &'static str = "ChannelObject";
        type Type = super::ChannelObject;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ChannelObject {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.connect_start_time_notify(|obj| obj.imp().update_time());
            obj.connect_end_time_notify(|obj| obj.imp().update_time());
        }
    }
}

glib::wrapper! {
    pub struct ChannelObject(ObjectSubclass<imp::ChannelObject>);
}

impl ChannelObject {
    pub fn new(id: &str, name: &str, stream_url: &str) -> Self {
        glib::Object::builder()
            .property("id", id)
            .property("name", name)
            .property("stream-url", stream_url)
            .build()
    }
    pub fn channel_icon(&self) -> Option<String> {
        match self.id().parse::<Channel>() {
            Ok(channel) => {
                let application = MdkApplication::get();

                Some(application.channel_icon(channel.icon_name()))
            }
            Err(_) => None,
        }
    }
}

channel_mapping! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Channel {
        #[channel(name = "ard_alpha", icon = "ard-alpha.svg")]
        ArdAlpha,
        #[channel(name = "arte", icon = "arte.svg")]
        Arte,
        #[channel(name = "br_nord", icon = "br.svg")]
        BrNord,
        #[channel(name = "br_sued", icon = "br.svg")]
        BrSüd,
        #[channel(name = "das_erste", icon = "das-erste.svg")]
        DasErste,
        #[channel(name = "deutsche_welle", icon = "deutsche-welle.svg")]
        DeutscheWelle,
        #[channel(name = "deutsche_welle_plus", icon = "deutsche-welle.svg")]
        DeutscheWellePlus,
        #[channel(name = "dreisat", icon = "3sat.svg")]
        DreiSat,
        #[channel(name = "hr", icon = "hr.svg")]
        Hr,
        #[channel(name = "kika", icon = "kika.svg")]
        KiKa,
        #[channel(name = "mdr_sachsen", icon = "mdr.svg")]
        MdrSachsen,
        #[channel(name = "mdr_sachsen_anhalt", icon = "mdr.svg")]
        MdrSachsenAnhalt,
        #[channel(name = "mdr_thueringen", icon = "mdr.svg")]
        MdrThüringen,
        #[channel(name = "ndr_hh", icon = "ndr.svg")]
        NdrHH,
        #[channel(name = "ndr_mv", icon = "ndr.svg")]
        NdrMV,
        #[channel(name = "ndr_nds", icon = "ndr.svg")]
        NdrNds,
        #[channel(name = "ndr_sh", icon = "ndr.svg")]
        NdrSH,
        #[channel(name = "one", icon = "one.svg")]
        One,
        #[channel(name = "parlamentsfernsehen_1", icon = "parlamentsfernsehen.svg")]
        Parlamentsfernsehen1,
        #[channel(name = "parlamentsfernsehen_2", icon = "parlamentsfernsehen.svg")]
        Parlamentsfernsehen2,
        #[channel(name = "phoenix", icon = "phoenix.svg")]
        Phoenix,
        #[channel(name = "rb", icon = "rb.svg")]
        Rb,
        #[channel(name = "rbb_berlin", icon = "rbb.svg")]
        RbbBerlin,
        #[channel(name = "rbb_brandenburg", icon = "rbb.svg")]
        RbbBrandenburg,
        #[channel(name = "sr", icon = "sr.svg")]
        Sr,
        #[channel(name = "swr_bw", icon = "swr.svg")]
        SwrBW,
        #[channel(name = "swr_rp", icon = "swr.svg")]
        SwrRP,
        #[channel(name = "tagesschau24", icon = "tagesschau24.svg")]
        Tagesschau24,
        #[channel(name = "wdr", icon = "wdr.svg")]
        Wdr,
        #[channel(name = "zdf", icon = "zdf.svg")]
        Zdf,
        #[channel(name = "zdf_info", icon = "zdf-info.svg")]
        ZdfInfo,
        #[channel(name = "zdf_neo", icon = "zdf-neo.svg")]
        ZdfNeo,
    }
}
