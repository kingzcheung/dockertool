use std::path::{Path, PathBuf};

use anyhow::Context;
use console::style;
use settings::save_config;

pub mod image;
pub mod schema;
pub mod settings;
pub mod signer;

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

    let settings = settings::Settings {
        github_token,
        github_pusher_repo,
        ak,
        sk,
    };
    let path = config_path()?;
    save_config(&path, settings)?;
    Ok(())
}
