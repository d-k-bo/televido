// SPDX-FileCopyrightText: David Cabot <d-k-bo@mailbox.org>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{cell::OnceCell, fmt::Display, str::FromStr};

use adw::{gio, glib, prelude::*};
use gsettings_macro::gen_settings;

use crate::{config::BASE_APP_ID, zapp::ChannelId};

#[gen_settings(file = "data/de.k_bo.Televido.gschema.xml")]
#[gen_settings_define(
    key_name = "live-channels",
    arg_type = "Vec<ChannelId>",
    ret_type = "Vec<ChannelId>"
)]
pub struct TvSettings;

impl TvSettings {
    pub fn get() -> Self {
        thread_local! {
            static SETTINGS: OnceCell<TvSettings> = const { OnceCell::new() };
        }
        SETTINGS.with(|settings| {
            settings
                .get_or_init(|| TvSettings::new(BASE_APP_ID))
                .clone()
        })
    }
}

impl Default for TvSettings {
    fn default() -> Self {
        Self::get()
    }
}

#[gen_settings(file = "data/de.k_bo.Televido.Player.gschema.xml")]
pub struct TvPlayerSettings;

impl TvPlayerSettings {
    pub fn get() -> Self {
        thread_local! {
            static SETTINGS: OnceCell<TvPlayerSettings> = const { OnceCell::new() };
        }
        SETTINGS.with(|settings| {
            settings
                .get_or_init(|| TvPlayerSettings::new("de.k_bo.Televido.Player"))
                .clone()
        })
    }
}

impl Default for TvPlayerSettings {
    fn default() -> Self {
        Self::get()
    }
}

impl StaticVariantType for ChannelId {
    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
        String::static_variant_type()
    }
}
impl FromVariant for ChannelId {
    fn from_variant(variant: &glib::Variant) -> Option<Self> {
        String::from_variant(variant).map(ChannelId::from)
    }
}
impl ToVariant for ChannelId {
    fn to_variant(&self) -> glib::Variant {
        AsRef::<str>::as_ref(self).to_variant()
    }
}
impl From<ChannelId> for glib::Variant {
    fn from(id: ChannelId) -> Self {
        id.to_variant()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VideoQuality {
    High,
    Medium,
    Low,
}
impl VideoQuality {
    pub fn default_playback() -> Self {
        TvSettings::get()
            .default_playback_quality()
            .parse()
            .unwrap()
    }
}
impl FromStr for VideoQuality {
    type Err = eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "high" => Ok(VideoQuality::High),
            "medium" => Ok(VideoQuality::Medium),
            "low" => Ok(VideoQuality::Low),
            _ => Err(eyre::eyre!("invalid value for video quality: \"{s}\"")),
        }
    }
}
impl Display for VideoQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            VideoQuality::High => "high",
            VideoQuality::Medium => "medium",
            VideoQuality::Low => "low",
        })
    }
}
