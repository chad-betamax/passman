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
    /// Insert a new password entry
    New {
        /// Entry name (e.g., example.com)
        path: String,
    },

    /// Display the decrypted contents of an entry (or dump runtime config)
    Show {
        /// Dump runtime configuration instead of decrypting an entry
        #[clap(long)]
        config: bool,

        /// Entry name (e.g., example.com)
        #[clap(value_name = "PATH", required_unless_present = "config")]
        path: Option<String>,

        /// Show file as QR code
        #[clap(long)]
        qr: bool,

        /// Which line to show. If omitted, prints all lines.
        #[clap(long, hide = true)]
        line: Option<usize>,
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
        /// Also include archived (dot-prefixed) entries
        #[clap(long)]
        all: bool,
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

    /// Do initial setup
    Init,
}
