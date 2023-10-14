use std::{
    cell::{OnceCell, RefCell},
    future::Future,
    rc::Rc,
    task::Waker,
};

use adw::{glib, gtk, prelude::*, subclass::prelude::*};

use gettextrs::gettext;
use tracing::error;

use crate::application::TvApplication;

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
        all_programs: RefCell<Vec<ExternalProgram>>,
        pub(super) program_type: OnceCell<ExternalProgramType>,
        pub(super) program: RefCell<Option<ExternalProgram>>,
        pub(super) future_data: Rc<RefCell<(ProgramSelection, Option<Waker>)>>,
    }

    impl ProgramSelector {
        pub(super) async fn load(&self) {
            match self
                .program_type
                .get()
                .expect("program_type was not initialized")
                .list()
                .await
            {
                Ok(programs) => {
                    for program in &programs {
                        let btn = gtk::CheckButton::builder()
                            .action_name("program-selector.select-program")
                            .action_target(&program.id.to_variant())
                            .build();
                        let row = adw::ActionRow::builder()
                            .title(program.name)
                            .subtitle(program.id)
                            .activatable_widget(&btn)
                            .build();
                        row.add_prefix(&btn);
                        self.program_list.add(&row);
                    }

                    self.obj().set_visible(true);
                    *self.all_programs.borrow_mut() = programs;
                }
                Err(e) => {
                    error!("{e:?}");
                    self.program_list.add(
                        &adw::ActionRow::builder()
                            .title(gettext("Could not load external application."))
                            .subtitle(gettext("See the logs for details."))
                            .build(),
                    )
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
                let selected_program = slf.selected_program();
                let slf = slf.imp();
                match slf
                    .all_programs
                    .borrow()
                    .iter()
                    .find(|program| program.id == selected_program)
                {
                    Some(program) => {
                        *slf.program.borrow_mut() = Some(*program);
                        slf.confirm_button.set_sensitive(true);
                    }
                    None => {
                        error!(
                            "{:?}",
                            eyre::eyre!("invalid program id: {selected_program}")
                        )
                    }
                }
            });

            self.obj().connect_close_request(move |slf| {
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
    pub async fn select_program(program_type: ExternalProgramType) -> Option<ExternalProgram> {
        let application = TvApplication::get();
        let parent = application.active_window()?;

        let slf = glib::Object::builder::<Self>()
            .property("modal", true)
            .property("application", application)
            .property("transient-for", parent)
            .build();

        match program_type {
            ExternalProgramType::Player => {
                slf.set_title(Some(&gettext("Select video player")));
                slf.set_description(gettext(
                    "Select one of the following external programs to stream content.",
                ));
            }
            ExternalProgramType::Downloader => {
                slf.set_title(Some(&gettext("Select video download")));
                slf.set_description(gettext(
                    "Select one of the following external programs to download content.",
                ));
            }
        }

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

        match *selection {
            ProgramSelection::Pending => {
                *waker = Some(cx.waker().clone());
                std::task::Poll::Pending
            }
            ProgramSelection::Selected(program) => std::task::Poll::Ready(Some(program)),
            ProgramSelection::Canceled => std::task::Poll::Ready(None),
        }
    }
}
