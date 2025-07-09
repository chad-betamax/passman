pub mod bash;
pub mod clap;

pub fn install() -> anyhow::Result<()> {
    // figure out exactly which binary the user is running right now
    let current_exe = std::env::current_exe()?;
    let bin_name = current_exe
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("passman");

    clap::generate_completion_script(bin_name)?;
    bash::install_file_path_completion(bin_name)?;
    Ok(())
}
