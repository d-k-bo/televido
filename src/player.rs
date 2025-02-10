// SPDX-FileCopyrightText: David Cabot <d-k-bo@mailbox.org>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::{Cell, RefCell};

use adw::{gio, glib, gtk, prelude::*, subclass::prelude::*};

use eyre::WrapErr;
use tracing::error;

use crate::{
    application::TvApplication,
    channel_icons::channel_icon_resource,
    config::{APP_ID, APP_NAME},
    settings::{TvPlayerSettings, VideoQuality},
    utils::{spawn, tokio},
};
mod imp {
    use super::*;

    #[derive(smart_default::SmartDefault, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/player.blp")]
    #[properties(wrapper_type=super::TvPlayer)]
    pub struct TvPlayer {
        #[template_child]
        pub(super) seek_bar: TemplateChild<clapper_gtk::SeekBar>,
        #[template_child]
        pub(super) clapper_menu_button: TemplateChild<clapper_gtk::ExtraMenuButton>,
        #[template_child]
        pub(super) custom_menu_button: TemplateChild<gtk::MenuButton>,

        #[property(
            name = "player",
            type = clapper::Player,
            get = |slf: &TvPlayer| slf.video.player().expect("should not be nullable")
        )]
        #[template_child]
        pub(super) video: TemplateChild<clapper_gtk::Video>,
        #[property(get)]
        #[default(clapper::Mpris::new(
            &format!("org.mpris.MediaPlayer2.{APP_ID}"),
            APP_NAME,
            Some(APP_ID),
        ))]
        mpris: clapper::Mpris,

        #[property(get, set)]
        subtitles_enabled: Cell<bool>,

        #[property(get, set)]
        title: RefCell<String>,
        #[property(get, set)]
        uri: RefCell<String>,
        #[property(get, set, nullable)]
        subtitle_uri: RefCell<Option<String>>,
        #[property(get, set, nullable)]
        uri_high: RefCell<Option<String>>,
        #[property(get, set, nullable)]
        uri_medium: RefCell<Option<String>>,
        #[property(get, set, nullable)]
        uri_low: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TvPlayer {
        const NAME: &'static str = "TvPlayer";
        type Type = super::TvPlayer;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("player.switch-to-high-quality", None, |slf, _, _| {
                slf.set_quality(VideoQuality::High);
                slf.start_playback();
            });
            klass.install_action("player.switch-to-medium-quality", None, |slf, _, _| {
                slf.set_quality(VideoQuality::Medium);
                slf.start_playback();
            });
            klass.install_action("player.switch-to-low-quality", None, |slf, _, _| {
                slf.set_quality(VideoQuality::Low);
                slf.start_playback();
            });

            klass.install_property_action("player.enable-subtitles", "subtitles-enabled");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for TvPlayer {
        fn constructed(&self) {
            self.parent_constructed();

            let slf = self.obj();

            slf.player().add_feature(&slf.mpris());

            let taginject = gst::ElementFactory::make("taginject")
                .property("scope", gst::TagScope::Global)
                .build()
                .expect("failed to create `taginject` element");

            slf.player().set_video_filter(Some(&taginject));

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

            slf.bind_property("subtitles-enabled", &slf.player(), "subtitles-enabled")
                .bidirectional()
                .build();

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
            settings
                .bind_subtitles_enabled(&*slf, "subtitles-enabled")
                .build();
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
    pub fn play(&self, video: VideoInfo) {
        self.player().stop();

        match video {
            VideoInfo::Live {
                title,
                uri,
                channel_id,
            } => {
                self.set_title(title);
                self.set_channel_icon(channel_id);

                self.imp().seek_bar.set_reveal_labels(false);
                self.imp().clapper_menu_button.set_visible(true);
                self.imp().custom_menu_button.set_visible(false);

                self.set_uri(uri);
                self.set_subtitle_uri(None::<&str>);
                self.set_uri_high(None::<&str>);
                self.set_uri_medium(None::<&str>);
                self.set_uri_low(None::<&str>);
            }
            VideoInfo::Mediathek {
                title,
                preferred_quality,
                subtitle_uri,
                uri_high,
                uri_medium,
                uri_low,
                channel_id,
            } => {
                self.set_title(title);
                self.set_channel_icon(channel_id);

                self.imp().seek_bar.set_reveal_labels(true);
                self.imp().clapper_menu_button.set_visible(false);
                self.imp().custom_menu_button.set_visible(true);
                self.action_set_enabled("player.switch-to-high-quality", uri_high.is_some());
                self.action_set_enabled("player.switch-to-medium-quality", uri_medium.is_some());
                self.action_set_enabled("player.switch-to-low-quality", uri_low.is_some());
                self.action_set_enabled("player.enable-subtitles", subtitle_uri.is_some());

                self.set_subtitle_uri(subtitle_uri);
                self.set_uri_high(uri_high);
                self.set_uri_medium(uri_medium);
                self.set_uri_low(uri_low);
                self.set_quality(preferred_quality);
            }
        }
        self.stop_playback();
        self.start_playback();
    }
    fn set_channel_icon(&self, channel_id: String) {
        let mpris = self.mpris();
        if let Some(icon_resource) = channel_icon_resource(&channel_id) {
            spawn(async move {
                if let Err(e) = async {
                    let icon_data =
                        gio::resources_lookup_data(&icon_resource, gio::ResourceLookupFlags::NONE)
                            .wrap_err("failed to load resource bytes")?;

                    let icon_path =
                        glib::user_cache_dir().join(format!("televido/{channel_id}.svg"));

                    tokio({
                        let icon_path = icon_path.clone();
                        async move {
                            tokio::fs::create_dir_all(icon_path.parent().unwrap()).await?;
                            tokio::fs::write(&icon_path, icon_data).await?;
                            Ok::<(), eyre::Report>(())
                        }
                    })
                    .await?;

                    mpris.set_fallback_art_url(
                        icon_path.to_str().map(|p| format!("file://{p}")).as_deref(),
                    );

                    Ok::<(), eyre::Report>(())
                }
                .await
                {
                    error!("{e:?}");
                }
            });
        }
    }
    fn set_quality(&self, quality: VideoQuality) {
        match quality {
            VideoQuality::High => {
                self.set_uri(self.uri_high().expect("no high qualtity video URI set"));
            }
            VideoQuality::Medium => {
                self.set_uri(self.uri_medium().expect("no medium qualtity video URI set"));
            }
            VideoQuality::Low => {
                self.set_uri(self.uri_low().expect("no low qualtity video URI set"));
            }
        }
    }
    fn start_playback(&self) {
        let player = self.player();
        let queue = player.queue().expect("should not be nullable");

        let position = player.position();

        let item = clapper::MediaItem::new(&self.uri());

        // Adding subtitles currently breaks playback
        // see https://gitlab.freedesktop.org/gstreamer/gstreamer/-/issues/4066
        // if let Some(subtitle_uri) = self.subtitle_uri() {
        //     item.set_suburi(&subtitle_uri);
        // }

        queue.add_item(&item);
        queue.select_item(Some(&item));

        // seek to previous position to continue playback
        // this should be a simple sequence of pause -> seek -> play
        // unfortunately we need to check the state before each step
        if position > 0.0 {
            player.pause();

            crate::utils::spawn(async move {
                loop {
                    if player.state() == clapper::PlayerState::Paused {
                        player.seek(position);

                        let handler_id = std::rc::Rc::new(RefCell::new(None));
                        handler_id.replace(Some(player.connect_seek_done({
                            let handler_id = handler_id.clone();
                            move |player| {
                                player.play();
                                glib::signal_handler_disconnect(player, handler_id.take().unwrap());
                            }
                        })));
                        break;
                    } else {
                        glib::timeout_future(std::time::Duration::from_millis(50)).await;
                    }
                }
            });
        } else {
            player.play();
        }
    }
    fn stop_playback(&self) {
        self.player().stop();
    }
}
#[derive(Debug)]
pub enum VideoInfo {
    Live {
        title: String,
        uri: String,
        channel_id: String,
    },
    Mediathek {
        title: String,
        preferred_quality: VideoQuality,
        subtitle_uri: Option<String>,
        uri_high: Option<String>,
        uri_medium: Option<String>,
        uri_low: Option<String>,
        channel_id: String,
    },
}
