// Copyright 2023 David Cabot
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::new_without_default)]

mod application;
mod config;
mod live;
mod mediathek;
mod settings;
mod utils;
mod window;

use self::{
    application::MdkApplication,
    config::{LOCALEDIR, PROJECT_NAME},
};

use adw::{gio, glib, prelude::*};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};

static GRESOURCE_BYTES: &[u8] =
    gvdb_macros::include_gresource_from_dir!("/de/k_bo/mediathek", "data/resources");

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

    MdkApplication::new().run()
}
