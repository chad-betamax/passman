pub mod bash;
pub mod clap;

pub fn install() -> anyhow::Result<()> {
    clap::generate_completion_script()?;
    bash::install_file_path_completion()?;
    Ok(())
}
