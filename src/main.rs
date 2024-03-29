use core::panic;
use std::fmt::Write;
use std::fs::File;
use std::path::PathBuf;
use std::time::SystemTime;

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
    Linux
}

impl OS {
    fn new() -> Self {
        match std::env::consts::OS {
            "windows" => OS::Windows,
            "macos" => OS::MacOS,
            "linux" => OS::Linux,
            _ => panic!("OS NOT SUPPORTED"),
        }
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        let current_dir = {
            let mut t = dirs::home_dir().unwrap();
            t.push("git");
            t.push("canvas-fuzzy-finder");
            t
        };

        std::env::set_current_dir(&current_dir)?;

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
            OS::MacOS => macos::fuzzy_finder(&self.config, str),
            OS::Linux => linux::fuzzy_finder(&self.config, str),
        }
    }

    fn open_link(&self, url: &str) {
        match self.config.os {
            OS::Windows => {
                windows::open_link(url);
            }
            OS::MacOS => {
                macos::open_link(url);
            }
            OS::Linux => {
                linux::open_link(url);
            }
        }
    }

    // gets a list of all the titles, urls, and course names of all pages from
    // all modules for a user
    async fn get_modules(&self) -> Result<String> {
        let mut buf = String::new();

        if !self.recache_all {
            // check if buf file exists
            if let Ok(file) = File::open({
                let mut t = self.config.current_dir.clone();
                t.push("buf");
                t
            }) {
                let sys_time_now = SystemTime::now();
                let duration = sys_time_now.duration_since(file.metadata()?.modified()?)?;
                if duration < std::time::Duration::new(300, 0) {
                    return Ok(std::fs::read_to_string({
                        let mut t = self.config.current_dir.clone();
                        t.push("buf");
                        t
                    })?);
                }
            }
        }

        // get all course ids/names in .env
        for course in &self.config.courses {
            // get module page of every course
            let modules = self
                .client
                .get(format!(
                    "{}/api/v1/courses/{}/modules",
                    &self.config.canvas_api_url, course.id
                ))
                .bearer_auth(&self.config.token)
                .query(&[("include[]", "items"), ("per_page", "100")])
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            // for every item (or dropdown menu in modules)
            for module in modules.as_array().unwrap() {
                for page in module["items"].as_array().unwrap() {
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
        std::fs::write(
            {
                let mut t = config.current_dir.clone();
                t.push("buf");
                t
            },
            str,
        )
        .unwrap();

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
    use std::process::Command;

    use crate::Config;

    pub fn fuzzy_finder(config: &Config, str: &str) -> String {
        // write buffer to current directory
        std::fs::write("buf", str).unwrap();

        // Open kitty with fzf in the external files directory
        Command::new("kitty")
            .args([
                "sh",
                {
                    let mut t = config.current_dir.clone();
                    t.push("fzf-to-title-url-name.sh");
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
        Command::new("open").arg(url).output().unwrap();
    }
}

mod linux {
    use std::process::Command;

    use crate::Config;

    pub fn fuzzy_finder(config: &Config, str: &str) -> String {
        // write buffer to current directory
        std::fs::write("buf", str).unwrap();

        // Open kitty with fzf in the external files directory
        Command::new("kitty")
            .args([
                "sh",
                {
                    let mut t = config.current_dir.clone();
                    t.push("fzf-to-title-url-name.sh");
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
        Command::new("xdg-open").arg(url).output().unwrap();
    }
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
