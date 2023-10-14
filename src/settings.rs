use std::{cell::OnceCell, fmt::Display, str::FromStr};

use adw::{gio, glib};
use gsettings_macro::gen_settings;

#[gen_settings(file = "data/de.k_bo.Televido.gschema.xml", id = "de.k_bo.Televido")]
pub struct TvSettings;

impl TvSettings {
    pub fn get() -> Self {
        thread_local! {
            static SETTINGS: OnceCell<TvSettings> = OnceCell::new();
        }
        SETTINGS.with(|settings| settings.get_or_init(TvSettings::new).clone())
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
    pub fn default_download() -> Self {
        TvSettings::get()
            .default_download_quality()
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
