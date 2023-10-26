// Copyright 2023 David Cabot
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    cell::{Cell, OnceCell, RefCell},
    sync::Arc,
    time::Duration,
};

use adw::{gio, glib, gtk, prelude::*, subclass::prelude::*};
use eyre::WrapErr;
use gettextrs::gettext;
use mediathekviewweb::{
    models::{QueryField, SortField, SortOrder},
    Mediathek,
};

use crate::{
    config::{APP_ID, PROJECT_URL, VERSION},
    settings::TvSettings,
    utils::{show_error, spawn, spawn_clone, tokio},
};

use super::{card::TvMediathekCard, shows::ShowObject};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/mediathek/view.blp")]
    #[properties(wrapper_type = super::TvMediathekView)]
    pub struct TvMediathekView {
        #[template_child]
        search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        results_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        nothing_found_view: TemplateChild<adw::StatusPage>,
        #[template_child]
        results_view: TemplateChild<gtk::ScrolledWindow>,

        #[property(get, set)]
        query_string: RefCell<String>,
        #[property(get, set)]
        search_in_topic: Cell<bool>,
        #[property(get, set)]
        search_in_title: Cell<bool>,
        #[property(get, set)]
        search_in_description: Cell<bool>,
        #[property(get, set)]
        search_in_channel: Cell<bool>,
        #[property(get, set)]
        include_future: Cell<bool>,

        #[property(get, set)]
        sort_by: RefCell<String>,
        #[property(get, set)]
        sort_order: RefCell<String>,

        #[property(get, set)]
        offset: Cell<u64>,

        #[property(get, set)]
        show_results: Cell<bool>,

        pub(super) client: OnceCell<Arc<Mediathek>>,
        pub(super) shows_model: OnceCell<gio::ListStore>,
        pub(super) request_id: Cell<u32>,
    }
    impl TvMediathekView {
        pub(super) fn shows_model(&self) -> gio::ListStore {
            self.shows_model
                .get_or_init(gio::ListStore::new::<ShowObject>)
                .clone()
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TvMediathekView {
        const NAME: &'static str = "TvMediathekView";
        type Type = super::TvMediathekView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_property_action("mediathek.search-in-topic", "search-in-topic");
            klass.install_property_action("mediathek.search-in-title", "search-in-title");
            klass.install_property_action(
                "mediathek.search-in-description",
                "search-in-description",
            );
            klass.install_property_action("mediathek.search-in-channel", "search-in-channel");
            klass.install_property_action("mediathek.sort-by", "sort-by");
            klass.install_property_action("mediathek.sort-order", "sort-order");
            klass.install_action_async("mediathek.load-more", None, |slf, _, _| async move {
                slf.load_more().await
            })
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for TvMediathekView {
        fn constructed(&self) {
            self.parent_constructed();

            let slf = self.obj();
            let settings = TvSettings::get();
            settings
                .bind_search_in_topic(&*slf, "search-in-topic")
                .flags(gio::SettingsBindFlags::DEFAULT)
                .build();
            settings
                .bind_search_in_title(&*slf, "search-in-title")
                .flags(gio::SettingsBindFlags::DEFAULT)
                .build();
            settings
                .bind_search_in_description(&*slf, "search-in-description")
                .flags(gio::SettingsBindFlags::DEFAULT)
                .build();
            settings
                .bind_search_in_channel(&*slf, "search-in-channel")
                .flags(gio::SettingsBindFlags::DEFAULT)
                .build();
            settings
                .bind_sort_by(&*slf, "sort-by")
                .flags(gio::SettingsBindFlags::DEFAULT)
                .build();
            settings
                .bind_sort_order(&*slf, "sort-order")
                .flags(gio::SettingsBindFlags::DEFAULT)
                .build();

            self.results_list
                .bind_model(Some(&self.shows_model()), |show| {
                    glib::Object::builder::<TvMediathekCard>()
                        .property("show", show)
                        .build()
                        .into()
                });

            self.results_list.connect_row_activated(|_, row| {
                let row = row
                    .downcast_ref::<TvMediathekCard>()
                    .expect("invalid ListBoxRow type");
                row.set_expanded(!row.expanded())
            });

            fn load(slf: &super::TvMediathekView) {
                spawn_clone!(slf => slf.load())
            }

            slf.connect_query_string_notify(load);
            slf.connect_search_in_topic_notify(load);
            slf.connect_search_in_title_notify(load);
            slf.connect_search_in_description_notify(load);
            slf.connect_search_in_channel_notify(load);
            slf.connect_sort_by_notify(load);
            slf.connect_sort_order_notify(load);

            self.shows_model().connect_items_changed(
                glib::clone!(@weak self as slf => move |results, _, _, _| {
                    if results.n_items() == 0 {
                        slf.stack.set_visible_child(&*slf.nothing_found_view);
                    } else {
                        slf.stack.set_visible_child(&*slf.results_view);
                    }
                }),
            );

            slf.connect_map(|slf| {
                slf.imp().search_entry.grab_focus();
            });
        }
    }
    impl WidgetImpl for TvMediathekView {}
    impl BinImpl for TvMediathekView {}
}

glib::wrapper! {
    pub struct TvMediathekView(ObjectSubclass<imp::TvMediathekView>)
        @extends gtk::Widget, adw::Bin;
}

impl TvMediathekView {
    fn client(&self) -> Arc<Mediathek> {
        self.imp()
            .client
            .get_or_init(|| {
                Arc::new(
                    Mediathek::new(
                        format!("{APP_ID}/{VERSION} ({PROJECT_URL})")
                            .try_into()
                            .expect("invalid user agent"),
                    )
                    .expect("failed to initialize HTTP client"),
                )
            })
            .clone()
    }
    fn query_props(&self) -> QueryProperties {
        let query_string = self.query_string();

        #[rustfmt::skip]
        let fields: Vec<QueryField> =
            self.search_in_topic().then_some(QueryField::Topic).into_iter()
            .chain(self.search_in_title().then_some(QueryField::Title))
            .chain(self.search_in_description().then_some(QueryField::Description))
            .chain(self.search_in_channel().then_some(QueryField::Channel))
            .collect();
        let include_future = self.include_future();
        let sort_by = match &*self.sort_by() {
            "channel" => SortField::Channel,
            "date" | "timestamp" => SortField::Timestamp,
            "duration" => SortField::Duration,
            _ => SortField::Timestamp,
        };
        let sort_order = match &*self.sort_order() {
            "asc" | "ascending" => SortOrder::Ascending,
            "desc" | "descending" => SortOrder::Descending,
            _ => SortOrder::Descending,
        };

        QueryProperties {
            query_string,
            fields,
            include_future,
            sort_by,
            sort_order,
        }
    }
    async fn delay_request_should_continue(&self) -> bool {
        let request_id = self.imp().request_id.get() + 1;
        self.imp().request_id.set(request_id);

        tokio(async { tokio::time::sleep(Duration::from_millis(500)).await }).await;

        self.imp().request_id.get() == request_id
    }
    async fn load(&self) {
        if !self.delay_request_should_continue().await {
            return;
        }

        let QueryProperties {
            query_string,
            fields,
            include_future,
            sort_by,
            sort_order,
        } = self.query_props();

        if query_string.is_empty() {
            return;
        }

        self.set_offset(0);

        let client = self.client();

        let slf = self.clone();

        match tokio(async move {
            client
                .query(fields, query_string)
                .include_future(include_future)
                .size(15)
                .sort_by(sort_by)
                .sort_order(sort_order)
                .send()
                .await
                .wrap_err_with(|| gettext("Failed to query the MediathekViewWeb API"))
        })
        .await
        {
            Ok(result) => {
                let shows_model = slf.imp().shows_model();
                shows_model.remove_all();

                for item in result.results {
                    shows_model.append(&ShowObject::new(item))
                }
            }
            Err(e) => show_error(e),
        }
    }
    async fn load_more(&self) {
        if !self.delay_request_should_continue().await {
            return;
        }

        let QueryProperties {
            query_string,
            fields,
            include_future,
            sort_by,
            sort_order,
        } = self.query_props();

        if query_string.is_empty() {
            return;
        }

        let offset = self.offset() + 15;
        self.set_offset(offset);

        let client = self.client();

        let slf = self.clone();

        match tokio(async move {
            client
                .query(fields, query_string)
                .include_future(include_future)
                .size(15)
                .offset(offset as usize)
                .sort_by(sort_by)
                .sort_order(sort_order)
                .send()
                .await
                .wrap_err_with(|| gettext("Failed to query the MediathekViewWeb API"))
        })
        .await
        {
            Ok(result) => {
                let shows_model = slf.imp().shows_model();

                for item in result.results {
                    shows_model.append(&ShowObject::new(item))
                }
            }
            Err(e) => show_error(e),
        }
    }
    pub fn reload(&self) {
        let slf = self.clone();
        spawn(async move { slf.load().await });
    }
}

#[derive(Debug)]
struct QueryProperties {
    query_string: String,
    fields: Vec<QueryField>,
    include_future: bool,
    sort_by: SortField,
    sort_order: SortOrder,
}
