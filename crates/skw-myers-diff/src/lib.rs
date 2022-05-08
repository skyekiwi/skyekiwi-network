mod types;
mod parse;
mod myers;

pub use types::{DiffOp};
pub use parse::{diff_ops_to_bytes, bytes_to_diff_ops};
pub use myers::{diff, patch};
