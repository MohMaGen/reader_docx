use std::path::PathBuf;

use argp::FromArgs;

/// Simple program to read and edit docx files.
#[derive(FromArgs)]
pub struct ReaderDoc {
    /// File to open
    #[argp(option, short = 'i')]
    pub input: Option<PathBuf>,
}
