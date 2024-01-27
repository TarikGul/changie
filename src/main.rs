mod types;

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

trait GithubCommits {
    async fn get(org: &String, repo: &String) -> Result<Vec<types::Commit>, ExitFailure>;
}

impl GithubCommits for Vec<types::Commit> {
    async fn get(org: &String, repo: &String) -> Result<Vec<types::Commit>, ExitFailure> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/commits?sha=main",
            org, repo
        );
        let url = Url::parse(&*url)?;

        let client = reqwest::Client::new();
        let res = client.get(url).headers(construct_headers()).send().await?;

        let result = match res.status() {
            reqwest::StatusCode::OK => {
                let json = match res.json::<Vec<types::Commit>>().await {
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

fn extract_commits(commits: Vec<types::Commit>) -> Vec<types::Commit> {
    let mut v: Vec<types::Commit> = Vec::new();

    for commit in commits {
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
    let res = <Vec<types::Commit>>::get(&args.org, &args.repo).await;

    let commits = res.unwrap();
    let parsed_commits = extract_commits(commits);

    println!("{:?}", parsed_commits);
    Ok(())
}
