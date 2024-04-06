// SPDX-FileCopyrightText: David Cabot <d-k-bo@mailbox.org>
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::{gdk, gdk::gdk_pixbuf, glib, prelude::*};
use eyre::WrapErr;
use phf::phf_map;
use tracing::error;

use crate::application::TvApplication;

pub fn load_channel_icon(channel_id: Option<&str>, image: &gtk::Image, size: i32) {
    let Some(icon_name) = channel_id.and_then(|id| ICON_NAMES.get(id)) else {
        image.set_icon_name(Some("image-missing-symbolic"));
        return;
    };

    let application = TvApplication::get();
    let style_manager = application.style_manager();
    let scale_factor = application.window().surface().unwrap().scale_factor();
    let size = scale_factor * size;

    set_icon(&style_manager, image, icon_name, size);

    style_manager.connect_dark_notify(glib::clone!(@weak image => move |style_manager| {
        set_icon(style_manager, &image, icon_name, size)
    }));

    fn set_icon(style_manager: &adw::StyleManager, image: &gtk::Image, icon_name: &str, size: i32) {
        match load_icon(
            icon_name,
            size,
            ColorScheme::for_style_manager(style_manager),
        ) {
            Ok(texture) => image.set_from_paintable(Some(&texture)),
            Err(e) => {
                error!("{e:?}");
                image.set_icon_name(Some("image-missing-symbolic"));
            }
        }
    }

    fn load_icon(
        icon_name: &str,
        size: i32,
        color_scheme: ColorScheme,
    ) -> eyre::Result<gdk::Texture> {
        let resource =
            format!("/de/k_bo/televido/icons/scalable/channels/{color_scheme}/{icon_name}");

        // load image manually with given size to avoid blurriness caused by scaling after rasterization
        gdk_pixbuf::Pixbuf::from_resource_at_scale(&resource, size, size, true)
            .map(|pixbuf| gdk::Texture::for_pixbuf(&pixbuf))
            .wrap_err_with(|| format!("failed to load channel logo from {resource}"))
    }
}

static ICON_NAMES: phf::Map<&'static str, &'static str> = phf_map! {
    // live
    "ard_alpha" => "ard-alpha.svg",
    "arte" => "arte.svg",
    "br_nord" => "br.svg",
    "br_sued" => "br.svg",
    "das_erste" => "das-erste.svg",
    "deutsche_welle" => "deutsche-welle.svg",
    "deutsche_welle_plus" => "deutsche-welle.svg",
    "dreisat" => "3sat.svg",
    "hr" => "hr.svg",
    "kika" => "kika.svg",
    "mdr_sachsen" => "mdr.svg",
    "mdr_sachsen_anhalt" => "mdr.svg",
    "mdr_thueringen" => "mdr.svg",
    "ndr_hh" => "ndr.svg",
    "ndr_mv" => "ndr.svg",
    "ndr_nds" => "ndr.svg",
    "ndr_sh" => "ndr.svg",
    "one" => "one.svg",
    "parlamentsfernsehen_1" => "parlamentsfernsehen.svg",
    "parlamentsfernsehen_2" => "parlamentsfernsehen.svg",
    "phoenix" => "phoenix.svg",
    "rb" => "rb.svg",
    "rbb_berlin" => "rbb.svg",
    "rbb_brandenburg" => "rbb.svg",
    "sr" => "sr.svg",
    "swr_bw" => "swr.svg",
    "swr_rp" => "swr.svg",
    "tagesschau24" => "tagesschau24.svg",
    "wdr" => "wdr.svg",
    "zdf" => "zdf.svg",
    "zdf_info" => "zdf-info.svg",
    "zdf_neo" => "zdf-neo.svg",
    // mediathek
    "3Sat" => "3sat.svg",
    "ARD" => "ard.svg",
    "ARTE.DE" => "arte.svg",
    "ARTE.EN" => "arte.svg",
    "ARTE.ES" => "arte.svg",
    "ARTE.FR" => "arte.svg",
    "ARTE.IT" => "arte.svg",
    "ARTE.PL" => "arte.svg",
    "BR" => "br.svg",
    "DW" => "deutsche-welle.svg",
    "Funk.net" => "funk.svg",
    "HR" => "hr.svg",
    "KiKA" => "kika.svg",
    "MDR" => "mdr.svg",
    "NDR" => "ndr.svg",
    "ORF" => "orf.svg",
    "PHOENIX" => "phoenix.svg",
    "Radio Bremen TV" => "rb.svg",
    "RBB" => "rbb.svg",
    "rbtv" => "rb.svg",
    "SR" => "sr.svg",
    "SRF" => "srf.svg",
    "SWR" => "swr.svg",
    "WDR" => "wdr.svg",
    "ZDF" => "zdf.svg",
    "ZDF-tivi" => "zdf.svg",
};

#[derive(Clone, Copy, Debug)]
enum ColorScheme {
    Light,
    Dark,
}

impl ColorScheme {
    fn for_style_manager(style_manager: &adw::StyleManager) -> Self {
        if style_manager.is_dark() {
            Self::Dark
        } else {
            Self::Light
        }
    }
}

impl std::fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Light => "light",
            Self::Dark => "dark",
        })
    }
}
