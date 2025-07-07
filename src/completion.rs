use anyhow::Result;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn install() -> Result<()> {
    let target_path = dirs::config_dir()
        .unwrap_or_else(|| "~/.config".into())
        .join("bash/passman.bash");

    let marker = "# passman bash completion";

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Avoid re-writing if function already exists
    if target_path.exists() {
        let f = fs::File::open(&target_path)?;
        for line in BufReader::new(f).lines().flatten() {
            if line.trim() == marker {
                println!(
                    "🔁 Completion script already exists at {}",
                    target_path.display()
                );
                return ensure_bashrc_sourced(&target_path);
            }
        }
    }

    // Write the function
    let mut file = fs::File::create(&target_path)?;
    writeln!(file, "{marker}")?;
    writeln!(
        file,
        r#"_passman_complete() {{
  local cur="${{COMP_WORDS[COMP_CWORD]}}"
  local base="$HOME/.passman/vault"
  local entries

  if [[ -d "$base" ]]; then
    entries=$(find "$base" -type f -name "*.rage" -print 2>/dev/null \
      | sed "s|^$base/||" \
      | sed 's/\.rage$//')
    COMPREPLY=( $(compgen -W "${{entries}}" -- "$cur") )
  else
    COMPREPLY=()
  fi
}}"#
    )?;
    writeln!(file, "complete -F _passman_complete passman")?;

    println!("✅ Bash completion installed to {}", target_path.display());

    ensure_bashrc_sourced(&target_path)
}

/// Ensure that ~/.bashrc sources the generated script
fn ensure_bashrc_sourced(target_path: &Path) -> Result<()> {
    let bashrc_path = dirs::home_dir().unwrap().join(".bashrc");

    let source_line = format!("source {}", target_path.display());
    if bashrc_path.exists() {
        let f = fs::File::open(&bashrc_path)?;
        for line in BufReader::new(f).lines().flatten() {
            if line.trim() == source_line {
                return Ok(());
            }
        }
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&bashrc_path)?;

    writeln!(file, "\n# Added by passman\n{}", source_line)?;
    println!("🔗 Added `source` line to ~/.bashrc");
    println!("👉 Run `source ~/.bashrc` or restart your shell to activate.");

    Ok(())
}
