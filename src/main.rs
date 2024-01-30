use types::commit::Commit;
use types::releases::LatestRelease;
mod types {
    pub mod commit;
    pub mod releases;
}

use chrono;
use clap::Parser;
use exitfailure::ExitFailure;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use reqwest::Url;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Org name for the given repository
    #[arg(short, long)]
    org: String,
    /// Name of the repository
    #[arg(short, long)]
    repo: String,
    /// Number of times to greet
    #[arg(short, long)]
    file: String,
    #[arg(short, long)]
    target_version: String,
}

impl LatestRelease {
    async fn get(org: &String, repo: &String) -> Result<LatestRelease, ExitFailure> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            org, repo
        );
        let url = Url::parse(&*url)?;

        let client = reqwest::Client::new();
        let res = client.get(url).headers(construct_headers()).send().await?;

        let result = match res.status() {
            reqwest::StatusCode::OK => {
                let json = match res.json::<LatestRelease>().await {
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
}

trait GithubCommits {
    async fn get(org: &String, repo: &String) -> Result<Vec<Commit>, ExitFailure>;
}

impl GithubCommits for Vec<Commit> {
    async fn get(org: &String, repo: &String) -> Result<Vec<Commit>, ExitFailure> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/commits?sha=main",
            org, repo
        );
        let url = Url::parse(&*url)?;

        let client = reqwest::Client::new();
        let res = client.get(url).headers(construct_headers()).send().await?;

        let result = match res.status() {
            reqwest::StatusCode::OK => {
                let json = match res.json::<Vec<Commit>>().await {
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
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    headers
}

fn extract_commits(commits: Vec<Commit>) -> Vec<Commit> {
    let mut v: Vec<Commit> = Vec::new();

    for commit in commits {
        // Once we reach the latest release commit we don't need to include it
        if commit.commit.message.starts_with("chore(release)") {
            break;
        }
        v.push(commit);
    }

    v
}

fn parse_commits(commits: Vec<Commit>, args: &Args) -> String {
    let mut h: HashMap<&str, Vec<String>> = HashMap::new();
    for c in commits.iter() {
        let prefix = c.commit.message.split(":").collect::<Vec<_>>()[0]
            .split("(")
            .collect::<Vec<_>>()[0];
        let v = c.commit.message.lines().collect::<Vec<_>>()[0]
            .split(" ")
            .collect::<Vec<_>>();
        let message = v[0..v.len() - 1].join("");
        let pr_num = v.last().copied().unwrap();
        let pr_link = format!(
            "https://github.com/{}/{}/pull/{}",
            &args.org,
            &args.repo,
            &pr_num[2..pr_num.len() - 1]
        );
        let pr_sha_link = format!(
            "https://github.com/{}/{}/commit/{}",
            &args.org, &args.repo, c.sha
        );
        let key = format!(
            "- {} ([#{}]({})) ([{}]({}))",
            message,
            &pr_num[2..pr_num.len() - 1],
            pr_link,
            &c.sha[0..6],
            pr_sha_link
        );

        match h.entry(prefix) {
            Entry::Vacant(e) => {
                e.insert(vec![key]);
            }
            Entry::Occupied(mut e) => e.get_mut().push(key),
        }
    }

    let mut body = String::from("");
    let keys = h.keys().collect::<Vec<&&str>>();
    for k in keys {
        let commit_header = "## ".to_owned() + &capitalize(k) + "\n\n";
        let v = h.get(k).unwrap();
        let joined_commits = v.join("\n");
        body = body + &commit_header + &joined_commits;
    }

    body
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn create_release_header(tag_name: &String, args: &Args) -> String {
    // ## [0.1.6](https://github.com/paritytech/asset-transfer-api/compare/v0.1.5..v0.1.6)(2024-01-22)

    let date = chrono::offset::Local::now().to_string();
    let parsed_tag = &tag_name[1..tag_name.len()];
    let header = format!(
        "## [{}](https://github.com/{}/{}/compare/{}..{})({})",
        parsed_tag,
        &args.org,
        &args.repo,
        &tag_name,
        &args.target_version,
        &date[0..10]
    );

    header
}

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    let args = Args::parse();
    let res_commits = <Vec<Commit>>::get(&args.org, &args.repo).await;
    let commits = res_commits.unwrap();
    let extracted_commits = extract_commits(commits);
    let release_info = LatestRelease::get(&args.org, &args.repo).await;

    let body = parse_commits(extracted_commits, &args);
    let header = create_release_header(&release_info.unwrap().tag_name, &args);

    Ok(())
}
