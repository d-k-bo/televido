use std::cell::{Cell, RefCell};

use adw::{glib, gtk, prelude::*, subclass::prelude::*};

use crate::utils::load_channel_icon;

use super::{channels::Channel, shows::ShowObject};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/mediathek/card.blp")]
    #[properties(wrapper_type=super::MdkMediathekCard)]
    pub struct MdkMediathekCard {
        #[template_child]
        icon: TemplateChild<gtk::Image>,
        #[template_child]
        revealer: TemplateChild<gtk::Revealer>,

        #[property(get, construct_only)]
        show: RefCell<Option<ShowObject>>,

        #[property(get, set)]
        expanded: Cell<bool>,
    }
    impl MdkMediathekCard {
        fn set_icon(&self) {
            let icon_name = self
                .obj()
                .show()
                .and_then(|c| c.channel().parse::<Channel>().ok())
                .map(|c| c.icon_name());

            load_channel_icon(&self.icon, icon_name);
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

            self.revealer.connect_child_revealed_notify(|revealer| {
                revealer.set_visible(revealer.is_child_revealed())
            });
        }
    }
    impl WidgetImpl for MdkMediathekCard {}
    impl ListBoxRowImpl for MdkMediathekCard {}
}

glib::wrapper! {
    pub struct MdkMediathekCard(ObjectSubclass<imp::MdkMediathekCard>)
        @extends gtk::Widget, gtk::ListBoxRow;
}
