use super::commit::Author;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Reactions {
    pub url: String,
    pub total_count: u16,
    #[serde(rename = "+1")]
    pub symbol1: u16,
    #[serde(rename = "-1")]
    pub symbol2: u16,
    pub laugh: u16,
    pub hooray: u16,
    pub confused: u16,
    pub heart: u16,
    pub rocket: u16,
    pub eyes: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Uploader {
    pub login: String,
    pub id: u64,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    pub r#type: String,
    pub site_admin: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Asset {
    pub url: String,
    pub id: String,
    pub node_id: String,
    pub name: String,
    pub label: Option<String>,
    pub uploader: Uploader,
    pub content_type: String,
    pub state: String,
    pub size: u64,
    pub download_counts: u32,
    pub created_at: String,
    pub updated_at: String,
    pub browser_download_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LatestRelease {
    pub url: String,
    pub assets_url: String,
    pub upload_url: String,
    pub html_url: String,
    pub id: u64,
    pub author: Author,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: String,
    pub assets: Vec<Asset>,
    pub tarball_url: String,
    pub zipball_url: String,
    pub body: String,
    pub reactions: Reactions,
    // pub mentions_count: u16,
}
