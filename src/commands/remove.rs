use crate::config::Config;
use crate::utils::git_ops;
use anyhow::{Context, Result};
use std::fs;

/// Delete a stored password entry
pub fn run(config: &Config, path: String) -> Result<()> {
    let file_path = config.prefix.join(format!("{}.rage", path));

    if !file_path.exists() {
        anyhow::bail!("No such entry: {}", file_path.display());
    }

    fs::remove_file(&file_path)
        .with_context(|| format!("Failed to delete file {:?}", file_path))?;

    // run from the vault dir ie the git root,
    // to pick up the deletion
    git_ops::sync_vault(&config.prefix)?;

    println!("✅ Removed entry `{}`", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use anyhow::Result;
    use std::fs::File;
    use tempfile::TempDir;

    /// Build a minimal Config where prefix==tmp, extension always "rage"
    fn make_config(tmp: &TempDir) -> Config {
        Config {
            base_dir: tmp.path().to_path_buf(),
            prefix: tmp.path().to_path_buf(),
            secret: tmp.path().join("secret.key"),
            crypto_extension: "rage".into(),
            public_key_filename: "public.key".into(),
        }
    }

    #[test]
    fn missing_entry_errors() {
        let tmp = TempDir::new().unwrap();
        let cfg = make_config(&tmp);

        let err = run(&cfg, "foo".to_string()).unwrap_err();
        let expected = format!("No such entry: {}", tmp.path().join("foo.rage").display());
        assert_eq!(err.to_string(), expected);
    }

    #[test]
    fn existing_entry_removed_successfully() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_config(&tmp);

        // create foo.rage
        let path = tmp.path().join("bar.rage");
        File::create(&path)?;

        // Should succeed and delete the file
        run(&cfg, "bar".to_string())?;
        assert!(!path.exists(), "file should have been deleted");

        Ok(())
    }
}
