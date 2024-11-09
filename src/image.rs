use octocrab::{models::repos::CommitAuthor, Octocrab};

pub async fn update_image_file(docker_name: &str) -> anyhow::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
    let octocrab = Octocrab::builder().personal_token(token).build()?;
    // let octocrab = octocrab::instance();
    let owner = "kingzcheung";
    let repo = "docker_image_pusher";
    let path = "images.txt";
    let branch = "main";

    let c = octocrab
        .repos(owner, repo)
        .get_content()
        .path(path)
        .r#ref(branch)
        .send()
        .await?;

    let sha = &c.items[0].sha;
    dbg!(docker_name);

    let message = format!("sync {}", docker_name);
    let content = docker_name;
    octocrab
        .repos(owner, repo)
        .update_file(path, &message, content, sha)
        .branch(branch)
        .commiter(CommitAuthor {
            name: "KingzCheung".to_string(),
            email: "kingzcheung@gmail.com".to_string(),
            date: None,
        })
        .author(CommitAuthor {
            name: "KingzCheung".to_string(),
            email: "kingzcheung@gmail.com".to_string(),
            date: None,
        })
        .send()
        .await?;
    Ok(())
}
