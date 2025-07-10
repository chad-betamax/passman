// src/cli.rs

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Do initial setup
    Init,

    /// Decrypt and display an entry (or dump runtime config)
    Show {
        /// Dump runtime configuration instead of decrypting an entry
        #[clap(long)]
        config: bool,

        /// Entry name (e.g., example.com)
        #[clap(value_name = "PATH", required_unless_present = "config")]
        path: Option<String>,

        /// Copy to clipboard (not implemented yet)
        #[clap(short, long)]
        clip: bool,

        /// Show file as QR code
        #[clap(long)]
        qr: bool,

        /// Which line to show (1-based). If omitted, prints all lines.
        #[clap(long, hide = true)]
        line: Option<usize>,
    },

    /// Insert a new password entry
    New {
        /// Entry name (e.g., example.com)
        path: String,

        /// Prompt for single-line input instead of opening $EDITOR
        #[clap(short, long)]
        prompt: bool,

        /// Echo input to screen (not implemented yet)
        #[clap(short, long)]
        echo: bool,

        /// Overwrite existing entry
        #[clap(short, long)]
        force: bool,
    },

    /// Edit an existing password file
    Edit {
        /// Entry name (e.g., example.com)
        path: String,
    },

    /// Display vault contents in a tree structure
    List {
        /// Optional sub-path within the vault
        #[clap()]
        path: Option<String>,
    },

    /// Hide a password file from the list command
    Archive {
        /// Path to the file you want to archive
        path: String,
    },

    /// Delete a file (local & github)
    Remove {
        /// Entry name (e.g., example.com)
        path: String,
    },
}
