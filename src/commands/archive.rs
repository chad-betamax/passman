use crate::config::Config;
use anyhow::{Context, Result};
use std::{fs, path::Path};

/// Hide a file or directory by renaming it to a dot‐prefixed hidden entry in the same directory.
///
/// Usage:
///     passman archive [--folder] <path>
///
/// `path` is relative to the vault root (base_dir + prefix).
/// When archiving a file, you can omit the crypto extension; it will be auto-added.
/// When `--folder` is set, `path` must refer to a dir (no extension logic applied).
pub fn run(cfg: &Config, path: String, folder: bool) -> Result<()> {
    // Compute vault root
    let vault_root = cfg.base_dir.join(&cfg.prefix);
    let mut full_path = vault_root.join(&path);

    // If archiving a file and no extension present, tack on the crypto_extension
    if !folder && full_path.extension().is_none() {
        let ext = cfg.crypto_extension.trim_start_matches('.');
        full_path.set_extension(ext);
    }

    // Existence check
    if !full_path.exists() {
        anyhow::bail!("Not found: {}", full_path.display());
    }
    // Type check
    if folder {
        if !full_path.is_dir() {
            anyhow::bail!("Not a directory: {}", full_path.display());
        }
    } else {
        if !full_path.is_file() {
            anyhow::bail!("Not a file: {}", full_path.display());
        }
    }

    // Extract the basename (with extension, if any)
    let name = full_path
        .file_name()
        .and_then(|n| n.to_str())
        .context("Invalid UTF-8 in name")?;

    // Prevent double‐archiving
    if name.starts_with('.') {
        anyhow::bail!(
            "{} already archived",
            if folder { "Directory" } else { "File" }
        );
    }

    // Build the new dot‐prefixed name
    let hidden_name = format!(".{}", name);
    let new_full = full_path.with_file_name(&hidden_name);

    // Perform the rename
    fs::rename(&full_path, &new_full).with_context(|| {
        format!(
            "Failed to archive {} → {}",
            full_path.display(),
            new_full.display()
        )
    })?;

    // Echo back only the relative path without extension
    let display_path = Path::new(&path).with_extension("");
    if folder {
        println!("Archived folder {}", display_path.display());
    } else {
        println!("Archived {}", display_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::fs::{self, File};
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    fn make_test_config(tmp: &TempDir) -> Config {
        Config {
            base_dir: tmp.path().to_path_buf(),
            prefix: tmp.path().to_path_buf(),
            secret: tmp.path().to_path_buf(),
            crypto_extension: "rage".into(),
            public_key_filename: "public.key".into(),
        }
    }

    #[test]
    fn file_not_found() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_test_config(&tmp);

        let err = run(&cfg, "foo".into(), false).unwrap_err();
        let expected = format!("Not found: {}", tmp.path().join("foo.rage").display());
        assert_eq!(err.to_string(), expected);
        Ok(())
    }

    #[test]
    fn dir_not_found_when_folder() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_test_config(&tmp);

        let err = run(&cfg, "bar".into(), true).unwrap_err();
        let expected = format!("Not found: {}", tmp.path().join("bar").display());
        assert_eq!(err.to_string(), expected);
        Ok(())
    }

    #[test]
    fn not_a_directory_error() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_test_config(&tmp);
        // create a file at path "entry"
        let entry = tmp.path().join("entry");
        File::create(&entry)?;

        let err = run(&cfg, "entry".into(), true).unwrap_err();
        let expected = format!("Not a directory: {}", entry.display());
        assert_eq!(err.to_string(), expected);
        Ok(())
    }

    #[test]
    fn not_a_file_error() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_test_config(&tmp);
        // create a directory at "foo.rage"
        let dir = tmp.path().join("foo.rage");
        fs::create_dir_all(&dir)?;

        let err = run(&cfg, "foo".into(), false).unwrap_err();
        let expected = format!("Not a file: {}", dir.display());
        assert_eq!(err.to_string(), expected);
        Ok(())
    }

    #[test]
    fn rename_failure_errors_with_context() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_test_config(&tmp);

        // Create the file "foo.rage"
        let file = tmp.path().join("foo.rage");
        File::create(&file)?.write_all(b"data")?;

        // Make the directory read-only so rename will fail
        let mut perms = fs::metadata(tmp.path())?.permissions();
        perms.set_mode(0o555);
        fs::set_permissions(tmp.path(), perms)?;

        // Attempt to archive "foo" → should hit our rename error context
        let err = run(&cfg, "foo".into(), false).unwrap_err();

        let hidden = tmp.path().join(".foo.rage");
        let expected = format!(
            "Failed to archive {} → {}",
            file.display(),
            hidden.display()
        );
        assert!(
            err.to_string().starts_with(&expected),
            "got `{}`, expected to start with `{}`",
            err,
            expected
        );

        Ok(())
    }
    #[test]
    fn already_archived_file_error() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_test_config(&tmp);
        // create the hidden file ".foo.rage"
        let hidden = tmp.path().join(".foo.rage");
        File::create(&hidden)?;

        let err = run(&cfg, ".foo".into(), false).unwrap_err();
        assert_eq!(err.to_string(), "File already archived");
        Ok(())
    }

    #[test]
    fn already_archived_directory_error() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_test_config(&tmp);
        // create the hidden directory ".bar"
        let hidden = tmp.path().join(".bar");
        fs::create_dir_all(&hidden)?;

        let err = run(&cfg, ".bar".into(), true).unwrap_err();
        assert_eq!(err.to_string(), "Directory already archived");
        Ok(())
    }

    #[test]
    fn successful_file_archive() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_test_config(&tmp);
        // create the file "foo.rage"
        let file = tmp.path().join("foo.rage");
        File::create(&file)?;

        run(&cfg, "foo".into(), false)?;

        // original should be gone, hidden should exist
        assert!(!file.exists());
        assert!(tmp.path().join(".foo.rage").exists());
        Ok(())
    }

    #[test]
    fn successful_directory_archive() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_test_config(&tmp);
        // create the directory "d1"
        let dir = tmp.path().join("d1");
        fs::create_dir_all(&dir)?;

        run(&cfg, "d1".into(), true)?;

        // original should be gone, hidden should exist
        assert!(!dir.exists());
        assert!(tmp.path().join(".d1").exists());
        Ok(())
    }
}
