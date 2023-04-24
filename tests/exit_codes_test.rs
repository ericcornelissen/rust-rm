// SPDX-License-Identifier: Apache-2.0

//! Test suite focussed on testing exit codes exactly.

pub mod common;

use crate::common::{with_test_dir, TestResult};

#[test]
fn normal_success() -> TestResult {
    test_success(&[])
}

#[test]
fn help() -> TestResult {
    test_success(&["-h"])?;
    test_success(&["--help"])?;

    Ok(())
}

#[test]
fn file_not_found() -> TestResult {
    test_runtime_error(&["file"])
}

#[test]
fn invalid_flags() -> TestResult {
    test_usage_error(&["--not-a", "--real-flag"])
}

#[test]
fn dir_and_recursive() -> TestResult {
    test_usage_error(&["--dir", "--recursive"])
}

#[test]
fn force_and_interactive() -> TestResult {
    test_usage_error(&["--force", "--interactive"])
}

#[test]
fn quiet_and_verbose() -> TestResult {
    test_usage_error(&["--quiet", "--verbose"])
}

fn test_success(args: &[&str]) -> TestResult {
    with_test_dir(|mut cmd, _test_dir| {
        cmd.args(args).assert().code(0);

        Ok(())
    })
}

fn test_runtime_error(args: &[&str]) -> TestResult {
    with_test_dir(|mut cmd, _test_dir| {
        cmd.args(args).assert().code(1);

        Ok(())
    })
}

fn test_usage_error(args: &[&str]) -> TestResult {
    with_test_dir(|mut cmd, _test_dir| {
        cmd.args(args).assert().code(2);

        Ok(())
    })
}
