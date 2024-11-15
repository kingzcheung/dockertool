use octocrab::{models::repos::CommitAuthor, Octocrab};

pub struct PushImage{
    octocrab: Octocrab,
    repo: String,
    owner: String,
    branch: Option<String>,
    path: Option<String>,
}

impl PushImage {
    pub fn new(token:&str,owner:&str,repo:&str) -> anyhow::Result<Self> {
        let octocrab = Octocrab::builder()
            .personal_token(token.to_string())
            .build()?;
        let branch = None;
        let path = None;
        let repo = repo.to_string();
        let owner = owner.to_string();
        
        Ok(Self { octocrab, repo, owner, branch, path })
    }

    pub async fn update_image_file(&self, docker_name: &str,git_user_name:Option<String>,git_user_email:Option<String>) -> anyhow::Result<()> {
        let path = self.path.clone().map_or("images.txt".into(), |v| v);
        let branch = self.branch.clone().map_or("main".into(), |v| v);
        let git_config = git2::Config::open_default()?;
        let git_user_name = git_user_name.unwrap_or(git_config.get_string("user.name")?);
        let git_user_email = git_user_email.unwrap_or(git_config.get_string("user.email")?);


        let c = self.octocrab
        .repos(self.owner.as_str(), self.repo.as_str())
        .get_content()
        .path(path.clone())
        .r#ref(branch.clone())
        .send()
        .await?;

    let sha = &c.items[0].sha;
    let message = format!("sync {}", docker_name);
    let content = docker_name;
    self.octocrab
        .repos(self.owner.as_str(), self.repo.as_str())
        .update_file(path, &message, content, sha)
        .branch(branch)
        .commiter(CommitAuthor {
            name: git_user_name.clone(),
            email: git_user_email.clone(),
            date: None,
        })
        .author(CommitAuthor {
            name: git_user_name.to_string(),
            email: git_user_email.to_string(),
            date: None,
        })
        .send()
        .await?;
        Ok(())
    }
}

