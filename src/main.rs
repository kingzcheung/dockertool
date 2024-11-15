use std::path::PathBuf;

use clap::{Parser, Subcommand};
use dockertool::{config_path, image::PushImage, set_config, settings};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Config,
    /// 同步镜像
    Sync {
        /// 镜像名称
        /// 如 "docker.io/library/nginx:latest"
        /// 或者 "nginx:latest"
        image: String,
        /// github 的推送仓库地址,如 abc/docker_image_pusher
        /// 需要 fork [kingzcheung/docker_image_pusher](https://github.com/kingzcheung/docker_image_pusher) 到你自己的账户下
        #[arg(short, long)]
        pusher: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    match &cli.command {
        Some(Commands::Config) => {
            ctrlc::set_handler(move || {}).expect("setting Ctrl-C handler");

            if let Err(e) = set_config() {
                println!("please set config first:{}", e);
            }
        }
        Some(Commands::Sync { image, pusher }) => {
            let path = config_path().expect("Home path not found");
            let settings = settings::load_config(&path).expect("Please set config first");

            let pusher_url = pusher.clone().unwrap_or(settings.github_pusher_repo);

            let (owner, repo) = parse_pusher_args(&pusher_url).unwrap();

            let push_image = PushImage::new(&settings.github_token, &owner, &repo).unwrap();
            if let Err(e) = push_image.update_image_file(image, None, None).await {
                println!("error:{e}")
            }
        }
        None => {}
    }
}

/// 支持 owner/repo
/// 支持 github 上的 url，如: https://github.com/kingzcheung/docker_image_pusher
fn parse_pusher_args(pusher: &str) -> anyhow::Result<(String, String)> {
    let mut push_path = pusher.trim_end_matches('/').to_string();
    if push_path.starts_with("https://github.com/") {
        push_path = push_path.replace("https://github.com/", "");
    }

    let res = push_path.split("/").collect::<Vec<_>>();
    if res.len() == 2 {
        return Ok((res[0].to_string(), res[1].to_string()));
    }

    anyhow::bail!("pusher is not valid")
}
