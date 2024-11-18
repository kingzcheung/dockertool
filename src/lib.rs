use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context;
use console::style;
use settings::{save_config, Settings};
use signer::{HttpRequest, Signer};

pub mod image;
pub mod schema;
pub mod settings;
pub mod signer;

pub async fn get_image_info(conf: &Settings,repository:&str) -> anyhow::Result<()> {
    let sign = Signer;
    // 如果有tag，去除tag
    let repository = repository.split(":").next().context("repository name should not contain tag")?;
    let url = format!(
            "https://swr-api.cn-south-1.myhuaweicloud.com/v2/manage/repos?namespace={namespace}&filter=name%3A%3A{repository}",
            // "https://swr-api.cn-south-1.myhuaweicloud.com/v2/manage/namespaces/{namespace}/repos/{repository}",
            namespace = conf.namespace.as_str(),
            repository = repository
        );
    let headers = HashMap::from([("content-type".to_string(), "application/json".to_string())]);
    let mut r = HttpRequest::new("GET", &url, Some(headers), "");
    sign.sign(&mut r, &conf.ak, &conf.sk);

    let rr = r.list_repos_details().await?;
    dbg!(rr);

    Ok(())
}

pub fn config_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home dir")?;
    let path = Path::new(&home).join(".config").join(".dockertool.toml");
    Ok(path)
}

pub fn set_config() -> anyhow::Result<()> {
    cliclack::clear_screen()?;
    cliclack::intro(style(" dockertool config ").on_cyan().black())?;

    let github_pusher_repo: String = cliclack::input("where is your github pusher repo?")
        .placeholder("https://github.com/kingzcheung/docker_image_pusher")
        .validate(|input: &String| {
            if input.is_empty() {
                Err("Please enter your github pusher repo.")
            } else if !input.starts_with("https://") {
                Err("Please enter a github pusher repo url starts with https://.")
            } else {
                Ok(())
            }
        })
        .interact()?;

    let github_token: String = cliclack::input("what is your github token?")
        .placeholder("github_xxx-xxxxxxxx")
        .validate(|input: &String| {
            if input.is_empty() {
                Err("Please enter your github token.")
            } else {
                Ok(())
            }
        })
        .interact()?;

    let ak: String = cliclack::input("what is your huawei cloud ak?")
        .placeholder("xxxxxxxx")
        .validate(|input: &String| {
            if input.is_empty() {
                Err("Please enter your huawei cloud ak.")
            } else {
                Ok(())
            }
        })
        .interact()?;

    let sk: String = cliclack::input("what is your huawei cloud sk?")
        .placeholder("xxxxxxxx")
        .validate(|input: &String| {
            if input.is_empty() {
                Err("Please enter your huawei cloud sk.")
            } else {
                Ok(())
            }
        })
        .interact()?;

    let namespace: String = cliclack::input("what is your huawei cloud namespace?")
        .placeholder("my_namespace")
        .validate(|input: &String| {
            if input.is_empty() {
                Err("Please enter your huawei cloud namespace.")
            } else {
                Ok(())
            }
        })
        .interact()?;

    let settings = settings::Settings {
        github_token,
        github_pusher_repo,
        ak,
        sk,
        namespace,
    };
    let path = config_path()?;
    save_config(&path, settings)?;
    Ok(())
}
