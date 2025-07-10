use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Do initial setup
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

        /// Which line to show (1-based). If omitted, prints all lines.
        #[arg(long)]
        line: Option<usize>,
    },

    /// Insert a new password entry
    New {
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
    /// Delete a file (local & github)
    Remove { path: String },
}
