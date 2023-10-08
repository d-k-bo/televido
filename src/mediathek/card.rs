use std::cell::RefCell;

use adw::{glib, gtk, prelude::*, subclass::prelude::*};

use crate::application::MdkApplication;

use super::{channels::Channel, shows::ShowObject};

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/mediathek/card.blp")]
    #[properties(wrapper_type=super::MdkMediathekCard)]
    pub struct MdkMediathekCard {
        #[property(get, construct_only)]
        pub(super) show: RefCell<Option<ShowObject>>,
        #[template_child]
        icon: TemplateChild<gtk::Image>,
    }
    impl MdkMediathekCard {
        fn set_icon(&self) {
            match self
                .obj()
                .show()
                .and_then(|c| c.channel().parse::<Channel>().ok())
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
    impl ObjectSubclass for MdkMediathekCard {
        const NAME: &'static str = "MdkMediathekCard";
        type Type = super::MdkMediathekCard;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MdkMediathekCard {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_show_notify(|slf| slf.imp().set_icon());
            MdkApplication::get()
                .style_manager()
                .connect_dark_notify(glib::clone!(@weak self as slf => move |_| slf.set_icon()));
        }
    }
    impl WidgetImpl for MdkMediathekCard {}
    impl ListBoxRowImpl for MdkMediathekCard {}
}

glib::wrapper! {
    pub struct MdkMediathekCard(ObjectSubclass<imp::MdkMediathekCard>)
        @extends gtk::Widget, gtk::ListBoxRow;
}
