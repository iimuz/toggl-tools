use anyhow::{Context, Error, Result};
use chrono::prelude::*;
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::Deserialize;
use std::collections::HashMap;

/// Togglからデータを取得し要約した結果を返す。
#[derive(Debug, Parser)]
struct Cli {
    #[clap(flatten)]
    verbose: Verbosity,
}

struct ToggleService {
    api_token: String,
}

impl ToggleService {
    fn get_time_entries(&self) -> Result<Vec<TimeEntry>, Error> {
        // let start_date_str = "2023-09-19T00:00:00.0000+09:00";
        let start_date_str = "2023-09-19";
        let start_date: DateTime<Local> = start_date_str.parse::<DateTime<Local>>().with_context(|| format!("could not convert start date from string."))?;
        log::debug!("{:?}, {:?}", start_date, start_date.to_rfc3339());

        let client = Client::new();
        let text = client
            .get("https://api.track.toggl.com/api/v9/me/time_entries".to_string())
            .basic_auth(&self.api_token, Some("api_token"))
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .query(&[
                // ("start_date", "2023-09-19T00:00:00.0000+09:00".to_string()),
                ("start_date", start_date.to_rfc3339()),
                ("end_date", "2023-09-20T00:00:00.0000+09:00".to_string()),
            ])
            .send()
            .with_context(|| format!("could not get."))?
            .text()
            .with_context(|| format!("could not get text"))?;

        let time_entry = serde_json::from_str::<Vec<TimeEntry>>(&text)?;

        Ok(time_entry)
    }

    fn get_projects(&self) -> Result<Vec<Project>, Error> {
        let client = Client::new();
        let text = client
            .get("https://api.track.toggl.com/api/v9/me/projects".to_string())
            .basic_auth(&self.api_token, Some("api_token"))
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .query(&[("workspace_id", &self.api_token)])
            .send()
            .with_context(|| format!("could not get."))?
            .text()
            .with_context(|| format!("could not get text"))?;
        let projects = serde_json::from_str::<Vec<Project>>(&text)?;

        Ok(projects)
    }
}

#[derive(Debug, Deserialize)]
struct TimeEntry {
    description: String,
    start: String,
    duration: i64,
    project_id: i64,
    tag_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
struct Project {
    id: i64,
    name: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();
    log::info!("{:#?}", args);

    let api_token = "hoge";
    let toggl_service = ToggleService {
        api_token: api_token.to_string(),
    };
    let iter = toggl_service
        .get_time_entries()
        .with_context(|| format!("could not get time entries."))?;
    let mut project_duration = HashMap::<i64, i64>::new();
    for entry in iter.iter() {
        *project_duration.entry(entry.project_id).or_default() += entry.duration;
    }

    let iter = toggl_service
        .get_projects()
        .with_context(|| format!("could not get projects."))?;
    let project_map = iter
        .into_iter()
        .map(|x| (x.id, x))
        .collect::<HashMap<_, _>>();
    for (project_id, duration) in project_duration.iter() {
        let project = project_map.get(project_id).unwrap();
        log::debug!("{}: {}", project.name, duration);
    }

    Ok(())
}
