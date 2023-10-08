use std::cell::OnceCell;

use adw::{gio, glib};
use gsettings_macro::gen_settings;

#[gen_settings(file = "data/de.k_bo.Mediathek.gschema.xml", id = "de.k_bo.Mediathek")]
pub struct MdkSettings;

impl MdkSettings {
    pub fn get() -> Self {
        thread_local! {
            static SETTINGS: OnceCell<MdkSettings> = OnceCell::new();
        }
        SETTINGS.with(|settings| settings.get_or_init(MdkSettings::new).clone())
    }
}
