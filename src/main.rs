use types::commit::Commit;
use types::releases::LatestRelease;
mod types {
    pub mod commit;
    pub mod releases;
}

use chrono;
use clap::Parser;
use exitfailure::ExitFailure;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT},
    Url,
};
use serde::Deserialize;
use std::collections::{hash_map::Entry, HashMap};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

/// CLI to generate release docs for a CHANGELOG
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Org name for the given repository
    #[arg(short, long)]
    org: String,
    /// Name of the repository
    #[arg(short, long)]
    repo: String,
    /// File path to the CHANGELOG
    #[arg(short, long)]
    file_path: String,
    /// Target version for the release. Format: vXX.XX.XX
    #[arg(short, long)]
    target_version: String,
    /// Sha or branch to start commits at. Defaults to 'main'.
    #[arg(short, long, default_value = "main")]
    sha: String,
}

struct CommitMessageParts {
    pr_num: String,
    message: String,
}

#[derive(Debug)]
struct SanitizedInfo {
    commits: Vec<Commit>,
}

impl SanitizedInfo {
    fn create_changelog_contents(&mut self, args: &Args, tag_name: &String) -> String {
        self.extract_commits();
        let mut h: HashMap<String, Vec<String>> = HashMap::new();
        for c in self.commits.iter() {
            let prefix = self.extract_prefix(&c);
            let m_parts = self.extract_message_parts(&c);
            let sanitized_commit = self.create_commit(&args, &m_parts, &c);

            match h.entry(prefix) {
                Entry::Vacant(e) => {
                    e.insert(vec![sanitized_commit]);
                }
                Entry::Occupied(mut e) => e.get_mut().push(sanitized_commit),
            }
        }

        let header = self.create_release_header(tag_name, &args);
        let mut body = String::from("");
        let keys = h.keys().collect::<Vec<&String>>();
        for k in keys {
            let commit_header = "### ".to_owned() + &capitalize(k) + "\n\n";
            let v = h.get(k).unwrap();
            let joined_commits = v.join("") + "\n";
            body = body + &commit_header + &joined_commits;
        }

        self.merge_changelog_contents(&header, &body)
    }

    fn merge_changelog_contents(&self, header: &String, body: &String) -> String {
        format!("{}\n{}", &header, &body)
    }

    fn extract_commits(&mut self) {
        let mut v: Vec<Commit> = Vec::new();

        for commit in &self.commits {
            // Once we reach the latest release commit we don't need to include it
            if commit.commit.message.starts_with("chore(release)") {
                break;
            }
            v.push(commit.clone());
        }

        self.commits = v
    }

    fn extract_prefix(&self, c: &Commit) -> String {
        c.commit.message.split(":").collect::<Vec<_>>()[0]
            .split("(")
            .collect::<Vec<_>>()[0]
            .to_owned()
    }

    fn extract_message_parts(&self, c: &Commit) -> CommitMessageParts {
        let m = c.commit.message.lines().collect::<Vec<_>>()[0]
            .split(" ")
            .collect::<Vec<_>>();

        CommitMessageParts {
            message: m[0..m.len() - 1].join(" "),
            pr_num: m.last().clone().unwrap().to_string(),
        }
    }

    fn create_release_header(&self, tag_name: &String, args: &Args) -> String {
        let date = chrono::offset::Local::now().to_string();
        let parsed_tag = &args.target_version[1..args.target_version.len()];
        let header = format!(
            "\n## [{}](https://github.com/{}/{}/compare/{}..{})({})\n",
            parsed_tag,
            &args.org,
            &args.repo,
            &tag_name,
            &args.target_version,
            &date[0..10]
        );

        header
    }

    fn create_pr_link(&self, args: &Args, pr_num: &String) -> String {
        format!(
            "https://github.com/{}/{}/pull/{}",
            &args.org,
            &args.repo,
            &pr_num[2..pr_num.len() - 1]
        )
    }

    fn create_sha_link(&self, args: &Args, sha: &String) -> String {
        format!(
            "https://github.com/{}/{}/commit/{}",
            &args.org, &args.repo, sha
        )
    }

    fn create_commit(&self, args: &Args, m_parts: &CommitMessageParts, c: &Commit) -> String {
        let pr_link = self.create_pr_link(&args, &m_parts.pr_num);
        let pr_sha_link = self.create_sha_link(&args, &c.sha);

        format!(
            "- {} ([#{}]({})) ([{}]({}))\n",
            m_parts.message,
            &m_parts.pr_num[2..m_parts.pr_num.len() - 1],
            pr_link,
            &c.sha[0..7],
            pr_sha_link
        )
    }
}

impl LatestRelease {
    async fn get(org: &String, repo: &String) -> Result<LatestRelease, ExitFailure> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            org, repo
        );
        http_get::<LatestRelease>(&url).await
    }
}

trait GithubCommits {
    async fn get(org: &String, repo: &String, sha: &String) -> Result<Vec<Commit>, ExitFailure>;
}

impl GithubCommits for Vec<Commit> {
    async fn get(org: &String, repo: &String, sha: &String) -> Result<Vec<Commit>, ExitFailure> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/commits?sha={}",
            org, repo, sha
        );
        http_get::<Vec<Commit>>(&url).await
    }
}

async fn http_get<T: for<'a> Deserialize<'a>>(url: &String) -> Result<T, ExitFailure> {
    let parsed_url = Url::parse(&*url)?;

    let client = reqwest::Client::new();
    let res = client
        .get(parsed_url)
        .headers(construct_headers())
        .send()
        .await?;

    let result = match res.status() {
        reqwest::StatusCode::OK => {
            let json = match res.json::<T>().await {
                Ok(parsed) => parsed,
                Err(err) => panic!("The response did not match the shape we expected: {}", err),
            };

            json
        }
        code => {
            panic!("Failed with a status code: {:?}", code);
        }
    };
    Ok(result)
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    headers
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn splice_data_into_file(path: &str, splice_at: u64, data: &[u8]) -> Result<(), String> {
    let mut file = File::options().read(true).write(true).open(path).unwrap();

    let seek = SeekFrom::Start(splice_at);
    file.seek(seek).unwrap();

    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    file.seek(seek).unwrap();

    if file.write_all(data).is_err() || file.write_all(&buf).is_err() {
        return Err("Error writing data to file".to_string());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    let args = Args::parse();
    let res_commits = <Vec<Commit>>::get(&args.org, &args.repo, &args.sha).await;
    let release_info = LatestRelease::get(&args.org, &args.repo).await;
    let mut info = SanitizedInfo {
        commits: res_commits.unwrap(),
    };
    let changelog_contents =
        SanitizedInfo::create_changelog_contents(&mut info, &args, &release_info.unwrap().tag_name);

    // The splice_at is hardcoded to 12 since it's after the initial header `# Changelog`.
    let _ = splice_data_into_file(&args.file_path, 12, changelog_contents.as_bytes());
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("this is a test"), "This is a test");
    }
}
