use crate::config::Config;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn run(config: &Config, path: Option<String>) -> Result<()> {
    let base = match path {
        Some(sub) => {
            let full_path = config.prefix.join(&sub);
            if full_path.is_dir() {
                full_path
            } else {
                let with_ext = config.prefix.join(format!("{sub}.rage"));
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
    walk(&base, vec![])?;
    Ok(())
}

fn walk(dir: &Path, prefix_parts: Vec<bool>) -> Result<()> {
    let mut entries = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| e.file_name() != ".git") // exclude .git directory
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
            walk(&path, new_prefix)?;
        } else if name.ends_with(".rage") {
            let short = name.strip_suffix(".rage").unwrap_or(&name);
            println!("{}{}", branch, short);
        }
    }

    Ok(())
}
