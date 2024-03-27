// Copyright 2023 David Cabot
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{cell::OnceCell, sync::Arc};

use adw::{gio, glib, gtk, prelude::*, subclass::prelude::*};
use eyre::WrapErr;
use gettextrs::gettext;

use crate::utils::{show_error, spawn, tokio};

use super::{
    card::TvLiveCard,
    channels::ChannelObject,
    zapp::{ChannelId, ChannelInfo, Show, ShowsResult, Zapp},
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(file = "src/live/view.blp")]
    pub struct TvLiveView {
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        channels_list: TemplateChild<gtk::ListBox>,

        client: OnceCell<Arc<Zapp>>,
        channels_model: OnceCell<gio::ListStore>,
    }

    impl TvLiveView {
        fn client(&self) -> Arc<Zapp> {
            self.client
                .get_or_init(|| Arc::new(Zapp::new().expect("failed to initialize HTTP client")))
                .clone()
        }
        fn channels_model(&self) -> gio::ListStore {
            self.channels_model
                .get_or_init(gio::ListStore::new::<ChannelObject>)
                .clone()
        }
        async fn load(&self) {
            let client = self.client();
            let load_channels = tokio(async move {
                let list = client
                    .channel_info_list()
                    .await
                    .wrap_err("Failed to load channel info list")?;

                let mut channels: Vec<(ChannelId, ChannelInfo, Option<Vec<Show>>)> =
                    Vec::with_capacity(list.len());

                for (channel_id, channel_info) in list {
                    match client.shows(&channel_id).await.wrap_err_with(|| {
                        eyre::Report::msg(
                            // translators: `{}` is replaced by the channel_id, e.g. `das_erste`
                            gettext("Failed load shows for channel “{}”")
                                .replace("{}", channel_id.as_ref()),
                        )
                    })? {
                        ShowsResult::Shows(shows) => {
                            channels.push((channel_id, channel_info, Some(shows)))
                        }
                        ShowsResult::Error(_) => channels.push((channel_id, channel_info, None)),
                    }
                }

                Ok::<_, eyre::Report>(channels)
            });

            match load_channels.await {
                Ok(channels) => {
                    for (channel_id, channel_info, shows) in channels {
                        let channel = ChannelObject::new(
                            channel_id.as_ref(),
                            &channel_info.name,
                            &channel_info.stream_url,
                        );
                        if let Some(Show {
                            title,
                            subtitle,
                            description,
                            channel: _,
                            start_time,
                            end_time,
                        }) = shows.as_ref().and_then(|shows| shows.first())
                        {
                            channel.set_title(Some(title.as_str()));
                            channel.set_subtitle(subtitle.as_deref());
                            channel.set_description(
                                description.as_deref().map(html2pango::markup).as_deref(),
                            );
                            channel.set_start_time(start_time.unix_timestamp());
                            channel.set_end_time(end_time.unix_timestamp());
                        }
                        self.channels_model().append(&channel)
                    }
                    self.stack.set_visible_child_name("channels");
                    self.spinner.set_spinning(false);
                }
                Err(e) => show_error(e.wrap_err(gettext("Failed to load livestream channels"))),
            }
        }
        pub(super) async fn reload(&self) {
            self.spinner.set_spinning(true);
            self.stack.set_visible_child_name("spinner");
            self.channels_model().remove_all();
            self.load().await;
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TvLiveView {
        const NAME: &'static str = "TvLiveView";
        type Type = super::TvLiveView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TvLiveView {
        fn constructed(&self) {
            self.parent_constructed();

            self.channels_list
                .bind_model(Some(&self.channels_model()), |channel| {
                    glib::Object::builder::<TvLiveCard>()
                        .property("channel", channel)
                        .build()
                        .upcast()
                });

            self.channels_list.connect_row_activated(|_, row| {
                let row = row
                    .downcast_ref::<TvLiveCard>()
                    .expect("invalid ListBoxRow type");
                row.set_expanded(!row.expanded())
            });

            let slf = self.to_owned();
            spawn(async move { slf.load().await });
        }
    }
    impl WidgetImpl for TvLiveView {}
    impl BinImpl for TvLiveView {}
}

glib::wrapper! {
    pub struct TvLiveView(ObjectSubclass<imp::TvLiveView>)
        @extends gtk::Widget, adw::Bin;
}

impl TvLiveView {
    pub fn reload(&self) {
        let slf = self.imp().to_owned();
        spawn(async move { slf.reload().await });
    }
}
