// Copyright 2023 David Cabot
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{cell::OnceCell, sync::Arc};

use adw::{gio, glib, gtk, prelude::*, subclass::prelude::*};
use tracing::error;

use crate::utils::{delegate_actions, spawn, tokio};

use super::{
    card::MdkLiveCard,
    channels::ChannelObject,
    zapp::{ChannelId, ChannelInfo, Show, ShowsResult, Zapp},
};

mod imp {
    use eyre::Context;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate /* , glib::Properties */)]
    #[template(file = "src/live/view.blp")]
    // #[properties(wrapper_type = super::MdkLiveView)]
    pub struct MdkLiveView {
        actions: gio::SimpleActionGroup,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        channels_list: TemplateChild<gtk::ListBox>,

        client: OnceCell<Arc<Zapp>>,
        channels_model: OnceCell<gio::ListStore>,
    }

    impl MdkLiveView {
        pub(super) fn client(&self) -> Arc<Zapp> {
            self.client
                .get_or_init(|| Arc::new(Zapp::new().expect("failed to initialize HTTP client")))
                .clone()
        }
        pub(super) fn channels_model(&self) -> gio::ListStore {
            self.channels_model
                .get_or_init(gio::ListStore::new::<ChannelObject>)
                .clone()
        }
        pub async fn load(&self) {
            let client = self.client();

            match tokio(async move {
                let list = client
                    .channel_info_list()
                    .await
                    .wrap_err("failed to load channel info list")?;

                let mut channels: Vec<(ChannelId, ChannelInfo, Option<Vec<Show>>)> =
                    Vec::with_capacity(list.len());

                for (channel_id, channel_info) in list {
                    match client.shows(&channel_id).await.wrap_err_with(|| {
                        eyre::eyre!("failed to load shows for channel '{channel_id}'")
                    })? {
                        ShowsResult::Shows(shows) => {
                            channels.push((channel_id, channel_info, Some(shows)))
                        }
                        ShowsResult::Error(_) => channels.push((channel_id, channel_info, None)),
                    }
                }

                Ok::<_, eyre::Report>(channels)
            })
            .await
            {
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
                Err(e) => error!("{:?}", e.wrap_err("failed to load livestream channels")),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MdkLiveView {
        const NAME: &'static str = "MdkLiveView";
        type Type = super::MdkLiveView;
        type ParentType = adw::Bin;
        type Interfaces = (gio::ActionGroup, gio::ActionMap);

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // #[glib::derived_properties]
    impl ObjectImpl for MdkLiveView {
        fn constructed(&self) {
            self.parent_constructed();

            self.channels_list
                .bind_model(Some(&self.channels_model()), |channel| {
                    glib::Object::builder::<MdkLiveCard>()
                        .property("channel", channel)
                        .build()
                        .upcast()
                });

            self.channels_list.connect_row_activated(|_, row| {
                let row = row
                    .downcast_ref::<MdkLiveCard>()
                    .expect("invalid ListBoxRow type");
                row.set_expanded(!row.expanded())
            });

            let slf = self.to_owned();
            spawn(async move { slf.load().await });
        }
    }
    impl WidgetImpl for MdkLiveView {}
    impl BinImpl for MdkLiveView {}
    delegate_actions! {MdkLiveView, actions }
}

glib::wrapper! {
    pub struct MdkLiveView(ObjectSubclass<imp::MdkLiveView>)
        @extends gtk::Widget, adw::Bin,
        @implements gio::ActionGroup, gio::ActionMap;
}
