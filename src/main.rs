use core::panic;
use std::fmt::Write;
use std::path::PathBuf;

use anyhow::Result;
use reqwest::*;

#[derive(Debug)]
struct Course {
    id: u32,
    name: String,
}

#[derive(Debug)]
pub struct Config {
    token: String,
    canvas_api_url: String,
    courses: Vec<Course>,
    os: OS,
    current_dir: PathBuf,
}

#[derive(Debug)]
enum OS {
    Windows,
    MacOS,
}

impl OS {
    fn new() -> Self {
        match std::env::consts::OS {
            "windows" => OS::Windows,
            "macos" => OS::MacOS,
            _ => panic!("OS NOT SUPPORTED"),
        }
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        // load environment variables, especially the `TOKEN`
        dotenv::dotenv()?;

        let token = std::env::var("TOKEN")?;

        let canvas_url = std::env::var("CANVAS_API_URL")?;

        let courses = std::env::var("COURSE_IDS")
            .unwrap()
            .split(", ")
            .zip(std::env::var("COURSE_NAMES").unwrap().split(", "))
            .map(|(id, name)| Course {
                id: id.parse::<u32>().unwrap(),
                name: name.to_string(),
            })
            .collect();

        let os = OS::new();

        let current_dir = {
            let mut t = dirs::home_dir().unwrap();
            t.push("git");
            t.push("canvas-fuzzy-finder");
            t
        };

        Ok(Self {
            token,
            canvas_api_url: canvas_url,
            courses,
            current_dir,
            os,
        })
    }
}

#[derive(Debug)]
struct Runner {
    config: Config,
    client: Client,
    user_id: Option<u64>,
    recache_all: bool,
}

impl Runner {
    pub fn new(config: Config) -> Self {
        let client = Client::new();
        Self {
            config,
            client,
            user_id: None,
            recache_all: false,
        }
    }

    async fn set_user_id(&mut self) -> Result<()> {
        let res = self
            .client
            .get(format!("{}/api/v1/users/self", &self.config.canvas_api_url))
            .bearer_auth(&self.config.token)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        dbg!(&res);

        self.user_id = Some(res["id"].as_u64().unwrap());

        Ok(())
    }

    fn fuzzy_find(&self, str: &str) -> String {
        match self.config.os {
            OS::Windows => windows::fuzzy_finder(&self.config, str),
            OS::MacOS => {
                todo!()
            }
        }
    }

    fn open_link(&self, url: &str) {
        match self.config.os {
            OS::Windows => {
                windows::open_link(url);
            }
            OS::MacOS => {
                todo!()
            }
        }
    }

    // gets a list of all the titles, urls, and course names of all pages from
    // all modules for a user
    async fn get_modules(&self) -> Result<String> {
        let mut buf = String::new();

        // get all course ids/names in .env
        for course in &self.config.courses {
            // get module page of every course
            let module = self
                .client
                .get(format!(
                    "{}/api/v1/courses/{}/modules",
                    &self.config.canvas_api_url, course.id
                ))
                .bearer_auth(&self.config.token)
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            // for every item (or dropdown menu in modules)
            for items in module.as_array().unwrap() {
                // add each pages to our growing buffer
                let items_url = items["items_url"].as_str().unwrap();

                let pages = self
                    .client
                    .get(items_url)
                    .bearer_auth(&self.config.token)
                    .send()
                    .await?
                    .json::<serde_json::Value>()
                    .await?;

                for page in pages.as_array().unwrap() {
                    if page["title"].as_str().is_none() || page["html_url"].as_str().is_none() {
                        continue;
                    }

                    writeln!(
                        &mut buf,
                        "{} || {} || {}",
                        page["title"].as_str().unwrap(),
                        page["html_url"].as_str().unwrap(),
                        course.name
                    )?;
                }
            }
        }
        Ok(buf)
    }
}

mod windows {
    use std::process::Command;

    use crate::Config;

    pub fn fuzzy_finder(config: &Config, str: &str) -> String {
        // write buffer to current directory
        std::fs::write("buf", str).unwrap();

        // Open kitty with fzf in the external files directory
        Command::new("pwsh")
            .args([
                "-File",
                {
                    let mut t = config.current_dir.clone();
                    t.push("fzf-to-title-url-name.ps1");
                    t
                }
                .to_str()
                .unwrap(),
            ])
            .output()
            .unwrap();

        std::fs::read_to_string({
            let mut t = config.current_dir.clone();
            t.push("title-url-name.txt");
            t
        })
        .unwrap()
        .trim()
        .to_string()
    }
    pub fn open_link(url: &str) {
        Command::new("explorer").arg(url).output().unwrap();
    }
}

mod macos {
    // pub fn fuzzy_finder_file_name(config: &Config) -> PathBuf {
    //     // Open kitty with fzf in the external files directory
    //     Command::new("pwsh")
    //         .args([
    //             "-WorkingDirectory",
    //             config.external_files_directory.to_str().unwrap(),
    //             "-File",
    //             {
    //                 let mut t = config.bookshelf_directory.clone();
    //                 t.push("fzf-to-file.ps1");
    //                 t
    //             }
    //             .to_str()
    //             .unwrap(),
    //         ])
    //         .output()
    //         .unwrap();

    //     let mut file = config.external_files_directory.clone();

    //     file.push(
    //         std::fs::read_to_string({
    //             let mut t = config.bookshelf_directory.clone();
    //             t.push("author-title.txt");
    //             t
    //         })
    //         .unwrap()
    //         .trim(),
    //     );

    //     file
    // }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::new()?;

    let runner = Runner::new(config);
    let buf = runner.get_modules().await?;
    let title_url_name = runner.fuzzy_find(&buf);

    // get url from string
    let mut it = title_url_name.split(" || ");
    it.next();
    let url = it.next().unwrap();
    runner.open_link(url);

    Ok(())
}
