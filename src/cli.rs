//! CLI related functions, uses the clap argparsing definitions from `opts.rs`.

use std::{
    io,
    path::{Path, PathBuf},
    vec::Vec,
};

use clap::Parser;
use fs_err as fs;
use once_cell::sync::OnceCell;

use crate::{progress::ProgressBarPolicy, Opts, QuestionPolicy, Subcommand};

/// Whether to enable accessible output (removes info output and reduces other
/// output, removes visual markers like '[' and ']')
pub static ACCESSIBLE: OnceCell<bool> = OnceCell::new();

impl Opts {
    /// A helper method that calls `clap::Parser::parse`.
    ///
    /// And:
    ///   1. Make paths absolute.
    ///   2. Checks the QuestionPolicy.
    pub fn parse_args() -> crate::Result<(Self, QuestionPolicy, ProgressBarPolicy)> {
        let mut opts = Self::parse();

        ACCESSIBLE.set(opts.accessible).unwrap();

        let (Subcommand::Compress { files, .. }
        | Subcommand::Decompress { files, .. }
        | Subcommand::List { archives: files, .. }) = &mut opts.cmd;
        *files = canonicalize_files(files)?;

        let skip_questions_positively = if opts.yes {
            QuestionPolicy::AlwaysYes
        } else if opts.no {
            QuestionPolicy::AlwaysNo
        } else {
            QuestionPolicy::Ask
        };

        let progress_bar_policy =
            if opts.disable_progress_bar { ProgressBarPolicy::Disable } else { ProgressBarPolicy::Enable };

        Ok((opts, skip_questions_positively, progress_bar_policy))
    }
}

fn canonicalize_files(files: &[impl AsRef<Path>]) -> io::Result<Vec<PathBuf>> {
    files.iter().map(fs::canonicalize).collect()
}
