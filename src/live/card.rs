use std::{
    cell::{Cell, RefCell},
    time::Duration,
};

use adw::{glib, gtk, prelude::*, subclass::prelude::*};

use crate::utils::{load_channel_icon, spawn, tokio};

use super::channels::{Channel, ChannelObject};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/live/card.blp")]
    #[properties(wrapper_type=super::MdkLiveCard)]
    pub struct MdkLiveCard {
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

    impl MdkLiveCard {
        fn set_icon(&self) {
            let icon_name = self
                .obj()
                .channel()
                .and_then(|c| c.id().parse::<Channel>().ok())
                .map(|c| c.icon_name());

            load_channel_icon(&self.icon, icon_name);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MdkLiveCard {
        const NAME: &'static str = "MdkLiveCard";
        type Type = super::MdkLiveCard;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("card.play", None, |slf, _, _| {
                slf.activate_action(
                    "app.play",
                    Some(&slf.channel().unwrap().stream_url().to_variant()),
                )
                .unwrap()
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MdkLiveCard {
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

            self.obj().connect_channel_notify(|slf| {
                if let Some(channel) = slf.channel() {
                    slf.imp()
                        .play_button
                        .set_action_target_value(Some(&channel.stream_url().to_variant()));
                }
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
    impl WidgetImpl for MdkLiveCard {}
    impl ListBoxRowImpl for MdkLiveCard {}
}

glib::wrapper! {
    pub struct MdkLiveCard(ObjectSubclass<imp::MdkLiveCard>)
        @extends gtk::Widget, gtk::ListBoxRow;
}
