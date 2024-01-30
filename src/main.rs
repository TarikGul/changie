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
use std::collections::{hash_map::Entry, HashMap};
use std::fs::{write, File};
use std::io::{BufRead, BufReader, Error, Write};
use std::path::Prefix;

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
    file_path: String,
    #[arg(short, long)]
    target_version: String,
}

struct CommitMessageParts {
    pr_num: String,
    message: String,
}

#[derive(Debug)]
struct SanitizedInfo {
    commits: Vec<Commit>,
    pub header: String,
    pub body: String,
}

impl SanitizedInfo {
    fn new(&mut self, args: Args, tag_name: &String) {
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

        let mut body = String::from("");
        let keys = h.keys().collect::<Vec<&String>>();
        for k in keys {
            let commit_header = "## ".to_owned() + &capitalize(k) + "\n\n";
            let v = h.get(k).unwrap();
            let joined_commits = v.join("\n") + "\n";
            body = body + &commit_header + &joined_commits;
        }

        self.body = body;
        self.header = self.create_release_header(tag_name, &args);
    }

    fn merge_changelog_contents(&self) -> String {
        format!("{}\n {}", &self.header, &self.body)
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
            "- {} ([#{}]({})) ([{}]({}))",
            m_parts.message,
            &m_parts.pr_num[2..m_parts.pr_num.len() - 1],
            pr_link,
            &c.sha[0..6],
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

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn write_to_file(path: &String, content: &String) -> Result<(), Error> {
    let input = File::open(path)?;
    let buffered = BufReader::new(input);

    for line in buffered.lines() {
        println!("{}", line?);
    }

    write(path, content)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    let args = Args::parse();
    let res_commits = <Vec<Commit>>::get(&args.org, &args.repo).await;
    let release_info = LatestRelease::get(&args.org, &args.repo).await;
    let mut info = SanitizedInfo {
        commits: res_commits.unwrap(),
        header: "".to_string(),
        body: "".to_string(),
    };
    SanitizedInfo::new(&mut info, args, &release_info.unwrap().tag_name);
    let changelog_contents = SanitizedInfo::merge_changelog_contents(&info);

    println!("{:?}", changelog_contents);
    // let _ = write_to_file(&args.file_path, &content);
    Ok(())
}
