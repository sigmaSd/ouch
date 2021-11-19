//! CLI related functions, uses the clap argparsing definitions from `opts.rs`.

use std::{
    io,
    path::{Path, PathBuf},
    vec::Vec,
};

use clap::Parser;
use fs_err as fs;

use crate::{Opts, QuestionPolicy, Subcommand};

/// Enable/Disable the progress bar.
#[derive(Debug, Clone, Copy)]
pub enum ProgressBarPolicy {
    /// Disable the progress bar.
    Disable,
    /// Enable the progress bar.
    Enable,
}
impl ProgressBarPolicy {
    /// Returns `true` if the progress bar is enabled.
    pub fn is_enabled(self) -> bool {
        match self {
            ProgressBarPolicy::Enable => true,
            ProgressBarPolicy::Disable => false,
        }
    }
}

impl Opts {
    /// A helper method that calls `clap::Parser::parse`.
    ///
    /// And:
    ///   1. Make paths absolute.
    ///   2. Checks the QuestionPolicy.
    pub fn parse_args() -> crate::Result<(Self, QuestionPolicy, ProgressBarPolicy)> {
        let mut opts = Self::parse();

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
