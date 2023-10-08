// SPDX-License-Identifier: Apache-2.0

//! Utility functions for writing integration tests for this project.

use std::env;
use std::error;

use assert_cmd::Command;
use assert_fs::TempDir;

/// Create a predicate that verifies a `str` contains all and only the provided lines.
///
/// # Examples
///
/// Basic usage:
///
/// ```no_run
/// pub mod common;
///
/// use crate::common::has_exactly_lines;
///
/// use predicates::prelude::*;
///
/// #[test]
/// fn basic_test() -> TestResult {
///     let test_str = "\
///         line1\n\
///         line2\n\
///         line3\n\
///     ";
///
///     assert_eq!(true, has_exactly_lines!("line1\n", "line2\n", "line3\n").eval(test_str));
///     assert_eq!(false, has_exactly_lines!("line1\n", "line2\n").eval(test_str));
/// }
/// ```
///
/// Assert the content of trailing lines by using a semicolon as delimiter:
///
/// ```no_run
/// pub mod common;
///
/// use crate::common::has_exactly_lines;
///
/// use predicates::prelude::*;
///
/// #[test]
/// fn advanced_test() -> TestResult {
///     let test_str = "\
///         line1\n\
///         line2\n\
///         line3\n\
///     ";
///
///     assert_eq!(true, has_exactly_lines!("line1\n", "line2\n"; "line3\n").eval(test_str));
///     assert_eq!(false, has_exactly_lines!("line1\n", "line3\n"; "line2\n").eval(test_str));
/// }
/// ```
#[allow(unused_macros)]
macro_rules! has_exactly_lines {
    ($($line:expr),* $(,)?) => {
        // Base predicate.
        predicates::str::contains("").normalize()
        // Contains all strings ...
        $( .and(predicates::str::contains($line)) )*
        // ... and has the same length as all strings taken together ...
        .and(predicates::function::function(|s: &str| {
            s.len() == [$( $line, )*].join("").len()
        }))
        // ... means it only has these lines.
    };
    ($($line:expr),* ; $($last_line:expr),* $(,)?) => {
        has_exactly_lines!($( $line, )* $( $last_line, )*).and(
            predicates::str::ends_with([$( $last_line, )*].join(""))
        )
    };
}

/// Create a predicate that verifies a `str` contains all the provided lines, but maybe others as
/// well.
///
/// # Examples
///
/// Basic usage:
///
/// ```no_run
/// pub mod common;
///
/// use crate::common::has_lines;
///
/// use predicates::prelude::*;
///
/// #[test]
/// fn basic_test() -> TestResult {
///     let test_str = "\
///         line1\n\
///         line2\n\
///         line3\n\
///     ";
///
///     assert_eq!(true, has_lines!("line1\n", "line2\n").eval(test_str));
///     assert_eq!(false, has_lines!("line1\n", "line3\n").eval(test_str));
/// }
/// ```
///
/// Assert the content of trailing lines by using a semicolon as delimiter:
///
/// ```no_run
/// pub mod common;
///
/// use crate::common::has_lines;
///
/// use predicates::prelude::*;
///
/// #[test]
/// fn advanced_test() -> TestResult {
///     let test_str = "\
///         line1\n\
///         line2\n\
///         line3\n\
///     ";
///
///     assert_eq!(true, has_lines!("line1\n"; "line3\n").eval(test_str));
///     assert_eq!(false, has_lines!("line1\n"; "line2\n").eval(test_str));
/// }
/// ```
#[allow(unused_macros)]
macro_rules! has_lines {
    ($($line:expr),* $(,)?) => {
        // Base predicate.
        predicates::str::contains("").normalize()
        // Contains all strings ...
        $( .and(predicates::str::contains($line)) )*
    };
    ($($line:expr),* ; $($last_line:expr),* $(,)?) => {
        has_lines!($( $line, )* $( $last_line, )*).and(
            predicates::str::ends_with([$( $last_line, )*].join(""))
        )
    };
}

#[allow(unused_imports)]
pub(crate) use {has_exactly_lines, has_lines};

/// Test helpers to generate strings used by the CLI to prompt the user.
///
/// # Examples
///
/// ```no_run
/// pub mod common;
///
/// use crate::common::rm_ask;
///
/// #[test]
/// fn advanced_test() -> TestResult {
///     let stdout = "Removed file\n";
///     assert_eq!(stdout, rm_ask::file("file"))
/// }
/// ```
pub mod rm_ask {
    pub fn descend<S: Into<String>>(subject: S) -> String {
        format!("Descend into directory {}? [Y/n] ", subject.into())
    }

    pub fn dir<S: Into<String>>(subject: S) -> String {
        format!("Remove directory {}? [Y/n] ", subject.into())
    }

    pub fn empty_dir<S: Into<String>>(subject: S) -> String {
        format!("Remove empty directory {}? [Y/n] ", subject.into())
    }

    pub fn file<S: Into<String>>(subject: S) -> String {
        format!("Remove regular file {}? [Y/n] ", subject.into())
    }

    pub fn link<S: Into<String>>(subject: S) -> String {
        format!("Remove symbolic link {}? [Y/n] ", subject.into())
    }
}

