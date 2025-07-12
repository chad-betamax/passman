use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create a new password entry
    New {
        /// /path/to/passwordfile
        path: String,
    },

    /// Display a decrypted entry (or dump runtime config)
    Show {
        /// Dump runtime configuration instead of decrypting an entry
        #[clap(long)]
        config: bool,

        /// /path/to/passwordfile
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
        /// /path/to/passwordfile
        path: String,
    },

    /// Display vault contents in a tree structure
    List {
        /// /path/to/passwordfile
        #[clap()]
        path: Option<String>,
        /// Also include archived (dot-prefixed) entries
        #[clap(long)]
        all: bool,
    },

    /// Hide a password file from the list command
    Archive {
        /// /path/to/thing/tobearchived
        path: String,

        /// Archive entire folder
        #[clap(long)]
        folder: bool,
    },

    /// Delete a file (local & remote)
    Remove {
        /// /path/to/passwordfile
        path: String,
    },

    /// Do initial setup
    Init,
}
