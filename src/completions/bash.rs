use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate_to, shells::Bash};
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
};

use crate::cli::Cli;
/// complete dirs under vault, excluding `.git` and its contents
fn vault_list_pipeline(vault_dir: &str) -> String {
    let find_cmd = format!(
        "cd \"{vault}\" && find . -mindepth 1 \
\\( -path './.git' -o -path './.git/*' \\) -prune -o \
-type d -printf '%P/\\n' | sort -u",
        vault = vault_dir
    );
    format!("dirs=$({})", find_cmd)
}

/// complete only files under vault, strip both .rage and .age
fn vault_show_pipeline(vault_dir: &str) -> String {
    format!(
        r#"files=$(cd "{vault}" && find . -type f \( -name '*.rage' -o -name '*.age' \) 2>/dev/null \
             | sed -e 's|^\./||' -e 's|\.rage$||' -e 's|\.age$||')"#,
        vault = vault_dir
    )
}

/// Generate & install both the base clap completions and
/// the custom file-based wrapper into ~/.config/bash/completions/{bin}.bash
pub fn install_file_path_completion(bin_name: &str) -> Result<()> {
    // Prepare the output path
    let config_dir = dirs::config_dir().expect("Could not determine config dir");
    let bash_dir = config_dir.join("bash/completions");
    fs::create_dir_all(&bash_dir)?;
    let completion_file = bash_dir.join(format!("{}.bash", bin_name));

    // Generate the raw clap script
    let mut cmd = Cli::command();
    generate_to(Bash, &mut cmd, bin_name, &bash_dir)?;

    let raw = fs::read_to_string(&completion_file)?;
    let mut patched = Vec::with_capacity(raw.len());
    let mut in_header = false;
    for line in raw.lines() {
        if line.starts_with(&format!("_{}()", bin_name)) {
            in_header = true;
            patched.push(line.to_owned());
            patched.push("    local i cur prev opts cmd".into());
            patched.push("    COMPREPLY=()".into());
            patched.push("    # Use COMP_WORDS exclusively".into());
            patched.push("    cur=\"${COMP_WORDS[COMP_CWORD]}\"".into());
            patched.push("    prev=\"${COMP_WORDS[COMP_CWORD-1]}\"".into());
            continue;
        }
        if in_header {
            // skip old header lines until the real body
            if line.trim_start().starts_with("cmd=") {
                in_header = false;
                patched.push(line.to_owned());
            }
            continue;
        }
        patched.push(line.to_owned());
    }
    fs::write(&completion_file, patched.join("\n"))?;

    // Compute your vault-dir string
    let vault_dir_str = {
        let mut vault = dirs::data_dir().expect("Could not determine data dir");
        vault.push("passman");
        vault.push("vault");
        vault.to_string_lossy().into_owned()
    };

    // Append _{bin_name}_wrapper if not already present
    let already = {
        let f = fs::File::open(&completion_file)?;
        BufReader::new(f).lines().any(|l| {
            l.unwrap_or_default()
                .contains(&format!("_{}_wrapper", bin_name))
        })
    };
    if !already {
        let mut f = OpenOptions::new().append(true).open(&completion_file)?;
        let list_pipe = vault_list_pipeline(&vault_dir_str);
        let show_pipe = vault_show_pipeline(&vault_dir_str);
        let wrapper = format!(
            r#"
# === `{bin}` custom wrapper for file-based completions ===
_{bin}_wrapper() {{
    local subcommand cur
    COMPREPLY=()
    cur="${{COMP_WORDS[COMP_CWORD]}}"
    subcommand="${{COMP_WORDS[1]}}"

    if [[ "$subcommand" == "list" || "$subcommand" == "new" || "$subcommand" == "edit" ]]; then
        {list_pipe}

        # read "$dirs" (newline-separated) into an array
        local raw; IFS=$'\n' read -r -d '' -a raw < <(echo "$dirs"; printf '\0')

        # filter out duplicates and anything not ending in '/'
        declare -A seen
        local filtered=() item
        for item in "${{raw[@]}}"; do
            if [[ "$item" == */ ]] && [[ -z "${{seen[$item]}}" ]]; then
                filtered+=("$item")
                seen[$item]=1
            fi
        done

        COMPREPLY=( $(compgen -W "${{filtered[*]}}" -- "$cur") )
        compopt -o nospace

    elif [[ "$subcommand" == "show" ]]; then
        {show_pipe}
        COMPREPLY=( $(compgen -W "${{files}}" -- "$cur") )

    else
        # fallback to clap’s builtin handler
        _{bin} "$@"
    fi
}}

# install our wrapper (and disable the automatic space after “/”)
complete -F _{bin}_wrapper -o nospace {bin}
"#,
            bin = bin_name,
            list_pipe = list_pipe,
            show_pipe = show_pipe
        );

        f.write_all(wrapper.as_bytes())?;
    }

    Ok(())
}
