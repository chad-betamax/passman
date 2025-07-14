use crate::{completions, config::Config, utils::keygen};
use anyhow::{Context, Result};
use dialoguer::{Confirm, Input};
use std::{env, fs, path::Path, process::ExitStatus};

/// Run the init command: generate keys, initialize git, install completions.
pub fn run(config: &Config) -> Result<()> {
    let secret_path = &config.secret;
    let public_path = &config.base_dir.join("public.key");

    // 1) Keypair generation prompt
    let mut do_generate = true;
    if secret_path.exists() || public_path.exists() {
        let prompt = format!(
            "⚠️  DANGER: Overwriting your secret key is irreversible!\n\
🔐 Private key: {}\n\
🟢 Public key:  {}\n\n\
Any files encrypted with your *current* private key will become PERMANENTLY inaccessible!\n\
Do you REALLY want to overwrite these files?",
            secret_path.display(),
            public_path.display()
        );
        if !CONFIRM_HOOK.with(|h| (h.borrow())(&prompt, false)) {
            println!("ℹ️  Keeping existing keys; skipping key generation.");
            do_generate = false;
        } else {
            fs::remove_file(secret_path).ok();
            fs::remove_file(public_path).ok();
        }
    }
    if do_generate {
        KEYGEN_HOOK.with(|h| (h.borrow())(&config.secret, &config.base_dir.join("public.key")))?;
    } else {
        println!("✅ Existing keypair remains intact.");
    }

    // 2) Git initialization
    let vault_dir = config.base_dir.join("vault");
    let git_dir = vault_dir.join(".git");
    if !git_dir.exists() {
        let prompt = format!(
            "No Git repo found at `{}`. Initialize one?",
            vault_dir.display()
        );
        if CONFIRM_HOOK.with(|h| (h.borrow())(&prompt, true)) {
            fs::create_dir_all(&vault_dir).with_context(|| {
                format!("Failed to create vault directory {}", vault_dir.display())
            })?;
            println!("🔧 git init in {}…", vault_dir.display());

            GIT_HOOK.with(|g| (g.borrow())(&["init"], &vault_dir))?;
            let _ = GIT_HOOK.with(|g| (g.borrow())(&["branch", "-M", "main"], &vault_dir));
            let _ = GIT_HOOK.with(|g| {
                (g.borrow())(
                    &["commit", "--allow-empty", "-m", "Initial commit"],
                    &vault_dir,
                )
            });

            let remote_url = INPUT_HOOK.with(|i| {
                (i.borrow())("Enter GitHub remote URL (SSH or HTTPS), or leave blank to skip")
            });
            if !remote_url.trim().is_empty() {
                GIT_HOOK.with(|g| {
                    (g.borrow())(&["remote", "add", "origin", remote_url.trim()], &vault_dir)
                })?;
                println!("✅ remote 'origin' -> {}", remote_url.trim());

                println!("🚀 Pushing 'main' and setting upstream…");
                let push_status = GIT_HOOK.with(|g| {
                    (g.borrow())(&["push", "--set-upstream", "origin", "main"], &vault_dir)
                })?;
                if !push_status.success() {
                    eprintln!(
                        "⚠️  Push failed (exit code {}).",
                        push_status.code().unwrap_or(-1)
                    );
                    let reb_prompt =
                        "Remote contains commits you don’t have. Pull & rebase then retry push?";
                    if CONFIRM_HOOK.with(|h| (h.borrow())(reb_prompt, true)) {
                        println!("🔄 git pull --rebase origin main…");
                        let pull_status = GIT_HOOK.with(|g| {
                            (g.borrow())(&["pull", "--rebase", "origin", "main"], &vault_dir)
                        })?;
                        if pull_status.success() {
                            println!("✅ Pull/rebase succeeded. Retrying push…");
                            let retry_status = GIT_HOOK.with(|g| {
                                (g.borrow())(
                                    &["push", "--set-upstream", "origin", "main"],
                                    &vault_dir,
                                )
                            })?;
                            if retry_status.success() {
                                println!("✅ Successfully pushed after rebase.");
                            } else {
                                eprintln!(
                                    "⚠️  Retry push still failed (code {}).",
                                    retry_status.code().unwrap_or(-1)
                                );
                            }
                        } else {
                            eprintln!(
                                "⚠️  Pull/rebase failed (code {}).",
                                pull_status.code().unwrap_or(-1)
                            );
                        }
                    }
                }
            }
        } else {
            println!("ℹ️  Skipping Git initialization.");
        }
    } else {
        println!("✅ Git repository detected; continuing.");
    }

    // 3) Shell completions
    INSTALL_HOOK.with(|h| (h.borrow())())?;
    let shell = env::var("SHELL").unwrap_or_default();
    let rc = if shell.ends_with("bash") {
        "~/.bashrc"
    } else if shell.ends_with("zsh") {
        "~/.zshrc"
    } else {
        "~/.bashrc (or ~/.zshrc)"
    };
    println!(
        "\n🔄 New completions installed!\n\
         source {}\n\
         (or add that to your {})",
        rc, rc
    );

    Ok(())
}

