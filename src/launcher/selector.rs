// Copyright 2023 David Cabot
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    borrow::Cow,
    cell::{Cell, OnceCell, RefCell},
    future::Future,
    rc::Rc,
    task::Waker,
};

use adw::{glib, gtk, prelude::*, subclass::prelude::*};

use gettextrs::gettext;

use crate::{
    application::TvApplication,
    utils::{show_error, spawn_clone},
};

use super::{ExternalProgram, ExternalProgramType};

#[derive(Debug, Default, PartialEq)]
enum ProgramSelection {
    #[default]
    Pending,
    Selected(ExternalProgram),
    Canceled,
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(file = "src/launcher/selector.blp")]
    #[properties(wrapper_type = super::ProgramSelector)]
    pub struct ProgramSelector {
        #[property(get, set)]
        description: RefCell<String>,
        #[property(get, set)]
        selected_program: RefCell<String>,
        #[template_child]
        confirm_button: TemplateChild<gtk::Button>,
        #[template_child]
        program_list: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        custom_program_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        custom_program_validation_icon: TemplateChild<gtk::Image>,
        all_programs: RefCell<Vec<ExternalProgram>>,
        pub(super) program_type: OnceCell<ExternalProgramType>,
        pub(super) program: RefCell<Option<ExternalProgram>>,
        pub(super) future_data: Rc<RefCell<(ProgramSelection, Option<Waker>)>>,
        loaded: Cell<bool>,
    }

    #[gtk::template_callbacks]
    impl ProgramSelector {
        #[template_callback]
        async fn start_custom_program_input(&self, #[rest] _: &[glib::Value]) {
            self.obj().set_selected_program("");
        }
        #[template_callback]
        async fn apply_custom_program(&self, entry: adw::EntryRow) {
            self.obj().set_selected_program(&*entry.text());
        }
    }

    impl ProgramSelector {
        async fn set_program(&self, id: Cow<'static, str>) {
            if id.is_empty() {
                *self.program.borrow_mut() = None;
                self.confirm_button.set_sensitive(false);
                self.custom_program_entry.set_css_classes(&[]);
                self.custom_program_validation_icon.set_visible(false);
                return;
            }

            if let Some(program) = self
                .all_programs
                .borrow()
                .iter()
                .find(|program| program.id == id)
            {
                *self.program.borrow_mut() = Some(program.clone());
                self.confirm_button.set_sensitive(true);
                self.custom_program_entry.set_text("");
                self.custom_program_entry.set_css_classes(&[]);
                self.custom_program_validation_icon.set_visible(false);
                return;
            }

            if self.custom_program_entry.text().as_str() != id {
                self.custom_program_entry.set_text(&id);
            }

            match ExternalProgram::find("", id.clone()).await {
                Ok(Some(program)) => {
                    *self.program.borrow_mut() = Some(program);
                    self.confirm_button.set_sensitive(true);
                    self.custom_program_entry.set_css_classes(&["success"]);
                    self.custom_program_validation_icon
                        .set_icon_name(Some("test-pass-symbolic"));
                    self.custom_program_validation_icon.set_tooltip_text(None);
                    self.custom_program_validation_icon.set_visible(true);
                }
                Ok(None) => {
                    *self.program.borrow_mut() = None;
                    self.confirm_button.set_sensitive(false);
                    self.custom_program_entry.set_css_classes(&["error"]);
                    self.custom_program_validation_icon
                        .set_icon_name(Some("error-symbolic"));
                    self.custom_program_validation_icon.set_tooltip_text(Some(
                        // translators: `{}` is replaced by the application ID, e.g. `org.example.Application`
                        &gettext("Could not find application “{}”").replace("{}", &id),
                    ));
                    self.custom_program_validation_icon.set_visible(true);
                }
                Err(e) => show_error(e),
            }
        }
        pub(super) async fn load(&self) {
            match ExternalProgram::find_known(
                *self
                    .program_type
                    .get()
                    .expect("program_type was not initialized"),
            )
            .await
            {
                Ok(programs) => {
                    self.program_list.remove(&*self.custom_program_entry);
                    for program in &programs {
                        let btn = gtk::CheckButton::builder()
                            .action_name("program-selector.select-program")
                            .action_target(&program.id.to_variant())
                            .build();

                        btn.connect_activate({
                            let id = program.id.clone();

                            glib::clone!(@weak self as slf => move |_| {
                                spawn_clone!(slf, id => async {
                                    slf.set_program(id).await
                                })
                            })
                        });

                        let row = adw::ActionRow::builder()
                            .title(&*program.name)
                            .subtitle(&*program.id)
                            .activatable_widget(&btn)
                            .build();
                        row.add_prefix(&btn);

                        self.program_list.add(&row);
                    }

                    self.program_list.add(&*self.custom_program_entry);

                    self.obj().set_visible(true);
                    *self.all_programs.borrow_mut() = programs;
                    self.loaded.set(true);

                    let selected_program = self.selected_program.borrow().clone();
                    self.set_program(selected_program.into()).await;
                }
                Err(e) => {
                    let msg = gettext("Failed to load external applications");
                    self.program_list.add(
                        &adw::ActionRow::builder()
                            .title(&msg)
                            .subtitle(gettext("See the terminal output for details."))
                            .build(),
                    );
                    show_error(e.wrap_err(msg));
                }
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProgramSelector {
        const NAME: &'static str = "ProgramSelector";
        type Type = super::ProgramSelector;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_property_action("program-selector.select-program", "selected-program");
            klass.install_action("program-selector.cancel", None, |slf, _, _| {
                *slf.imp().program.borrow_mut() = None;

                slf.close()
            });
            klass.install_action("program-selector.confirm", None, |slf, _, _| {
                let Some(program) = slf.imp().program.take() else {
                    return;
                };

                {
                    let (selection, waker) = &mut *slf.imp().future_data.borrow_mut();
                    *selection = ProgramSelection::Selected(program);

                    if let Some(waker) = waker.take() {
                        waker.wake()
                    }
                }

                slf.close()
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProgramSelector {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_selected_program_notify(|slf| {
                if slf.imp().loaded.get() {
                    spawn_clone!(slf => slf.imp().set_program(slf.selected_program().into()))
                }
            });

            self.obj().connect_close_request(|slf| {
                let (selection, waker) = &mut *slf.imp().future_data.borrow_mut();
                if *selection == ProgramSelection::Pending {
                    *selection = ProgramSelection::Canceled;

                    if let Some(waker) = waker.take() {
                        waker.wake();
                    }
                }

                glib::Propagation::Proceed
            });
        }
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }
        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }
        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
    impl WidgetImpl for ProgramSelector {}
    impl WindowImpl for ProgramSelector {}
}

glib::wrapper! {
    pub struct ProgramSelector(ObjectSubclass<imp::ProgramSelector>)
        @extends gtk::Widget, gtk::Window;
}

impl ProgramSelector {
    pub async fn select_program(
        program_type: ExternalProgramType,
        initial_id: String,
    ) -> Option<ExternalProgram> {
        let application = TvApplication::get();
        let parent = application.active_window()?;

        let slf = glib::Object::builder::<Self>()
            .property("modal", true)
            .property("application", application)
            .property("transient-for", parent)
            .property("selected-program", initial_id)
            .build();

        let (title, description) = match program_type {
            ExternalProgramType::Player => (
                gettext("Select video player"),
                gettext("Select one of the following external programs to stream content."),
            ),
            ExternalProgramType::Downloader => (
                gettext("Select video downloader"),
                gettext("Select one of the following external programs to download content."),
            ),
        };
        slf.set_title(Some(&title));
        slf.set_description(format!(
            "{description}\n<small>{}</small>",
            gettext(
                "You can also specify a custom application ID (e.g. org.example.Application) of a different program that supports DBus activation, is able to open a https:// URI and is accessible from the context of this application.",
            )
        ));

        slf.imp()
            .program_type
            .set(program_type)
            .expect("already initialized");
        slf.imp().load().await;

        ProgramSelectFuture(slf.imp().future_data.clone()).await
    }
}

struct ProgramSelectFuture(Rc<RefCell<(ProgramSelection, Option<Waker>)>>);
impl Future for ProgramSelectFuture {
    type Output = Option<ExternalProgram>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let (selection, waker) = &mut *self.0.borrow_mut();

        match selection {
            ProgramSelection::Pending => {
                *waker = Some(cx.waker().clone());
                std::task::Poll::Pending
            }
            ProgramSelection::Selected(program) => std::task::Poll::Ready(Some(program.clone())),
            ProgramSelection::Canceled => std::task::Poll::Ready(None),
        }
    }
}
