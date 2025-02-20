// SPDX-License-Identifier: Apache-2.0

//! Test suite focussed on testing the functionality of GNU mode.

pub mod common;

use crate::common::{TestResult, rm_ask, rm_out};

use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
#[cfg(feature = "gnu-mode")]
fn remove_file() -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.arg(filename).assert().success();
        file.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg(feature = "gnu-mode")]
fn remove_empty_dir() -> TestResult {
    let dirname = "dir";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;

        cmd.arg(dirname).assert().failure().stdout("").stderr(rm_out::is_a_dir(dirname));
        dir.assert(predicate::path::exists());

        cmd.arg("--dir").assert().success();
        dir.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg(feature = "gnu-mode")]
fn remove_empty_dir_recursively() -> TestResult {
    let dirname = "dir";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;

        cmd.arg(dirname).assert().failure().stdout("").stderr(rm_out::is_a_dir(dirname));
        dir.assert(predicate::path::exists());

        cmd.arg("--recursive").assert().success();
        dir.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg(feature = "gnu-mode")]
fn remove_filled_dir_recursively() -> TestResult {
    let dirname = "dir";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;
        let file = dir.child("file");
        file.touch()?;

        cmd.arg(dirname).assert().failure().stdout("").stderr(rm_out::is_a_dir(dirname));
        dir.assert(predicate::path::exists());
        file.assert(predicate::path::exists());

        cmd.arg("--recursive").assert().success();
        dir.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg(feature = "gnu-mode")]
#[cfg_attr(
    all(windows, not(feature = "test-symlink")),
    ignore = "Only run with the test-symlink feature"
)]
fn remove_symlink() -> TestResult {
    let linkname = "link";

    with_test_dir(|mut cmd, test_dir| {
        let linked_file = test_dir.child("linked_file");
        linked_file.touch()?;
        let link = test_dir.child(linkname);
        link.symlink_to_file(&linked_file)?;

        cmd.arg(linkname).assert().success();
        linked_file.assert(predicate::path::exists());
        link.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg(feature = "gnu-mode")]
fn remove_missing() -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, _test_dir| {
        cmd.arg(filename).assert().failure().stdout("").stderr(rm_out::not_found(filename));

        Ok(())
    })
}

#[test]
#[cfg(feature = "gnu-mode")]
fn remove_missing_with_force() -> TestResult {
    with_test_dir(|mut cmd, _test_dir| {
        cmd.args(["--force", "file"]).assert().success();

        Ok(())
    })
}

#[test]
#[cfg(feature = "gnu-mode")]
fn stdout_and_stderr_on_success() -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.arg(filename).assert().success().stdout("").stderr("");

        Ok(())
    })
}

#[test]
#[cfg(feature = "gnu-mode")]
fn interactive_no() -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.args(["--interactive", filename])
            .write_stdin("n")
            .assert()
            .success()
            .stdout("")
            .stderr(rm_ask::file(filename));
        file.assert(predicate::path::exists());

        Ok(())
    })
}

#[test]
#[cfg(feature = "gnu-mode")]
fn interactive_yes() -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.args(["--interactive", filename])
            .write_stdin("Y")
            .assert()
            .success()
            .stdout("")
            .stderr(rm_ask::file(filename));
        file.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg(feature = "gnu-mode")]
fn invalid_flag_blind() -> TestResult {
    unsupported_flag("--blind")?;
    unsupported_flag_with_force("--blind")?;

    Ok(())
}

#[test]
#[cfg(feature = "gnu-mode")]
fn invalid_flag_quiet() -> TestResult {
    unsupported_flag("--quiet")?;
    unsupported_flag_with_force("--quiet")?;

    Ok(())
}

#[test]
#[cfg(all(feature = "gnu-mode", feature = "trash"))]
fn invalid_flag_trash() -> TestResult {
    unsupported_flag("--trash")?;
    unsupported_flag_with_force("--trash")?;

    Ok(())
}

#[test]
#[cfg(not(feature = "gnu-mode"))]
fn ignored_without_the_build_feature() -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.arg(filename).assert().success();
        file.assert(predicate::path::exists());

        cmd.arg("--force").assert().success();
        file.assert(predicate::path::missing());

        Ok(())
    })
}

/// Test the behavior of using an unsupported flag in GNU mode (without `--force`).
///
/// # Example
///
/// ```no_run
/// unsupported_flag("--flag");
/// ```
#[cfg(feature = "gnu-mode")]
fn unsupported_flag(flag: &str) -> TestResult {
    with_test_dir(|mut cmd, _test_dir| {
        cmd.arg(flag)
            .assert()
            .failure()
            .stdout("")
            .stderr(format!("error: option {flag} not supported in GNU mode\n"));

        Ok(())
    })
}

/// Test the behavior of using an unsupported flag in GNU mode with `--force`.
///
/// # Example
///
/// ```no_run
/// unsupported_flag_with_force("--flag");
/// ```
#[cfg(feature = "gnu-mode")]
fn unsupported_flag_with_force(flag: &str) -> TestResult {
    with_test_dir(|mut cmd, _test_dir| {
        cmd.args(["--force", flag]).assert().success();

        Ok(())
    })
}

/// Run a test with GNU mode enabled.
///
/// See also [`common::with_test_dir`].
fn with_test_dir<C>(callback: C) -> TestResult
where
    C: FnOnce(assert_cmd::Command, &assert_fs::TempDir) -> TestResult,
{
    common::with_test_dir(|mut cmd, test_dir| {
        cmd.env("RUST_RM_GNU_MODE", "1");
        callback(cmd, test_dir)
    })
}