// —— Hookable defaults —— //

type ConfirmFn = fn(&str, bool) -> bool;
type InputFn = fn(&str) -> String;
type KeygenFn = fn(&Path, &Path) -> Result<()>;
type InstallFn = fn() -> Result<()>;
type GitFn = fn(&[&str], &Path) -> Result<ExitStatus>;

fn default_confirm(prompt: &str, def: bool) -> bool {
    Confirm::new()
        .with_prompt(prompt)
        .default(def)
        .interact()
        .unwrap()
}
fn default_input(prompt: &str) -> String {
    Input::new()
        .with_prompt(prompt)
        .allow_empty(true)
        .interact_text()
        .unwrap()
}
fn default_keygen(secret: &Path, public: &Path) -> Result<()> {
    keygen::generate_keypair(secret, public)
}
fn default_install() -> Result<()> {
    completions::install()
}
fn default_git(args: &[&str], cwd: &Path) -> Result<ExitStatus> {
    Ok(std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()?)
}

thread_local! {
    static CONFIRM_HOOK: std::cell::RefCell<ConfirmFn> =
        std::cell::RefCell::new(default_confirm);
    static INPUT_HOOK:   std::cell::RefCell<InputFn>   =
        std::cell::RefCell::new(default_input);
    static KEYGEN_HOOK:  std::cell::RefCell<KeygenFn>  =
        std::cell::RefCell::new(default_keygen);
    static INSTALL_HOOK: std::cell::RefCell<InstallFn> =
        std::cell::RefCell::new(default_install);
    static GIT_HOOK:     std::cell::RefCell<GitFn>     =
        std::cell::RefCell::new(default_git);
}

