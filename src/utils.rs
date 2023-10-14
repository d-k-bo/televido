use std::{future::Future, sync::OnceLock, time::Duration};

use adw::{
    glib,
    gtk::{self, gdk, gdk_pixbuf},
    prelude::*,
};
use eyre::WrapErr;
use gettextrs::gettext;
use tracing::error;

use crate::application::MdkApplication;

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
    // docs: https://gitlab.gnome.org/GNOME/glib/-/blob/main/glib/gdatetime.c?ref_type=heads#L3358-3479
    // translators:  %b is Month name (short)
    //				 %-e is the Day number
    //				 %Y is the year (with century)
    //				 %H is the hours (24h format)
    //				 %M is the minutes
    static TIME_FORMAT: &str = "%b %-e, %Y %H:%M";

    glib::DateTime::from_unix_local(t)
        .and_then(|datetime| datetime.format(&gettext(TIME_FORMAT)))
        .ok()
}

pub fn format_timestamp_time(t: i64) -> Option<glib::GString> {
    // docs: https://gitlab.gnome.org/GNOME/glib/-/blob/main/glib/gdatetime.c?ref_type=heads#L3358-3479
    // translators:  %H is the hours (24h format)
    //				 %M is the minutes
    static TIME_FORMAT: &str = "%H:%M";

    glib::DateTime::from_unix_local(t)
        .and_then(|datetime| datetime.format(&gettext(TIME_FORMAT)))
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

macro_rules! channel_mapping {
    (
        $( #[ $attrs:meta ] )*
        pub enum $Enum:ident {
            $(
                #[channel(name = $name:literal, icon = $icon:literal)]
                $Variant:ident,
            )+
        }
    ) => {
        $( #[ $attrs ] )*
        pub enum $Enum {
            $( $Variant, )+
        }
        impl $Enum {
            #[allow(dead_code)]
            pub fn all() -> &'static [Self] {
                static ALL: &[$Enum] = &[ $( $Enum::$Variant ),+ ];
                ALL
            }
            pub fn icon_name(&self) -> &'static str {
                match self {
                    $( $Enum::$Variant => $icon, )+
                }
            }
        }
        impl std::str::FromStr for $Enum {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $( $name => Ok($Enum::$Variant), )+
                    _ => Err(())
                }
            }
        }
        impl std::fmt::Display for $Enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(
                    match self {
                        $( $Enum::$Variant => $name, )+
                    }
                )
            }
        }
    };
}
pub(crate) use channel_mapping;

pub fn load_channel_icon(image: &gtk::Image, icon_name: Option<&'static str>) {
    let Some(icon_name) = icon_name else {
        image.set_icon_name(Some("image-missing-symbolic"));
        return;
    };

    let style_manager = MdkApplication::get().style_manager();

    set_icon(&style_manager, image, icon_name);

    style_manager.connect_dark_notify(glib::clone!(@weak image => move |style_manager| {
        set_icon(style_manager, &image, icon_name)
    }));

    fn set_icon(style_manager: &adw::StyleManager, image: &gtk::Image, icon_name: &str) {
        match load_icon(style_manager, icon_name) {
            Ok(texture) => image.set_from_paintable(Some(&texture)),
            Err(e) => {
                error!("{e:?}");
                image.set_icon_name(Some("image-missing-symbolic"));
            }
        }
    }

    fn load_icon(style_manager: &adw::StyleManager, icon_name: &str) -> eyre::Result<gdk::Texture> {
        let resource = if style_manager.is_dark() {
            format!("/de/k_bo/mediathek/icons/scalable/channels/dark/{icon_name}",)
        } else {
            format!("/de/k_bo/mediathek/icons/scalable/channels/light/{icon_name}",)
        };

        // load image manually with given size to avoid blurriness caused by scaling after rasterization
        gdk_pixbuf::Pixbuf::from_resource_at_scale(&resource, 64, 64, true)
            .map(|pixbuf| gdk::Texture::for_pixbuf(&pixbuf))
            .wrap_err_with(|| format!("failed to load channel logo from {resource}"))
    }
}
