// SPDX-License-Identifier: Apache-2.0

//! Test suite focussed on testing the functionality of the `--trash`/`-t` option.
//!
//! These tests only run when the "test-trash" feature is enabled. You can use the following command
//! to run these tests: `cargo test --features test-trash`

pub mod common;

use crate::common::{has_exactly_lines, rm_out, TestResult};

use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
#[cfg(feature = "trash")]
#[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
fn file() -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.arg(filename)
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_trashed(filename);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0),
            ))
            .stderr("");
        file.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::trashed(filename);
                rm_out::newline(),
                rm_out::conclusion(1, 0),
            ))
            .stderr("");
        file.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg(feature = "trash")]
#[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
fn empty_directory() -> TestResult {
    let dirname = "dir";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;

        cmd.arg(dirname)
            .assert()
            .failure()
            .stdout(has_exactly_lines!(
                ;
                rm_out::newline(),
                rm_out::dry_conclusion(0, 1),
            ))
            .stderr(rm_out::is_a_dir(dirname));
        dir.assert(predicate::path::exists());

        cmd.arg("--dir")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_trashed(dirname);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::trashed(dirname);
                rm_out::newline(),
                rm_out::conclusion(1, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg(feature = "trash")]
#[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
fn filled_directory() -> TestResult {
    let dirname = "dir";
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;
        let file = dir.child(filename);
        file.touch()?;

        cmd.arg(dirname)
            .assert()
            .failure()
            .stdout(has_exactly_lines!(
                ;
                rm_out::newline(),
                rm_out::dry_conclusion(0, 1),
            ))
            .stderr(rm_out::is_a_dir(dirname));
        dir.assert(predicate::path::exists());
        file.assert(predicate::path::exists());

        cmd.arg("--recursive")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_trashed(dirname);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::exists());
        file.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::trashed(dirname);
                rm_out::newline(),
                rm_out::conclusion(1, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg(feature = "trash")]
#[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
fn link() -> TestResult {
    let linkname = "link";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child("linked_file");
        file.touch()?;
        let link = test_dir.child(linkname);
        link.symlink_to_file(&file)?;

        cmd.arg(linkname)
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_trashed(linkname);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0),
            ))
            .stderr("");
        file.assert(predicate::path::exists());
        link.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::trashed(linkname);
                rm_out::newline(),
                rm_out::conclusion(1, 0),
            ))
            .stderr("");
        file.assert(predicate::path::exists());
        link.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg(not(feature = "trash"))]
fn trash_not_supported_without_the_build_feature() -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child("file");
        file.touch()?;

        cmd.arg(filename).assert().failure();
        file.assert(predicate::path::exists());

        Ok(())
    })
}

/// Run a test with `--trash` enabled.
///
/// See also [`common::with_test_dir`].
fn with_test_dir<C>(callback: C) -> TestResult
where
    C: FnOnce(assert_cmd::Command, &assert_fs::TempDir) -> TestResult,
{
    common::with_test_dir(|mut cmd, test_dir| {
        cmd.arg("--trash");
        callback(cmd, test_dir)
    })
}
