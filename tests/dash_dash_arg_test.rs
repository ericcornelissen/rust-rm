//! Test suite focussed on testing the behavior of the special '--' option.

pub mod common;

use crate::common::{has_exactly_lines, rm_out, with_test_dir, TestResult};

use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn short_flag() -> TestResult {
    test_ignore_flag("-f")
}

#[test]
fn long_flag() -> TestResult {
    test_ignore_flag("--force")
}

#[test]
fn filename_is_one_dash() -> TestResult {
    test_remove_file("-")
}

#[test]
fn filename_is_two_dashes() -> TestResult {
    test_remove_file("--")
}

#[test]
fn filename_with_a_leading_dash() -> TestResult {
    test_remove_file("-file")
}

#[test]
fn filename_with_leading_dashes() -> TestResult {
    test_remove_file("--file")
}

/// Test that the given flag is ignored when placed after the special '--' option.
///
/// # Errors
///
/// If the test fails.
///
/// # Examples
///
/// ```no_run
/// use crate::common::TestResult;
///
/// #[test]
/// fn example_test() -> TestResult {
///     test_ignore_flag("--force")
/// }
/// ```
fn test_ignore_flag(flag: &str) -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.args([filename, "--", flag])
            .assert()
            .failure()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(filename),
                rm_out::newline(),
                rm_out::dry_conclusion(1, 1),
            ))
            .stderr(has_exactly_lines!(rm_out::not_found(flag),));
        file.assert(predicate::path::exists());

        Ok(())
    })
}

/// Test removing a file with the given name by placing it after the special '--' option.
///
/// # Errors
///
/// If the test fails.
///
/// # Examples
///
/// ```no_run
/// use crate::common::TestResult;
///
/// #[test]
/// fn example_test() -> TestResult {
///     test_remove_file("filename")
/// }
/// ```
fn test_remove_file(filename: &str) -> TestResult {
    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.args(["--force", "--", filename])
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(filename);
                rm_out::newline(),
                rm_out::conclusion(1, 0),

            ))
            .stderr("");
        file.assert(predicate::path::missing());

        Ok(())
    })
}
