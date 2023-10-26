// Copyright (c) 2023 d-k-bo
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// SPDX-License-Identifier: MIT

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
