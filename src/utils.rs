// Copyright 2023 David Cabot
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    cell::{OnceCell, RefCell},
    fmt::Debug,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::OnceLock,
    task::{Poll, Waker},
    time::Duration,
};

use adw::{gio, glib, gtk, prelude::*};
use gettextrs::gettext;

use crate::{application::TvApplication, window::TvWindow};

pub use self::async_resource::AsyncResource;

pub async fn tokio<Fut, T>(fut: Fut) -> T
where
    Fut: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

    RUNTIME
        .get_or_init(|| tokio::runtime::Runtime::new().unwrap())
        .spawn(fut)
        .await
        .expect("tokio thread panicked")
}

pub fn spawn<Fut>(fut: Fut)
where
    Fut: Future<Output = ()> + 'static,
{
    gtk::glib::MainContext::default().spawn_local(fut);
}

macro_rules! spawn_clone {
    ($( $val:ident ),* => $future:expr) => {{
        $(
            let $val = $val.clone();
        )*
        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move { $future.await; });
    }};
}
pub(crate) use spawn_clone;

pub fn format_timestamp_full(t: i64) -> Option<glib::GString> {
    // translators:  %b is Month name (short)
    //				 %-e is the Day number
    //				 %Y is the year (with century)
    //				 %H is the hours (24h format)
    //				 %M is the minutes
    let time_format = gettext("%b %-e, %Y %H:%M");

    glib::DateTime::from_unix_local(t)
        .and_then(|datetime| datetime.format(&time_format))
        .ok()
}

pub fn format_timestamp_time(t: i64) -> Option<glib::GString> {
    // translators:  %H is the hours (24h format)
    //				 %M is the minutes
    let time_format = gettext("%H:%M");

    glib::DateTime::from_unix_local(t)
        .and_then(|datetime| datetime.format(&time_format))
        .ok()
}

pub fn format_duration(duration: &Duration) -> String {
    let mut s = duration.as_secs();

    let h = s / 3_600;
    s %= 3_600;
    let m = s / 60;
    s %= 60;

    if h > 0 {
        format!("{h:2}h{m:02}m{s:02}s")
    } else {
        format!("{m:02}m{s:02}s")
    }
}

pub fn show_error(e: eyre::Report) {
    if let Some(window) = TvApplication::get()
        .active_window()
        .and_downcast::<TvWindow>()
    {
        window.add_toast(
            adw::Toast::builder()
                .title(format!(
                    "{e}. {}",
                    gettext("See the terminal output for details.")
                ))
                .build(),
        );
    }
    tracing::error!("{e:?}");
}

pub trait ListStoreExtManual {
    fn typed_insert_sorted<T: IsA<glib::Object>>(
        &self,
        item: &T,
        compare_func: impl FnMut(&T, &T) -> std::cmp::Ordering,
    ) -> u32;
    fn typed_sort<T: IsA<glib::Object>>(
        &self,
        compare_func: impl FnMut(&T, &T) -> std::cmp::Ordering,
    );
}

impl ListStoreExtManual for gio::ListStore {
    fn typed_insert_sorted<T: IsA<glib::Object>>(
        &self,
        item: &T,
        mut compare_func: impl FnMut(&T, &T) -> std::cmp::Ordering,
    ) -> u32 {
        self.insert_sorted(item, |a, b| {
            compare_func(a.downcast_ref().unwrap(), b.downcast_ref().unwrap())
        })
    }
    fn typed_sort<T: IsA<glib::Object>>(
        &self,
        mut compare_func: impl FnMut(&T, &T) -> std::cmp::Ordering,
    ) {
        self.sort(|a, b| compare_func(a.downcast_ref().unwrap(), b.downcast_ref().unwrap()))
    }
}

mod async_resource {
    use super::*;

    #[derive(Clone, Default)]
    pub struct AsyncResource<T: Clone + Debug> {
        inner: Rc<AsyncResourceInner<T>>,
    }

    impl<T: Clone + Debug> std::fmt::Debug for AsyncResource<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(std::any::type_name::<Self>())
        }
    }

    #[derive(Default)]
    struct AsyncResourceInner<T: Clone + Debug> {
        data: RefCell<Option<T>>,
        load_fn: OnceCell<LoadFn<T>>,
        wakers: RefCell<Vec<Waker>>,
    }

    type LoadFn<T> = Rc<dyn Fn() -> Pin<Box<dyn Future<Output = T> + 'static>>>;

    impl<T: Clone + Debug> AsyncResource<T> {
        #[track_caller]
        pub fn set_load_fn(
            &self,
            load_fn: impl Fn() -> Pin<Box<dyn Future<Output = T> + 'static>> + 'static,
        ) {
            if self.inner.load_fn.set(Rc::new(load_fn)).is_err() {
                panic!("Resource load function has already been initialized")
            }
        }
    }

    impl<T: 'static + Clone + Debug> AsyncResource<T> {
        pub fn load(&self) {
            match self.inner.load_fn.get().cloned() {
                Some(load_fn) => {
                    let inner = self.inner.clone();
                    spawn(async move {
                        *inner.data.borrow_mut() = None;
                        let data = load_fn().await;
                        *inner.data.borrow_mut() = Some(data);
                        for waker in inner.wakers.borrow_mut().drain(..) {
                            waker.wake();
                        }
                    });
                }
                None => panic!("Resource load function has not been initialized"),
            }
        }
    }

    impl<T: Clone + Debug> Future for AsyncResource<T> {
        type Output = T;

        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            match &*self.inner.data.borrow() {
                Some(data) => Poll::Ready(data.clone()),
                None => {
                    self.inner.wakers.borrow_mut().push(cx.waker().clone());
                    Poll::Pending
                }
            }
        }
    }
}
