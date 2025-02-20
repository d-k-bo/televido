// SPDX-FileCopyrightText: David Cabot <d-k-bo@mailbox.org>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::new_without_default)]

mod application;
mod channel_icons;
mod config;
mod launcher;
mod live;
mod mediathek;
mod player;
mod preferences;
mod settings;
mod utils;
mod window;
mod zapp;

use self::{
    application::TvApplication,
    config::{LOCALEDIR, PROJECT_NAME},
};

use adw::{gio, glib, prelude::*};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};

static GRESOURCE_BYTES: &[u8] =
    gvdb_macros::include_gresource_from_dir!("/de/k_bo/televido", "data/resources");

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    bindtextdomain(PROJECT_NAME, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(PROJECT_NAME, "UTF-8").expect("Unable to set the text domain encoding");
    textdomain(PROJECT_NAME).expect("Unable to switch to the text domain");
    gio::resources_register(
        &gio::Resource::from_data(&glib::Bytes::from_static(GRESOURCE_BYTES)).unwrap(),
    );

    glib::setenv("CLAPPER_USE_PLAYBIN3", "1", false)
        .expect("failed to set CLAPPER_USE_PLAYBIN3 environment variable");
    clapper::init().expect("failed to initialize libclapper");

    TvApplication::new().run()
}
