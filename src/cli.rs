use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Generate a new rage keypair and store in ~/.passman
    Init,

    /// Decrypt and display a stored password
    Show {
        /// Entry name (e.g., example.com)
        path: String,

        /// Copy to clipboard (not implemented yet)
        #[arg(short, long)]
        clip: bool,

        /// Show as QR code
        #[arg(long)]
        qr: bool,

        /// Line number to extract (default: 1)
        #[arg(long, default_value_t = 1)]
        line: usize,
    },

    /// Insert a new password entry
    Insert {
        /// Entry name (e.g., example.com)
        path: String,

        /// Prompt for single-line input instead of opening $EDITOR
        #[arg(short, long)]
        prompt: bool,

        /// Echo input to screen (not implemented yet)
        #[arg(short, long)]
        echo: bool,

        /// Overwrite existing entry
        #[arg(short, long)]
        force: bool,
    },

    /// Display vault contents in a tree structure
    List {
        #[arg()]
        path: Option<String>,
    },
    Remove {
        /// Entry name to delete
        path: String,
    },
}
