use crate::config::Config;
use crate::crypto;
use crate::utils::git_ops;
use anyhow::{Context, Result};
use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

type EditFn = fn() -> Result<String>;

thread_local! {
    // By default, call the real editor
    static EDIT_HOOK: std::cell::RefCell<EditFn> =
        std::cell::RefCell::new(real_edit);
}

/// Test-only hook to override `edit()` behavior
#[cfg(test)]
pub fn set_edit_hook(f: EditFn) {
    EDIT_HOOK.with(|c| *c.borrow_mut() = f);
}

pub fn run(config: &Config, path: String) -> Result<()> {
    let output_path = config.entry_path(&path);

    if output_path.exists() {
        anyhow::bail!("File already exists at {}.", output_path.display());
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    let public = config.read_public()?;
    let plaintext = EDIT_HOOK.with(|c| (c.borrow())())?;
    crypto::encrypt(&public, &output_path, &plaintext)?;
    println!("Password for '{}' stored successfully.", path);

    git_ops::sync_vault(&config.prefix)
}

fn real_edit() -> Result<String> {
    let file = NamedTempFile::new().context("Failed to create temporary file")?;
    let path = file.path();

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = Command::new(editor)
        .arg(path)
        .status()
        .context("Failed to launch $EDITOR")?;

    if !status.success() {
        anyhow::bail!("Editor exited with error");
    }

    let contents = fs::read_to_string(path)
        .context("Failed to read from temporary file")?
        .trim_end()
        .to_string();

    if contents.is_empty() {
        anyhow::bail!("Aborted: file was empty");
    }

    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::crypto::{CryptoBackend, set_encrypt_factory};
    use anyhow::Result;
    use serial_test::serial;
    use std::fs::{self, File};
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
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
    fn existing_file_errors() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_test_config(&tmp);

        let entry = cfg.entry_path("mypw");
        File::create(&entry)?;

        let err = run(&cfg, "mypw".into()).unwrap_err();
        let expected = format!("File already exists at {}.", entry.display());
        assert_eq!(err.to_string(), expected);
        Ok(())
    }

    #[test]
    fn cannot_create_parent_dir_errors() -> Result<()> {
        let tmp = TempDir::new()?;
        // create a file where we expect the dir
        let parent_file = tmp.path().join("parent");
        File::create(&parent_file)?;
        let mut cfg = make_test_config(&tmp);
        cfg.prefix = parent_file.clone();

        let err = run(&cfg, "mypw".into()).unwrap_err();
        let expected = format!("Failed to create directory {}", parent_file.display());
        assert!(
            err.to_string().contains(&expected),
            "got {:?}, expected to contain {:?}",
            err,
            expected
        );
        Ok(())
    }

    #[test]
    fn edit_success_and_encrypts() -> Result<()> {
        set_edit_hook(|| Ok("my secret".into()));

        struct Spy;
        impl CryptoBackend for Spy {
            fn encrypt(
                &self,
                _recipient: &str,
                _output_file: &Path,
                plaintext: &str,
            ) -> Result<()> {
                assert_eq!(plaintext, "my secret");
                Ok(())
            }
            fn decrypt(&self, _identity_file: &Path, _encrypted_file: &Path) -> Result<String> {
                panic!("decrypt should never be called");
            }
        }

        set_encrypt_factory(|_: &Path| Box::new(Spy));

        let tmp = TempDir::new()?;
        let pub_path = tmp.path().join("public.key");
        File::create(&pub_path)?.write_all(b"dummy")?;

        let cfg = make_test_config(&tmp);
        run(&cfg, "newpw".into())?;
        Ok(())
    }

    // Helper to build and chmod a one-line script
    fn make_editor_script(tmp: &TempDir, contents: &str) -> std::path::PathBuf {
        let script = tmp.path().join("editor.sh");
        let mut f = File::create(&script).unwrap();
        writeln!(f, "#!/bin/sh").unwrap();
        writeln!(f, "{}", contents).unwrap();
        let mut perms = fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script, perms).unwrap();
        script
    }

    #[test]
    #[serial]
    fn real_edit_empty_aborts() -> Result<()> {
        let tmp = TempDir::new()?;
        // script that does nothing (exit 0)
        let script = make_editor_script(&tmp, "exit 0");
        unsafe {
            std::env::set_var("EDITOR", &script);
        }

        let err = real_edit().unwrap_err();
        assert_eq!(err.to_string(), "Aborted: file was empty");
        Ok(())
    }

    #[test]
    #[serial]
    fn real_edit_editor_failure() -> Result<()> {
        // Use the builtin `false` to simulate an editor that runs but exits non-zero
        unsafe {
            std::env::set_var("EDITOR", "false");
        }

        let err = real_edit().unwrap_err();
        assert_eq!(err.to_string(), "Editor exited with error");
        Ok(())
    }

    #[test]
    #[serial]
    fn real_edit_success_reads_tempfile() -> Result<()> {
        let tmp = TempDir::new()?;
        // script that writes "magic" into $1
        let script = make_editor_script(&tmp, r#"echo "magic" > "$1""#);
        unsafe {
            std::env::set_var("EDITOR", &script);
        }

        let out = real_edit()?;
        assert_eq!(out, "magic");
        Ok(())
    }
}
