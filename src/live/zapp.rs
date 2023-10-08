use indexmap::IndexMap;
use serde::Deserialize;
use time::OffsetDateTime;

use crate::config::{APP_ID, PROJECT_URL, VERSION};

const ZAPP_BACKEND_URL: &str = "https://api.zapp.mediathekview.de";

#[derive(Debug)]
pub struct Zapp {
    http: reqwest::Client,
}
impl Zapp {
    pub fn new() -> eyre::Result<Self> {
        Ok(Self {
            http: reqwest::Client::builder()
                .user_agent(format!("{APP_ID}/{VERSION} ({PROJECT_URL})"))
                .build()?,
        })
    }
}
impl Zapp {
    pub async fn channel_info_list(&self) -> eyre::Result<ChannelInfoList> {
        Ok(self
            .http
            .get(format!("{ZAPP_BACKEND_URL}/v1/channelInfoList"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
    pub async fn shows(&self, channel_id: &ChannelId) -> eyre::Result<ShowsResult> {
        Ok(self
            .http
            .get(format!("{ZAPP_BACKEND_URL}/v1/shows/{channel_id}"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}

pub type ChannelInfoList = IndexMap<ChannelId, ChannelInfo>;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize)]
pub struct ChannelId(String);
impl AsRef<str> for ChannelId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Display for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelInfo {
    pub name: String,
    pub stream_url: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShowsResult {
    Shows(Vec<Show>),
    Error(String),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Show {
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub channel: ChannelId,
    #[serde(with = "time::serde::rfc3339")]
    pub start_time: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub end_time: OffsetDateTime,
}
