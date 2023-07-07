// SPDX-License-Identifier: Apache-2.0

//! Test suite focussed on testing the functionality of the `--recursive`/`-r` option.

pub mod common;

use crate::common::{has_exactly_lines, rm_out, TestResult};

use std::path::MAIN_SEPARATOR;

use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn file() -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.arg(filename)
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(filename);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0),
            ))
            .stderr("");
        file.assert(predicate::path::exists());

        cmd.arg("--force")
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

#[test]
fn empty_dir() -> TestResult {
    let dirname = "dir";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;

        cmd.arg(dirname)
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(dirname);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(dirname);
                rm_out::newline(),
                rm_out::conclusion(1, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn filled_dir() -> TestResult {
    let dirname = "dir";
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;
        let nested_file = dir.child(filename);
        nested_file.touch()?;

        cmd.arg(dirname)
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(format!("{dirname}{MAIN_SEPARATOR}{filename}")),
                rm_out::dry_removed(dirname);
                rm_out::newline(),
                rm_out::dry_conclusion(2, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::exists());
        nested_file.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(format!("{dirname}{MAIN_SEPARATOR}{filename}")),
                rm_out::removed(dirname);
                rm_out::newline(),
                rm_out::conclusion(2, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn nested_dir() -> TestResult {
    let dirname = "dir";
    let filename = "file1";
    let nested_dirname = "nested_dir";
    let nested_filename = "file2";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        let nested_dir = dir.child(nested_dirname);
        nested_dir.create_dir_all()?;

        let dir_file = dir.child(filename);
        dir_file.touch()?;
        let nested_dir_file = nested_dir.child(nested_filename);
        nested_dir_file.touch()?;

        cmd.arg(dirname)
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(format!("{dirname}{MAIN_SEPARATOR}{nested_dirname}{MAIN_SEPARATOR}{nested_filename}")),
                rm_out::dry_removed(format!("{dirname}{MAIN_SEPARATOR}{nested_dirname}")),
                rm_out::dry_removed(format!("{dirname}{MAIN_SEPARATOR}{filename}")),
                rm_out::dry_removed(dirname);
                rm_out::newline(),
                rm_out::dry_conclusion(4, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::exists());
        dir_file.assert(predicate::path::exists());
        nested_dir.assert(predicate::path::exists());
        nested_dir_file.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(format!("{dirname}{MAIN_SEPARATOR}{nested_dirname}{MAIN_SEPARATOR}{nested_filename}")),
                rm_out::removed(format!("{dirname}{MAIN_SEPARATOR}{nested_dirname}")),
                rm_out::removed(format!("{dirname}{MAIN_SEPARATOR}{filename}")),
                rm_out::removed(dirname);
                rm_out::newline(),
                rm_out::conclusion(4, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg_attr(
    all(windows, not(feature = "test-symlink")),
    ignore = "Only run with the test-symlink feature"
)]
fn symlink_to_file() -> TestResult {
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
                rm_out::dry_removed(linkname);
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
                rm_out::removed(linkname);
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
#[cfg_attr(windows, ignore = "TODO: investigate symlink test errors on Windows")]
fn symlink_to_empty_dir() -> TestResult {
    let linkname = "link";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child("linked_dir");
        dir.create_dir_all()?;
        let link = test_dir.child(linkname);
        link.symlink_to_dir(&dir)?;

        cmd.arg(linkname)
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(linkname);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::exists());
        link.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(linkname);
                rm_out::newline(),
                rm_out::conclusion(1, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::exists());
        link.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg_attr(windows, ignore = "TODO: investigate symlink test errors on Windows")]
fn symlink_to_filled_dir() -> TestResult {
    let linkname = "link";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child("linked_dir");
        dir.create_dir_all()?;
        let nested_file = dir.child("file");
        nested_file.touch()?;
        let link = test_dir.child(linkname);
        link.symlink_to_dir(&dir)?;

        cmd.arg(linkname)
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(linkname);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0),
            ))
            .stderr("");
        link.assert(predicate::path::exists());
        nested_file.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(linkname);
                rm_out::newline(),
                rm_out::conclusion(1, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::exists());
        link.assert(predicate::path::missing());
        nested_file.assert(predicate::path::exists());

        Ok(())
    })
}

/// Run a test with `--dir` and `--recursive` enabled.
///
/// See also [`common::with_test_dir`].
fn with_test_dir<C>(callback: C) -> TestResult
where
    C: FnOnce(assert_cmd::Command, &assert_fs::TempDir) -> TestResult,
{
    common::with_test_dir(|mut cmd, test_dir| {
        cmd.arg("--recursive");
        callback(cmd, test_dir)
    })
}