#[cfg(test)]
pub fn set_confirm_hook(f: ConfirmFn) {
    CONFIRM_HOOK.with(|h| *h.borrow_mut() = f);
}
#[cfg(test)]
pub fn set_input_hook(f: InputFn) {
    INPUT_HOOK.with(|h| *h.borrow_mut() = f);
}
#[cfg(test)]
pub fn set_keygen_hook(f: KeygenFn) {
    KEYGEN_HOOK.with(|h| *h.borrow_mut() = f);
}
#[cfg(test)]
pub fn set_install_hook(f: InstallFn) {
    INSTALL_HOOK.with(|h| *h.borrow_mut() = f);
}
#[cfg(test)]
pub fn set_git_hook(f: GitFn) {
    GIT_HOOK.with(|h| *h.borrow_mut() = f);
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::cell::RefCell;
    use std::{
        cell::Cell, fs, os::unix::process::ExitStatusExt, path::PathBuf, process::ExitStatus,
    };
    use tempfile::TempDir;

    // helper to build a Config
    fn make_config(tmp: &TempDir) -> Config {
        Config {
            base_dir: tmp.path().to_path_buf(),
            prefix: PathBuf::new(),
            secret: tmp.path().join("secret.agekey"),
            crypto_extension: "rage".into(),
            public_key_filename: "public.key".into(),
        }
    }

    // thread-local storage for spy_git
    thread_local! {
        static GIT_CALLS:      RefCell<Vec<Vec<String>>> = RefCell::new(vec![]);
        static GIT_FAIL_AT:    Cell<Option<u32>>         = Cell::new(None);
        static GIT_CALL_COUNT: Cell<u32>                = Cell::new(0);
    }

    // A single spy for all git calls, with configurable failure point
    fn spy_git(args: &[&str], _cwd: &Path) -> Result<ExitStatus> {
        GIT_CALLS.with(|c| {
            c.borrow_mut()
                .push(args.iter().map(|s| s.to_string()).collect())
        });
        let n = GIT_CALL_COUNT.with(|c| {
            let v = c.get();
            c.set(v + 1);
            v + 1
        });
        let code = GIT_FAIL_AT.with(|f| if f.get() == Some(n) { 1 } else { 0 });
        Ok(ExitStatus::from_raw((code << 8) as i32))
    }
    fn set_git_fail_at(n: Option<u32>) {
        GIT_FAIL_AT.with(|c| c.set(n));
        GIT_CALL_COUNT.with(|c| c.set(0));
        GIT_CALLS.with(|c| c.borrow_mut().clear());
    }
    fn install_git_spy() {
        set_git_hook(spy_git);
        set_git_fail_at(None);
    }

    // simple spies for other hooks
    fn spy_keygen(_s: &Path, _p: &Path) -> Result<()> {
        Ok(())
    }
    fn spy_install() -> Result<()> {
        Ok(())
    }
    fn input_blank(_p: &str) -> String {
        "".into()
    }

    // stub_confirm uses thread-locals to decide replies
    thread_local! {
        static CONFIRM_INIT_REPO:   Cell<bool> = Cell::new(true);
        static CONFIRM_REBASE_RETRY:Cell<bool> = Cell::new(true);
    }
    fn stub_confirm(prompt: &str, default: bool) -> bool {
        if prompt.starts_with("No Git repo found") {
            CONFIRM_INIT_REPO.with(|c| c.get())
        } else if prompt.starts_with("Remote contains commits") {
            CONFIRM_REBASE_RETRY.with(|c| c.get())
        } else {
            default
        }
    }
    fn set_confirm_init_repo(val: bool) {
        CONFIRM_INIT_REPO.with(|c| c.set(val));
    }
    fn set_confirm_rebase_retry(val: bool) {
        CONFIRM_REBASE_RETRY.with(|c| c.set(val));
    }

    #[test]
    fn skip_git_init_when_user_says_no() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_config(&tmp);

        set_keygen_hook(spy_keygen);
        set_confirm_init_repo(false);
        set_confirm_rebase_retry(false);
        set_confirm_hook(stub_confirm);
        set_input_hook(input_blank);
        install_git_spy();
        set_install_hook(spy_install);

        run(&cfg)?;
        assert!(GIT_CALLS.with(|c| c.borrow().is_empty()));
        Ok(())
    }

    #[test]
    fn full_git_init_with_no_remote() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_config(&tmp);

        set_keygen_hook(spy_keygen);
        set_confirm_init_repo(true);
        set_confirm_rebase_retry(false);
        set_confirm_hook(stub_confirm);
        set_input_hook(input_blank);
        install_git_spy();
        set_install_hook(spy_install);

        run(&cfg)?;
        let calls = GIT_CALLS.with(|c| c.borrow().clone());
        assert_eq!(calls[0], vec!["init"]);
        assert_eq!(calls[1], vec!["branch", "-M", "main"]);
        assert_eq!(
            calls[2],
            vec!["commit", "--allow-empty", "-m", "Initial commit"]
        );
        assert_eq!(calls.len(), 3);
        Ok(())
    }

    #[test]
    fn git_init_with_remote_and_push_success() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_config(&tmp);

        set_keygen_hook(spy_keygen);
        set_confirm_init_repo(true);
        set_confirm_rebase_retry(false);
        set_confirm_hook(stub_confirm);
        set_input_hook(|_| "git@host:repo.git".into());
        install_git_spy();
        set_install_hook(spy_install);

        run(&cfg)?;
        let calls = GIT_CALLS.with(|c| c.borrow().clone());
        assert!(
            calls
                .iter()
                .any(|a| a == &vec!["remote", "add", "origin", "git@host:repo.git"])
        );
        assert!(
            calls
                .iter()
                .any(|a| a == &vec!["push", "--set-upstream", "origin", "main"])
        );
        Ok(())
    }

    #[test]
    fn git_init_push_fail_rebase_skipped() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_config(&tmp);

        set_keygen_hook(spy_keygen);
        set_confirm_init_repo(true);
        set_confirm_rebase_retry(false);
        set_confirm_hook(stub_confirm);
        set_input_hook(|_| "URL".into());
        install_git_spy();
        set_git_fail_at(Some(4));
        set_install_hook(spy_install);

        run(&cfg)?;
        let calls = GIT_CALLS.with(|c| c.borrow().clone());
        assert!(calls.iter().any(|a| a[0] == "push"));
        assert!(!calls.iter().any(|a| a[0] == "pull"));
        Ok(())
    }

    #[test]
    fn git_init_push_fail_rebase_and_retry_success() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_config(&tmp);

        set_keygen_hook(spy_keygen);
        set_confirm_init_repo(true);
        set_confirm_rebase_retry(true);
        set_confirm_hook(stub_confirm);
        set_input_hook(|_| "URL".into());
        install_git_spy();
        set_git_fail_at(Some(5));
        set_install_hook(spy_install);

        run(&cfg)?;
        let calls = GIT_CALLS.with(|c| c.borrow().clone());
        assert!(calls.iter().any(|a| a[0] == "pull"));
        assert!(calls.iter().filter(|a| a[0] == "push").count() >= 2);
        Ok(())
    }

    #[test]
    fn already_repo_skips_all() -> Result<()> {
        let tmp = TempDir::new()?;
        let cfg = make_config(&tmp);
        fs::create_dir_all(tmp.path().join("vault/.git"))?;

        set_keygen_hook(spy_keygen);
        // stub_confirm won't be called at all
        set_confirm_hook(stub_confirm);
        install_git_spy();
        set_install_hook(spy_install);

        run(&cfg)?;
        Ok(())
    }
}
