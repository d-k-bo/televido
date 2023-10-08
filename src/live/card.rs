use std::{cell::RefCell, time::Duration};

use adw::{glib, gtk, prelude::*, subclass::prelude::*};

use crate::{
    application::MdkApplication,
    utils::{spawn, tokio},
};

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
        progress: TemplateChild<gtk::ProgressBar>,

        #[property(get, construct_only)]
        pub(super) channel: RefCell<Option<ChannelObject>>,
    }

    impl MdkLiveCard {
        fn set_icon(&self) {
            match self
                .obj()
                .channel()
                .and_then(|c| c.id().parse::<Channel>().ok())
            {
                Some(channel) => {
                    let application = MdkApplication::get();
                    let icon = application.channel_icon(channel.icon_name());

                    self.icon.set_resource(Some(&icon));
                }
                None => self.icon.set_icon_name(Some("image-missing-symbolic")),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MdkLiveCard {
        const NAME: &'static str = "MdkLiveCard";
        type Type = super::MdkLiveCard;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
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
            MdkApplication::get()
                .style_manager()
                .connect_dark_notify(glib::clone!(@weak self as slf => move |_| slf.set_icon()));

            self.title
                .connect_label_notify(|label| label.set_visible(!label.label().is_empty()));

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
