use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate_to, shells::Bash};
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
};

/// Generate & install both the base clap completions and
/// your custom file‐based wrapper into ~/.config/bash/completions/{bin}.bash
pub fn install_file_path_completion(bin_name: &str) -> Result<()> {
    // 1. Prepare the output path
    let config_dir = dirs::config_dir().expect("Could not determine config dir");
    let bash_dir = config_dir.join("bash/completions");
    fs::create_dir_all(&bash_dir)?;
    let completion_file = bash_dir.join(format!("{}.bash", bin_name));

    // 2. Generate the raw clap script into that directory
    let mut cmd = crate::cli::Cli::command();
    generate_to(Bash, &mut cmd, bin_name, &bash_dir)?;

    // 3. Read & patch the generated script: remove any old if/else/fi and cur/prev by $2/$3
    let raw = fs::read_to_string(&completion_file)?;
    let mut patched = Vec::with_capacity(raw.len());
    let mut in_header = false;
    for line in raw.lines() {
        // Detect the start of the completer function:
        if line.starts_with(&format!("_{}()", bin_name)) {
            in_header = true;
            patched.push(line.to_owned());
            // Immediately inject our cur/prev block instead of the old header
            patched.push("    local i cur prev opts cmd".into());
            patched.push("    COMPREPLY=()".into());
            patched.push("    # Use COMP_WORDS exclusively".into());
            patched.push("    cur=\"${COMP_WORDS[COMP_CWORD]}\"".into());
            patched.push("    prev=\"${COMP_WORDS[COMP_CWORD-1]}\"".into());
            continue;
        }
        // Skip all old header lines until we hit the real body (cmd=)
        if in_header {
            if line.trim_start().starts_with("cmd=") {
                in_header = false;
                patched.push(line.to_owned());
            }
            // else drop the line
            continue;
        }
        // Outside the header, just keep the line
        patched.push(line.to_owned());
    }
    fs::write(&completion_file, patched.join("\n"))?;

    // 4. Compute your vault-dir string
    let mut vault = dirs::data_dir().expect("Could not determine data dir");
    vault.push("passman");
    vault.push("vault");
    let vault_dir = vault.to_string_lossy().into_owned();

    // 5. Append _{bin_name}_wrapper if not already present
    let already = {
        let f = fs::File::open(&completion_file)?;
        BufReader::new(f).lines().any(|l| {
            l.unwrap_or_default()
                .contains(&format!("_{}_wrapper", bin_name))
        })
    };
    if !already {
        let mut f = OpenOptions::new().append(true).open(&completion_file)?;
        let wrapper = format!(
            r#"
# === `{bin_name}` custom wrapper for file-based completions ===
_{bin_name}_wrapper() {{
    local subcommand cur files
    COMPREPLY=()
    cur="${{COMP_WORDS[COMP_CWORD]}}"
    subcommand="${{COMP_WORDS[1]}}"

    if [[ "$subcommand" == "list" || "$subcommand" == "show" ]]; then
        files=$(find "{vault}" -type f -name '*.rage' 2>/dev/null \
        | sed -e 's|.rage$||' -e 's|^{vault}/||')

        COMPREPLY=( $(compgen -W "${{files}}" -- "${{cur}}") )
    else
        # Delegate back to the original clap handler
        _{bin_name} "$@"
    fi
}}

# Override default handler but allow bash’s built-ins too
complete -F _{bin_name}_wrapper {bin_name}
"#,
            bin_name = bin_name,
            vault = vault_dir
        );
        f.write_all(wrapper.as_bytes())?;
    }

    Ok(())
}
