// SPDX-License-Identifier: Apache-2.0

//! Test suite focussed on testing that no-color options suppress ansi-formatting in stdout/stderr.

pub mod common;

use crate::common::TestResult;

use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn help() -> TestResult {
    with_test_dir(|mut cmd, _test_dir| {
        cmd.arg("--help").assert().stdout(no_ansi()).stderr(no_ansi());

        Ok(())
    })
}

#[test]
fn run_success() -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        test_dir.child(filename).touch()?;

        cmd.arg(filename).assert().success().stdout(no_ansi()).stderr(no_ansi());

        Ok(())
    })
}

#[test]
fn run_error() -> TestResult {
    with_test_dir(|mut cmd, _test_dir| {
        cmd.arg("missing").assert().failure().stdout(no_ansi()).stderr(no_ansi());

        Ok(())
    })
}

/// Run a test with the `NO_COLOR` environment variable set.
///
/// See also [`common::with_test_dir`].
fn with_test_dir<C>(callback: C) -> TestResult
where
    C: FnOnce(assert_cmd::Command, &assert_fs::TempDir) -> TestResult,
{
    common::with_test_dir(|mut cmd, test_dir| {
        cmd.env("NO_COLOR", "1");
        callback(cmd, test_dir)
    })
}

/// Create a predicate that evaluates to `true` if a given string does not contain ANSI characters.
///
/// The implementation is based on <https://github.com/TheLarkInn/ansi-regex> (MIT license).
#[allow(clippy::unwrap_used)]
fn no_ansi() -> predicates::boolean::NotPredicate<predicates::str::RegexPredicate, str> {
    predicates::str::is_match(r"\x1b\[([\x30-\x3f]*[\x20-\x2f]*[\x40-\x7e])").unwrap().not()
}
