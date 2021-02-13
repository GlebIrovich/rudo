use self_update::cargo_crate_version;
use std::env;

const REPO_OWNER: &str = "GlebIrovich";
const REPO_NAME: &str = "rudo";
const EXE_NAME: &str = "rudo";

pub fn update() -> Result<(), Box<dyn ::std::error::Error>> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(EXE_NAME)
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    println!("Update status: `{}`!", status.version());
    Ok(())
}