/// Test helpers to generate strings outputted by the CLI.
///
/// # Examples
///
/// ```no_run
/// pub mod common;
///
/// use crate::common::rm_out;
///
/// #[test]
/// fn advanced_test() -> TestResult {
///     let stdout = "Removed file\n";
///     assert_eq!(stdout, rm_out::removed("file"))
/// }
/// ```
pub mod rm_out {
    #[must_use]
    pub fn conclusion(removed: usize, errored: usize) -> String {
        format!(
            "{removed} removed, {errored} {} occurred\n",
            if errored == 1 { "error" } else { "errors" }
        )
    }

    pub fn dir_not_empty<S: Into<String>>(subject: S) -> String {
        format!(
            "Cannot remove {}: Directory not empty (use '--recursive' to remove)\n",
            subject.into()
        )
    }

    pub fn dir_not_empty_no_tip<S: Into<String>>(subject: S) -> String {
        format!("Cannot remove {}: Directory not empty\n", subject.into())
    }

    #[must_use]
    pub fn dry_conclusion(removed: usize, errored: usize) -> String {
        format!(
            "{removed} would be removed{}, {errored} {} occurred\n",
            if removed > 0 { " (use '--force' to remove)" } else { "" },
            if errored == 1 { "error" } else { "errors" },
        )
    }

    pub fn dry_removed<S: Into<String>>(subject: S) -> String {
        format!("Would remove {}\n", subject.into())
    }

    pub fn dry_trashed<S: Into<String>>(subject: S) -> String {
        format!("Would move {} to trash\n", subject.into())
    }

    pub fn found_dir<S: Into<String>>(subject: S) -> String {
        format!("[found directory at {}]\n", subject.into())
    }

    pub fn found_file<S: Into<String>>(subject: S) -> String {
        format!("[found file at {}]\n", subject.into())
    }

    pub fn found_link<S: Into<String>>(subject: S) -> String {
        format!("[found symbolic link at {}]\n", subject.into())
    }

    pub fn found_nothing<S: Into<String>>(subject: S) -> String {
        format!("[found nothing at {}]\n", subject.into())
    }

    pub fn is_a_dir<S: Into<String>>(subject: S) -> String {
        format!("Cannot remove {}: Is a directory (use '--dir' to remove)\n", subject.into())
    }

    #[must_use]
    pub fn newline() -> String {
        "\n".to_owned()
    }

    pub fn not_found<S: Into<String>>(subject: S) -> String {
        format!("Cannot remove {}: Not found (use '--blind' to ignore)\n", subject.into())
    }

    pub fn not_found_no_tip<S: Into<String>>(subject: S) -> String {
        format!("Cannot remove {}: Not found\n", subject.into())
    }

    pub fn refused<S: Into<String>>(subject: S) -> String {
        format!("Cannot remove {}: Refused to remove\n", subject.into())
    }

    pub fn removed<S: Into<String>>(subject: S) -> String {
        format!("Removed {}\n", subject.into())
    }

    pub fn skipped_empty<S: Into<String>>(subject: S) -> String {
        format!("[skipped {}: Directory is empty]\n", subject.into())
    }

    pub fn skipped_invalid_input<S: Into<String>>(subject: S) -> String {
        format!("[skipped {}: Unrecognized input]\n", subject.into())
    }

    pub fn skipped_kept<S: Into<String>>(subject: S) -> String {
        format!("[skipped {}: Kept by user]\n", subject.into())
    }

    pub fn skipped_not_found<S: Into<String>>(subject: S) -> String {
        format!("[skipped {}: Not found]\n", subject.into())
    }

    #[must_use]
    pub fn start() -> String {
        "[start processing]\n".to_owned()
    }

    pub fn trashed<S: Into<String>>(subject: S) -> String {
        format!("Moved {} to trash\n", subject.into())
    }
}

/// The environment variable name to enable debugging mode for tests.
const TEST_DEBUG_MODE: &str = "RUST_RM_DEBUG_TEST";

/// The `Result` type used by [`with_test_dir`].
pub type TestResult = Result<(), Box<dyn error::Error>>;

/// Run a test with access to a (temporary) testing directory.
///
/// # Errors
///
/// Any error returned by the test callback is returned by this function.
///
/// An error may also occur if the test could not be set up.
///
/// # Examples
///
/// ```no_run
/// pub mod common;
///
/// use crate::common::{with_test_dir, TestResult};
///
/// use assert_fs::prelude::*;
///
/// #[test]
/// fn example_test() -> TestResult {
///     with_test_dir(|mut cmd, test_dir| {
///         // Test something using `test_dir` ...
///
///         Ok(())
///     })
/// }
/// ```
pub fn with_test_dir<C>(callback: C) -> TestResult
where
    C: FnOnce(Command, &TempDir) -> TestResult,
{
    let debug = env::var_os(TEST_DEBUG_MODE).is_some();
    let temp_dir = TempDir::new()?.into_persistent_if(debug);

    let mut cmd = Command::cargo_bin("rust-rm")?;
    cmd.current_dir(&temp_dir);

    callback(cmd, &temp_dir)
}
