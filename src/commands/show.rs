use crate::config::Config;
use crate::crypto;
use crate::utils::qr::print_qr;
use anyhow::Result;

/// Show a password entry, optionally only a single line.
pub fn run(config: &Config, path: String, qrcode: bool, line: Option<usize>) -> Result<()> {
    let file_path = config.entry_path(&path);
    if !file_path.exists() {
        anyhow::bail!("No such password: {}", file_path.display());
    }

    let decrypted = crypto::decrypt(&config.secret, &file_path)?;

    let output = match line {
        None | Some(0) => decrypted.clone(),
        Some(n) => decrypted
            .lines()
            .nth(n - 1)
            .ok_or_else(|| anyhow::anyhow!("File {} has fewer than {} lines", path, n))?
            .to_string(),
    };

    if qrcode {
        print_qr(&output)?;
    } else {
        println!("{}", output);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::crypto::{CryptoBackend, set_decrypt_factory};
    use anyhow::Result;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    /// Build a minimal Config with a given tempdir, using `.rage` as the extension.
    fn make_test_config(tmp: &TempDir) -> Config {
        Config {
            base_dir: tmp.path().to_path_buf(),
            prefix: tmp.path().to_path_buf(),
            secret: tmp.path().to_path_buf(), // still safe to point here
            crypto_extension: "rage".into(),
            public_key_filename: "public.key".into(),
        }
    }

    /// A mock backend that always returns three lines.
    struct MockBackend;

    impl CryptoBackend for MockBackend {
        fn encrypt(
            &self,
            _recipient: &str,
            _output_file: &std::path::Path,
            _plaintext: &str,
        ) -> Result<()> {
            unreachable!("encrypt not used in show::run tests");
        }

        fn decrypt(
            &self,
            _identity_file: &std::path::Path,
            _encrypted_file: &std::path::Path,
        ) -> Result<String> {
            // simulate a file whose plaintext is exactly three lines
            Ok("first line\nsecond line\nthird line".into())
        }
    }

    /// Factory that always yields our `MockBackend`.
    fn mock_factory(_: &std::path::Path) -> Box<dyn CryptoBackend> {
        Box::new(MockBackend)
    }

    #[test]
    fn missing_file_errors_gracefully() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let cfg = make_test_config(&tmp);

        let err = run(&cfg, "does_not_exist".into(), false, None).unwrap_err();
        let expected = format!(
            "No such password: {}",
            cfg.entry_path("does_not_exist").display()
        );
        assert_eq!(err.to_string(), expected);
        Ok(())
    }

    #[test]
    fn prints_entire_decrypted_contents_when_no_line_arg() -> Result<()> {
        set_decrypt_factory(mock_factory);

        let tmp = tempfile::tempdir()?;
        let cfg = make_test_config(&tmp);

        // create the file with the proper extension
        let entry = cfg.entry_path("mypw");
        File::create(&entry)?.write_all(b"")?;

        // should not error
        run(&cfg, "mypw".into(), false, None)?;
        Ok(())
    }

    #[test]
    fn selects_correct_line_if_requested() -> Result<()> {
        set_decrypt_factory(mock_factory);

        let tmp = tempfile::tempdir()?;
        let cfg = make_test_config(&tmp);

        let entry = cfg.entry_path("mypw");
        File::create(&entry)?.write_all(b"")?;

        // asking for line 2 should succeed ("second line")
        run(&cfg, "mypw".into(), false, Some(2))?;
        Ok(())
    }

    #[test]
    fn error_if_line_out_of_range() -> Result<()> {
        set_decrypt_factory(mock_factory);

        let tmp = tempfile::tempdir()?;
        let cfg = make_test_config(&tmp);

        let entry = cfg.entry_path("mypw");
        File::create(&entry)?; // empty file

        let err = run(&cfg, "mypw".into(), false, Some(10)).unwrap_err();
        assert!(
            err.to_string().contains("has fewer than 10 lines"),
            "unexpected error: {}",
            err
        );
        Ok(())
    }
}
