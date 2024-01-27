use types::commit::Commit;
use types::releases::LatestRelease;
mod types {
    pub mod commit;
    pub mod releases;
}

use clap::Parser;
use exitfailure::ExitFailure;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use reqwest::Url;

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

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    let args = Args::parse();
    let res_commits = <Vec<Commit>>::get(&args.org, &args.repo).await;
    let commits = res_commits.unwrap();
    let parsed_commits = extract_commits(commits);

    let release_info = LatestRelease::get(&args.org, &args.repo).await;

    println!("{:?}", parsed_commits);
    println!("{:?}", release_info);
    Ok(())
}
