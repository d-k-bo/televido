// SPDX-FileCopyrightText: David Cabot <d-k-bo@mailbox.org>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    cell::{Cell, RefCell},
    time::Duration,
};

use adw::{glib, gtk, prelude::*, subclass::prelude::*};

use crate::{
    channel_icons::load_channel_icon,
    player::VideoInfo,
    utils::{spawn, tokio},
    TvApplication,
};

use super::channels::ChannelObject;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/live/card.blp")]
    #[properties(wrapper_type=super::TvLiveCard)]
    pub struct TvLiveCard {
        #[template_child]
        icon: TemplateChild<gtk::Image>,
        #[template_child]
        title: TemplateChild<gtk::Label>,
        #[template_child]
        subtitle: TemplateChild<gtk::Label>,
        #[template_child]
        progress: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        play_button: TemplateChild<gtk::Button>,

        #[property(get, set)]
        expanded: Cell<bool>,

        #[property(get, construct_only)]
        pub(super) channel: RefCell<Option<ChannelObject>>,
    }

    impl TvLiveCard {
        fn set_icon(&self) {
            load_channel_icon(
                self.obj().channel().map(|c| c.id()).as_deref(),
                &self.icon,
                64,
            )
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TvLiveCard {
        const NAME: &'static str = "TvLiveCard";
        type Type = super::TvLiveCard;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async("card.play", None, |slf, _, _| async move {
                let channel = slf.channel().unwrap();
                TvApplication::get()
                    .play(VideoInfo::Live {
                        title: channel.name(),
                        uri: channel.stream_url(),
                        channel_id: channel.id(),
                    })
                    .await
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for TvLiveCard {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj()
                .connect_channel_notify(|slf| slf.imp().set_icon());

            self.title
                .connect_label_notify(|title| title.set_visible(!title.label().is_empty()));

            self.subtitle.connect_label_notify(|subtitle| {
                subtitle.set_visible(!subtitle.label().is_empty())
            });

            self.revealer.connect_child_revealed_notify(|revealer| {
                revealer.set_visible(revealer.is_child_revealed())
            });

            // update progress bar every 10 seconds
            let slf = self.downgrade();
            spawn(async move {
                loop {
                    let Some(slf) = slf.upgrade() else { break };

                    if let Some(channel) = slf.channel.borrow().as_ref() {
                        let start_time = channel.start_time();
                        let end_time = channel.end_time();
                        let Ok(now) = glib::DateTime::now_local() else {
                            continue;
                        };

                        if start_time == 0 || end_time == 0 {
                            slf.progress.set_visible(false);
                        } else {
                            let fraction = (((now.to_unix() - start_time) as f64)
                                / ((end_time - start_time) as f64))
                                .max(0.0);
                            slf.progress.set_fraction(fraction);
                            slf.progress.set_visible(true);
                        }
                    }
                    tokio(async { tokio::time::sleep(Duration::from_secs(10)).await }).await;
                }
            });
        }
    }
    impl WidgetImpl for TvLiveCard {}
    impl ListBoxRowImpl for TvLiveCard {}
}

glib::wrapper! {
    pub struct TvLiveCard(ObjectSubclass<imp::TvLiveCard>)
        @extends gtk::Widget, gtk::ListBoxRow;
}
