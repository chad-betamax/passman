use crate::config::Config;
use crate::crypto;
use crate::utils::git_ops;
use anyhow::{Context, Result};
use std::cell::RefCell;
use std::path::Path;
use std::{env, fs, process::Command};
use tempfile::NamedTempFile;

/// Edit an existing password entry in your $EDITOR and re‐encrypt it.
pub fn run(config: &Config, path: String) -> Result<()> {
    // Locate the encrypted file
    let file_path = config.entry_path(&path);
    if !file_path.exists() {
        anyhow::bail!("No such password: {}", file_path.display());
    }

    // Decrypt using private key
    let plaintext =
        crypto::decrypt(&config.secret, &file_path).context("Failed to decrypt existing entry")?;

    // Write decrypted contents to a temp file (through a hook for testability)
    let tmp = NamedTempFile::new().context("Failed to create temporary file")?;
    let p = tmp.path();
    WRITE_HOOK.with(|h| {
        (h.borrow())(p, &plaintext)
            .with_context(|| format!("Failed to write to temporary file {:?}", p))
    })?;

    // Launch editor
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = Command::new(editor)
        .arg(tmp.path())
        .status()
        .context("Failed to launch $EDITOR")?;
    if !status.success() {
        anyhow::bail!("Editor exited with an error");
    }

    // Read updated contents
    let updated = fs::read_to_string(tmp.path())
        .context("Failed to read from temporary file")?
        .trim_end()
        .to_string();
    if updated.is_empty() {
        anyhow::bail!("Aborted: no content (file was empty)");
    }

    // Re-encrypt with updated contents
    let public = config.read_public()?;
    crypto::encrypt(&public, &file_path, &updated).context("Failed to re-encrypt updated entry")?;
    println!("Password for '{}' updated successfully.", path);

    // Sync vault
    git_ops::sync_vault(&config.prefix)?;

    Ok(())
}

thread_local! {
    static WRITE_HOOK: RefCell<fn(&Path, &str) -> std::io::Result<()>> =
        RefCell::new(|p, contents| fs::write(p, contents));
}

