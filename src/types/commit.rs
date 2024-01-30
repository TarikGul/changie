use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitInfo {
    author: CommitAuthor,
    committer: CommitCommiter,
    pub message: String,
    tree: CommitTree,
    url: String,
    comment_count: u32,
    verification: CommitVerification,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitAuthor {
    name: String,
    email: String,
    date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitCommiter {
    name: String,
    email: String,
    date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitTree {
    sha: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitVerification {
    verified: bool,
    reason: String,
    signature: String,
    payload: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Author {
    login: String,
    id: u64,
    node_id: String,
    avatar_url: String,
    gravatar_id: String,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
    r#type: String,
    site_admin: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Committer {
    login: String,
    id: u64,
    node_id: String,
    avatar_url: String,
    gravatar_id: String,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
    r#type: String,
    site_admin: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parents {
    sha: String,
    url: String,
    html_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Commit {
    pub sha: String,
    pub node_id: String,
    pub commit: CommitInfo,
    pub url: String,
    pub html_url: String,
    pub comments_url: String,
    pub author: Author,
    pub committer: Committer,
    pub parents: Vec<Parents>,
}
