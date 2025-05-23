// SPDX-FileCopyrightText: David Cabot <d-k-bo@mailbox.org>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::{Cell, RefCell};

use adw::{gio, glib, gtk, prelude::*, subclass::prelude::*};
use gettextrs::gettext;

use crate::{
    application::TvApplication,
    channel_icons::load_channel_icon,
    player::VideoInfo,
    settings::{TvSettings, VideoQuality},
    utils::{show_error, spawn},
};

use super::shows::ShowObject;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/mediathek/card.blp")]
    #[properties(wrapper_type=super::TvMediathekCard)]
    pub struct TvMediathekCard {
        #[template_child]
        icon: TemplateChild<gtk::Image>,
        #[template_child]
        revealer: TemplateChild<gtk::Revealer>,

        #[property(get, construct_only)]
        show: RefCell<Option<ShowObject>>,

        #[property(get, set)]
        expanded: Cell<bool>,
    }
    impl TvMediathekCard {
        fn set_icon(&self) {
            load_channel_icon(
                self.obj().show().map(|c| c.channel()).as_deref(),
                &self.icon,
                64,
            )
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TvMediathekCard {
        const NAME: &'static str = "TvMediathekCard";
        type Type = super::TvMediathekCard;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for TvMediathekCard {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().setup_actions();
            self.obj().connect_show_notify(|slf| slf.imp().set_icon());

            self.revealer.connect_child_revealed_notify(|revealer| {
                revealer.set_visible(revealer.is_child_revealed())
            });
        }
    }
    impl WidgetImpl for TvMediathekCard {}
    impl ListBoxRowImpl for TvMediathekCard {}
}

glib::wrapper! {
    pub struct TvMediathekCard(ObjectSubclass<imp::TvMediathekCard>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl TvMediathekCard {
    fn play(&self, quality: VideoQuality) {
        let show = self
            .show()
            .expect("action must only be enabled if show is not None");

        spawn(async move {
            TvApplication::get()
                .play(VideoInfo::Mediathek {
                    title: show.title(),
                    preferred_quality: quality,
                    subtitle_uri: show.subtitle_url(),
                    uri_high: show.video_url_high(),
                    uri_medium: show.video_url_medium(),
                    uri_low: show.video_url_low(),
                    channel_id: show.channel(),
                })
                .await
        });
    }
    fn copy_video_url(&self, quality: VideoQuality) {
        self.clipboard().set(
            &self
                .show()
                .and_then(|show| show.video_url(quality))
                .expect("action must only be enabled if url is not None"),
        );

        TvApplication::get()
            .window()
            .add_toast(adw::Toast::new(&gettext("Copied video URL to clipboard")));
    }
    fn copy_subtitles_url(&self) {
        self.clipboard().set(
            &self
                .show()
                .and_then(|show| show.subtitle_url())
                .expect("action must only be enabled if url is not None"),
        );

        TvApplication::get()
            .window()
            .add_toast(adw::Toast::new(&gettext(
                "Copied subtitles URL to clipboard",
            )));
    }
    fn download(&self) {
        self.activate_action(
            "app.download",
            Some(
                &self
                    .show()
                    .and_then(|show| show.website_url())
                    .expect("action must only be enabled if url is not None")
                    .to_variant(),
            ),
        )
        .unwrap()
    }
    fn setup_actions(&self) {
        let actions = gio::SimpleActionGroup::new();

        macro_rules! video_url_action {
            ( $name:literal, $method:ident, $quality:expr) => {{
                let action = gio::SimpleAction::new($name, None);
                action.connect_activate(glib::clone!(
                    #[weak(rename_to = slf)]
                    self,
                    move |_, _| slf.$method($quality)
                ));
                self.connect_show_notify(glib::clone!(
                    #[weak]
                    action,
                    move |slf| {
                        action.set_enabled(
                            slf.show()
                                .and_then(|show| show.video_url($quality))
                                .is_some(),
                        );
                    }
                ));
                actions.add_action(&action);
                action
            }};
        }

        let play_default =
            video_url_action!("play-default", play, VideoQuality::default_playback());
        TvSettings::get().connect_default_playback_quality_changed(glib::clone!(
            #[weak(rename_to = slf)]
            self,
            #[weak]
            play_default,
            move |_| {
                play_default.set_enabled(
                    slf.show()
                        .and_then(|show| show.video_url(VideoQuality::default_playback()))
                        .is_some(),
                );
            }
        ));
        video_url_action!("play-high", play, VideoQuality::High);
        video_url_action!("play-medium", play, VideoQuality::Medium);
        video_url_action!("play-low", play, VideoQuality::Low);

        video_url_action!("copy-url-high", copy_video_url, VideoQuality::High);
        video_url_action!("copy-url-medium", copy_video_url, VideoQuality::Medium);
        video_url_action!("copy-url-low", copy_video_url, VideoQuality::Low);

        let copy_subtitles_url = gio::SimpleAction::new("copy-subtitles-url", None);
        copy_subtitles_url.connect_activate(glib::clone!(
            #[weak(rename_to = slf)]
            self,
            move |_, _| slf.copy_subtitles_url()
        ));
        self.connect_show_notify(glib::clone!(
            #[weak]
            copy_subtitles_url,
            move |slf| {
                copy_subtitles_url
                    .set_enabled(slf.show().and_then(|show| show.subtitle_url()).is_some());
            }
        ));
        actions.add_action(&copy_subtitles_url);

        let download = gio::SimpleAction::new("download", None);
        download.connect_activate(glib::clone!(
            #[weak(rename_to = slf)]
            self,
            move |_, _| slf.download()
        ));
        self.connect_show_notify(glib::clone!(
            #[weak]
            download,
            move |slf| {
                download.set_enabled(slf.show().and_then(|show| show.website_url()).is_some());
            }
        ));
        actions.add_action(&download);

        let open_website = gio::SimpleAction::new("open-website", None);
        open_website.connect_activate(glib::clone!(
            #[weak(rename_to = slf)]
            self,
            move |_, _| spawn(async move {
                let url = slf
                    .show()
                    .and_then(|show| show.website_url())
                    .expect("action must only be enabled if url is not None");
                if let Err(e) = gtk::UriLauncher::new(&url)
                    .launch_future(slf.root().and_downcast_ref::<adw::Window>())
                    .await
                {
                    show_error(
                        eyre::Report::msg(e.to_string())
                            .wrap_err(gettext("Failed to open website in browser")),
                    );
                }
            })
        ));
        self.connect_show_notify(glib::clone!(
            #[weak]
            open_website,
            move |slf| {
                open_website.set_enabled(slf.show().and_then(|show| show.website_url()).is_some());
            }
        ));
        actions.add_action(&open_website);

        self.insert_action_group("card", Some(&actions));
    }
}
