// Copyright 2023 David Cabot
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Cow;

use crate::{application::TvApplication, utils::tokio};

use self::application_proxy::ApplicationProxy;
pub use self::selector::ProgramSelector;

mod application_proxy;
mod selector;

pub static PLAYERS: &[ExternalProgram] = &[
    ExternalProgram::new("Videos", "org.gnome.Totem"),
    ExternalProgram::new("Celluloid", "io.github.celluloid_player.Celluloid"),
    ExternalProgram::new("Clapper", "com.github.rafostar.Clapper"),
    ExternalProgram::new("Daikhan", "io.gitlab.daikhan.stable"),
    // not dbus-activatable
    // ExternalProgram::new("µPlayer", "org.sigxcpu.Livi"),
    // ExternalProgram::new("Glide", "net.baseart.Glide"),
    // doesn't implement org.freedesktop.Application
    // ExternalProgram::new("VLC", "org.videolan.VLC"),
    // ExternalProgram::new("mpv", "io.mpv.Mpv"),
    // ExternalProgram::new("Haruna Media µPlayer", "org.kde.haruna"),
];

pub static DOWNLOADERS: &[ExternalProgram] = &[ExternalProgram::new(
    "Parabolic",
    "org.nickvision.tubeconverter",
)];

#[derive(Clone, Debug, PartialEq)]
pub struct ExternalProgram {
    pub name: Cow<'static, str>,
    pub id: Cow<'static, str>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExternalProgramType {
    Player,
    Downloader,
}

impl ExternalProgram {
    pub async fn open(self, uri: impl Into<String>) -> eyre::Result<()> {
        let conn = TvApplication::dbus().await;
        let uri = uri.into();

        tokio(async move {
            let proxy = ApplicationProxy::new(
                &conn,
                self.id.clone(),
                format!("/{}", self.id.replace('.', "/")),
            )
            .await?;

            proxy.open(&[&uri], Default::default()).await?;

            Ok(())
        })
        .await
    }
}

impl ExternalProgram {
    const fn new(name: &'static str, id: &'static str) -> Self {
        ExternalProgram {
            name: Cow::Borrowed(name),
            id: Cow::Borrowed(id),
        }
    }
    pub async fn find(
        name: impl Into<Cow<'static, str>>,
        id: impl Into<Cow<'static, str>>,
    ) -> eyre::Result<Option<Self>> {
        let conn = TvApplication::dbus().await;
        let name = name.into();
        let id = id.into();

        tokio(async move {
            let dbus_proxy = zbus::fdo::DBusProxy::new(&conn).await?;

            if dbus_proxy
                .list_activatable_names()
                .await?
                .into_iter()
                .any(|bus_name| bus_name == &*id)
            {
                Ok(Some(ExternalProgram { name, id }))
            } else {
                Ok(None)
            }
        })
        .await
    }

    pub async fn find_known(program_type: ExternalProgramType) -> eyre::Result<Vec<Self>> {
        let conn = TvApplication::dbus().await;

        tokio(async move {
            let known_programs = match program_type {
                ExternalProgramType::Player => PLAYERS,
                ExternalProgramType::Downloader => DOWNLOADERS,
            };
            let dbus_proxy = zbus::fdo::DBusProxy::new(&conn).await?;

            let programs = dbus_proxy
                .list_activatable_names()
                .await?
                .into_iter()
                .filter_map(|bus_name| {
                    known_programs
                        .iter()
                        .find(|program| program.id == bus_name.as_str())
                        .cloned()
                })
                .collect();

            Ok(programs)
        })
        .await
    }
}
