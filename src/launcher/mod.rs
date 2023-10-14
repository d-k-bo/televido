use crate::{application::MdkApplication, utils::tokio};

use self::application_proxy::ApplicationProxy;
pub use self::selector::ProgramSelector;

mod application_proxy;
mod selector;

pub static PLAYERS: &[ExternalProgram] = &[
    ExternalProgram {
        name: "Videos",
        id: "org.gnome.Totem",
    },
    ExternalProgram {
        name: "Celluloid",
        id: "io.github.celluloid_player.Celluloid",
    },
    ExternalProgram {
        name: "Clapper",
        id: "com.github.rafostar.Clapper",
    },
    // not dbus-activatable
    // ExternalProgram { name: "µExternalProgram", id: "org.sigxcpu.Livi"},
    // ExternalProgram { name: "Glide", id: "net.baseart.Glide"},
    // ExternalProgram { name: "Daikhan", id: "io.gitlab.daikhan.stable"},
    // doesn't implement org.freedesktop.Application
    // ExternalProgram { name: "VLC", id: "org.videolan.VLC"},
    // ExternalProgram { name: "mpv", id: "io.mpv.Mpv"},
    // ExternalProgram { name: "Haruna Media ExternalProgram", id: "org.kde.haruna"},
];

pub static DOWNLOADERS: &[ExternalProgram] = &[ExternalProgram {
    name: "Parabolic",
    id: "org.nickvision.tubeconverter",
}];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExternalProgram {
    pub name: &'static str,
    pub id: &'static str,
}
impl ExternalProgram {
    pub async fn play(self, uri: impl Into<String>) -> eyre::Result<()> {
        let conn = MdkApplication::dbus().await;
        let uri = uri.into();

        tokio(async move {
            let proxy =
                ApplicationProxy::new(&conn, self.id, format!("/{}", self.id.replace('.', "/")))
                    .await?;

            proxy.open(&[&uri], Default::default()).await?;

            Ok(())
        })
        .await
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExternalProgramType {
    Player,
    Downloader,
}

impl ExternalProgramType {
    pub fn all(self) -> &'static [ExternalProgram] {
        match self {
            ExternalProgramType::Player => PLAYERS,
            ExternalProgramType::Downloader => DOWNLOADERS,
        }
    }
    pub fn find(self, id: &str) -> Option<ExternalProgram> {
        self.all().iter().find(|program| program.id == id).copied()
    }
    pub async fn list(self) -> eyre::Result<Vec<ExternalProgram>> {
        let conn = MdkApplication::dbus().await;

        tokio(async move {
            let all = self.all();
            let dbus_proxy = zbus::fdo::DBusProxy::new(&conn).await?;

            let programs = dbus_proxy
                .list_activatable_names()
                .await?
                .into_iter()
                .filter_map(|bus_name| {
                    all.iter()
                        .find(|program| program.id == bus_name.as_str())
                        .copied()
                })
                .collect();

            Ok(programs)
        })
        .await
    }
}
