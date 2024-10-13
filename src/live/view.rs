// SPDX-FileCopyrightText: David Cabot <d-k-bo@mailbox.org>
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::{gio, glib, gtk, prelude::*, subclass::prelude::*};
use eyre::WrapErr;
use gettextrs::gettext;
use smart_default::SmartDefault;

use crate::{
    application::TvApplication,
    settings::TvSettings,
    utils::{show_error, spawn, tokio},
    zapp::{ChannelId, ChannelInfo, Show, ShowsResult},
};

use super::{card::TvLiveCard, channels::ChannelObject};

mod imp {
    use super::*;

    #[derive(Debug, SmartDefault, gtk::CompositeTemplate)]
    #[template(file = "src/live/view.blp")]
    pub struct TvLiveView {
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        channels_list: TemplateChild<gtk::ListBox>,
    }

    impl TvLiveView {
        async fn load_channels(&self) -> eyre::Result<gio::ListStore> {
            let client = TvApplication::get().zapp();
            let settings = TvSettings::get();
            let visible_channels = settings.live_channels();

            let live_channels = TvApplication::get().live_channels().await;

            let live_channels: Vec<(ChannelId, ChannelInfo)> = if visible_channels.is_empty() {
                live_channels
                    .iter()
                    .map(|(channel_id, channel_info)| (channel_id.clone(), channel_info.clone()))
                    .collect()
            } else {
                visible_channels
                    .into_iter()
                    .filter_map(|channel_id| {
                        let channel_info = live_channels.get(&channel_id)?;
                        Some((channel_id, channel_info.clone()))
                    })
                    .collect()
            };

            let load_channels = tokio(async move {
                let mut channels: Vec<(ChannelId, ChannelInfo, Option<Vec<Show>>)> =
                    Vec::with_capacity(live_channels.len());

                for (channel_id, channel_info) in live_channels.iter() {
                    match client.shows(channel_id).await.wrap_err_with(|| {
                        eyre::Report::msg(
                            // translators: `{}` is replaced by the channel_id, e.g. `das_erste`
                            gettext("Failed load shows for channel “{}”")
                                .replace("{}", channel_id.as_ref()),
                        )
                    })? {
                        ShowsResult::Shows(shows) => {
                            channels.push((channel_id.clone(), channel_info.clone(), Some(shows)))
                        }
                        ShowsResult::Error(_) => {
                            channels.push((channel_id.clone(), channel_info.clone(), None))
                        }
                    }
                }

                Ok::<_, eyre::Report>(channels)
            });

            let channel_objects = load_channels
                .await?
                .into_iter()
                .map(|(channel_id, channel_info, shows)| {
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
                    channel
                })
                .collect::<gio::ListStore>();

            Ok(channel_objects)
        }
        pub(super) async fn reload(&self) {
            self.spinner.set_spinning(true);
            self.stack.set_visible_child_name("spinner");

            match self.load_channels().await {
                Ok(channels) => {
                    self.channels_list.bind_model(Some(&channels), |channel| {
                        glib::Object::builder::<TvLiveCard>()
                            .property("channel", channel)
                            .build()
                            .upcast()
                    });
                    self.stack.set_visible_child_name("channels");
                    self.spinner.set_spinning(false);
                }
                Err(e) => show_error(e.wrap_err(gettext("Failed to load livestream channels"))),
            }
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

            let settings = TvSettings::get();

            self.channels_list.connect_row_activated(|_, row| {
                let row = row
                    .downcast_ref::<TvLiveCard>()
                    .expect("invalid ListBoxRow type");
                row.set_expanded(!row.expanded())
            });

            let slf = self.to_owned();
            spawn(async move { slf.reload().await });

            settings.connect_live_channels_changed(glib::clone!(
                #[weak(rename_to = slf)]
                self,
                move |_| {
                    spawn(async move { slf.reload().await });
                }
            ));
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
