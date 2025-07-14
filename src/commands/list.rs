use crate::config::Config;
use anyhow::Result;
use std::{fs, path::Path, path::PathBuf};

pub fn run(config: &Config, path: Option<String>, show_all: bool) -> Result<()> {
    // Determine crypto file extension (ensure it starts with a dot)
    let ext = if config.crypto_extension.starts_with('.') {
        config.crypto_extension.clone()
    } else {
        format!(".{}", config.crypto_extension)
    };

    // Resolve base directory
    let base: PathBuf = match path {
        Some(sub) => {
            let full_path = config.prefix.join(&sub);
            if full_path.is_dir() {
                full_path
            } else {
                let with_ext = config.prefix.join(format!("{sub}{ext}"));
                if with_ext.exists() {
                    println!("{}", sub);
                    return Ok(());
                } else {
                    anyhow::bail!("Entry not found: {}", sub);
                }
            }
        }
        None => config.prefix.clone(),
    };

    // Print header
    let label = base
        .strip_prefix(&config.prefix)
        .map(|p| {
            if p.as_os_str().is_empty() {
                "vault".to_string()
            } else {
                format!("vault/{}", p.display())
            }
        })
        .unwrap_or_else(|_| base.display().to_string());
    println!("📂 {}", label);

    // Walk tree
    walk(&base, vec![], show_all, &ext)?;
    Ok(())
}

fn walk(dir: &Path, prefix_parts: Vec<bool>, show_all: bool, ext: &str) -> Result<()> {
    let mut entries = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| {
            let file_name_os = e.file_name();
            let name = file_name_os.to_string_lossy();
            // Always skip .git, and skip hidden unless --all
            name != ".git" && (show_all || !name.starts_with('.'))
        })
        .collect::<Vec<_>>();

    entries.sort_by_key(|e| e.file_name());
    let last_idx = entries.len().saturating_sub(1);

    for (i, entry) in entries.into_iter().enumerate() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_last = i == last_idx;

        // draw prefix tree lines
        for &draw_line in &prefix_parts {
            if draw_line {
                print!("│   ");
            } else {
                print!("    ");
            }
        }

        let branch = if is_last { "└── " } else { "├── " };

        if path.is_dir() {
            println!("{}{}", branch, name);
            let mut new_prefix = prefix_parts.clone();
            new_prefix.push(!is_last);
            walk(&path, new_prefix, show_all, ext)?;
        } else if name.ends_with(ext) {
            let short = name.strip_suffix(ext).unwrap_or(&name);
            println!("{}{}", branch, short);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::fs::File;
    use tempfile::TempDir;

    /// Build a Config pointing `prefix` at the temp directory.
    fn make_config(tmp: &TempDir, ext: &str) -> Config {
        Config {
            base_dir: tmp.path().to_path_buf(),
            prefix: tmp.path().to_path_buf(),
            secret: tmp.path().join("secret.key"),
            crypto_extension: ext.into(),
            public_key_filename: "public.key".into(),
        }
    }

    #[test]
    fn missing_entry_returns_err() {
        let tmp = TempDir::new().unwrap();
        let cfg = make_config(&tmp, "rage");

        let err = run(&cfg, Some("nope".to_string()), false).unwrap_err();
        assert_eq!(err.to_string(), "Entry not found: nope");
    }

    #[test]
    fn file_entry_returns_ok() {
        let tmp = TempDir::new().unwrap();
        let cfg = make_config(&tmp, "rage");
        // create foo.rage
        File::create(tmp.path().join("foo.rage")).unwrap();

        assert!(run(&cfg, Some("foo".to_string()), false).is_ok());
    }

    #[test]
    fn directory_entry_returns_ok() {
        let tmp = TempDir::new().unwrap();
        let cfg = make_config(&tmp, "rage");
        // create a subdir
        let sub = tmp.path().join("subdir");
        fs::create_dir_all(&sub).unwrap();

        assert!(run(&cfg, Some("subdir".to_string()), false).is_ok());
    }

    #[test]
    fn extension_formatting_dot_and_no_dot() {
        let tmp = TempDir::new().unwrap();
        // ext without dot
        let cfg1 = make_config(&tmp, "age");
        let err1 = run(&cfg1, Some("x".to_string()), false).unwrap_err();
        assert_eq!(err1.to_string(), "Entry not found: x");

        // ext with dot
        let cfg2 = make_config(&tmp, ".age");
        let err2 = run(&cfg2, Some("x".to_string()), false).unwrap_err();
        assert_eq!(err2.to_string(), "Entry not found: x");

        // create a.age and x.age
        File::create(tmp.path().join("a.age")).unwrap();
        assert!(run(&cfg2, Some("a".to_string()), false).is_ok());

        File::create(tmp.path().join("b.age")).unwrap();
        assert!(run(&cfg1, Some("b".to_string()), false).is_ok());
    }

    #[test]
    fn list_root_ok() {
        let tmp = TempDir::new().unwrap();
        let cfg = make_config(&tmp, "rage");
        // empty directory
        assert!(run(&cfg, None, false).is_ok());
    }

    #[test]
    fn show_all_includes_hidden() {
        let tmp = TempDir::new().unwrap();
        let cfg = make_config(&tmp, "rage");
        // create visible and hidden files
        File::create(tmp.path().join("v.rage")).unwrap();
        File::create(tmp.path().join(".h.rage")).unwrap();

        // without show_all, Ok and skip hidden
        assert!(run(&cfg, None, false).is_ok());
        // with show_all, Ok and include hidden in walk
        assert!(run(&cfg, None, true).is_ok());
    }
}
