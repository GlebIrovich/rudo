const REPO_OWNER: &str = "GlebIrovich";
const REPO_NAME: &str = "rudo";
const EXE_NAME: &str = "rudo";

pub const CURRENT_APP_VERSION: &str = "0.2.2";

pub fn update() -> Result<String, Box<dyn ::std::error::Error>> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(EXE_NAME)
        .show_download_progress(true)
        .current_version(CURRENT_APP_VERSION)
        .build()?
        .update()?;

    let version = status.version().to_string();
    Ok(version)
}
