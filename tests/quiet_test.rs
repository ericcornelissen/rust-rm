// SPDX-License-Identifier: Apache-2.0

//! Test suite focussed on testing the functionality of the `--quiet`/`-q` option.

pub mod common;

use crate::common::{TestResult, has_exactly_lines, rm_out};

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

        cmd.arg("--force").assert().success().stdout("").stderr("");
        file.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg_attr(
    all(windows, not(feature = "test-symlink")),
    ignore = "Only run with the test-symlink feature"
)]
fn symlink() -> TestResult {
    let linkname = "link";

    with_test_dir(|mut cmd, test_dir| {
        let linked_file = test_dir.child("linked_file");
        linked_file.touch()?;
        let link = test_dir.child(linkname);
        link.symlink_to_file(&linked_file)?;

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
        linked_file.assert(predicate::path::exists());

        cmd.arg("--force").assert().success().stdout("").stderr("");
        link.assert(predicate::path::missing());
        linked_file.assert(predicate::path::exists());

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
                rm_out::dry_removed(dirname);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::exists());

        cmd.arg("--force").assert().success().stdout("").stderr("");
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
                rm_out::dry_removed(format!("{dirname}{MAIN_SEPARATOR}{filename}")),
                rm_out::dry_removed(dirname);
                rm_out::newline(),
                rm_out::dry_conclusion(2, 0),
            ))
            .stderr("");
        dir.assert(predicate::path::exists());
        file.assert(predicate::path::exists());

        cmd.arg("--force").assert().success().stdout("").stderr("");
        dir.assert(predicate::path::missing());

        Ok(())
    })
}

/// Run a test with `--quiet` enabled.
///
/// See also [`common::with_test_dir`].
fn with_test_dir<C>(callback: C) -> TestResult
where
    C: FnOnce(assert_cmd::Command, &assert_fs::TempDir) -> TestResult,
{
    common::with_test_dir(|mut cmd, test_dir| {
        cmd.arg("--quiet");
        callback(cmd, test_dir)
    })
}