/// Override the temp‐file write behavior (test only).
#[cfg(test)]
pub fn set_write_hook(f: fn(&Path, &str) -> std::io::Result<()>) {
    WRITE_HOOK.with(|h| *h.borrow_mut() = f);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::crypto::{CryptoBackend, set_decrypt_factory, set_encrypt_factory};
    use anyhow::Result;
    use serial_test::serial;
    use std::{
        env,
        fs::{self, File},
        io::Write,
        os::unix::fs::PermissionsExt,
        path::{Path, PathBuf},
    };
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

    // Backend that always fails decrypt with "boom"
    struct BoomBackend;
    impl CryptoBackend for BoomBackend {
        fn decrypt(&self, _id: &Path, _enc: &Path) -> Result<String> {
            Err(anyhow::anyhow!("boom"))
        }
        fn encrypt(&self, _r: &str, _o: &Path, _p: &str) -> Result<()> {
            unreachable!()
        }
    }
    fn boom_factory(_: &Path) -> Box<dyn CryptoBackend> {
        Box::new(BoomBackend)
    }

    // Backend that returns a fixed plaintext
    struct OkBackend(&'static str);
    impl CryptoBackend for OkBackend {
        fn decrypt(&self, _id: &Path, _enc: &Path) -> Result<String> {
            Ok(self.0.to_string())
        }
        fn encrypt(&self, _r: &str, _o: &Path, _p: &str) -> Result<()> {
            Ok(())
        }
    }
    fn ok_factory_empty(_: &Path) -> Box<dyn CryptoBackend> {
        Box::new(OkBackend(""))
    }
    fn ok_factory_orig(_: &Path) -> Box<dyn CryptoBackend> {
        Box::new(OkBackend("orig"))
    }

    /// Helper to write & chmod a one‐line script in `tmp`
    fn make_editor_script(tmp: &TempDir, line: &str) -> PathBuf {
        let script = tmp.path().join("editor.sh");
        let mut f = File::create(&script).unwrap();
        writeln!(f, "#!/bin/sh").unwrap();
        writeln!(f, "{}", line).unwrap();
        let mut perms = fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script, perms).unwrap();
        script
    }

    #[test]
    fn missing_file_errors() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_test_config(&tmp);

        let err = run(&cfg, "noentry".into()).unwrap_err();
        let expected = format!("No such password: {}", cfg.entry_path("noentry").display());
        assert_eq!(err.to_string(), expected);
        Ok(())
    }

    #[test]
    fn decrypt_failure_errors() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_test_config(&tmp);

        let entry = cfg.entry_path("entry");
        File::create(&entry)?;

        set_decrypt_factory(boom_factory);

        let err = run(&cfg, "entry".into()).unwrap_err();
        assert_eq!(err.to_string(), "Failed to decrypt existing entry");
        Ok(())
    }

    #[test]
    #[serial]
    fn abort_on_empty_edit() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_test_config(&tmp);

        let entry = cfg.entry_path("entry");
        File::create(&entry)?;

        set_decrypt_factory(ok_factory_empty);

        let script = make_editor_script(&tmp, "exit 0");
        unsafe {
            env::set_var("EDITOR", &script);
        }

        let err = run(&cfg, "entry".into()).unwrap_err();
        assert_eq!(err.to_string(), "Aborted: no content (file was empty)");
        Ok(())
    }

    /// Simulate the editor exiting with a non-zero code
    #[test]
    #[serial]
    fn editor_exit_error() -> Result<()> {
        let tmp = TempDir::new()?;
        // create public key so read_public() succeeds
        let pub_key = tmp.path().join("public.key");
        File::create(&pub_key)?;

        // set up config and a dummy encrypted file
        let cfg = make_test_config(&tmp);
        let entry = cfg.entry_path("entry");
        File::create(&entry)?;

        // stub decrypt to return some non-empty plaintext
        set_decrypt_factory(ok_factory_orig);

        // Use the builtin `false` to simulate an editor that runs but exits non-zero
        unsafe {
            std::env::set_var("EDITOR", "false");
        }

        let err = run(&cfg, "entry".into()).unwrap_err();
        assert_eq!(err.to_string(), "Editor exited with an error");
        Ok(())
    }

    #[test]
    #[serial]
    fn success_and_reencrypts() -> Result<()> {
        let tmp = TempDir::new()?;
        let pub_key = tmp.path().join("public.key");
        File::create(&pub_key)?;

        let cfg = make_test_config(&tmp);
        let entry = cfg.entry_path("entry");
        File::create(&entry)?;

        set_decrypt_factory(ok_factory_orig);

        struct Spy;
        impl CryptoBackend for Spy {
            fn encrypt(&self, _r: &str, _o: &Path, p: &str) -> Result<()> {
                assert_eq!(p, "updated");
                Ok(())
            }
            fn decrypt(&self, _i: &Path, _e: &Path) -> Result<String> {
                unreachable!()
            }
        }
        set_encrypt_factory(|_: &Path| Box::new(Spy));

        let script = make_editor_script(&tmp, r#"echo "updated" > "$1""#);
        unsafe {
            env::set_var("EDITOR", &script);
        }

        run(&cfg, "entry".into())?;
        Ok(())
    }

    #[test]
    #[serial]
    fn write_failure_errors() -> Result<()> {
        let tmp = TempDir::new()?;
        let pub_key = tmp.path().join("public.key");
        File::create(&pub_key)?;

        let cfg = make_test_config(&tmp);
        let entry = cfg.entry_path("e1");
        File::create(&entry)?;

        set_decrypt_factory(ok_factory_empty);

        // Force the write to fail
        set_write_hook(|_, _| Err(std::io::Error::new(std::io::ErrorKind::Other, "fail")));

        let err = run(&cfg, "e1".into()).unwrap_err();
        assert!(
            err.to_string()
                .starts_with("Failed to write to temporary file"),
            "unexpected: {}",
            err
        );
        Ok(())
    }
}
