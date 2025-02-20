// SPDX-License-Identifier: Apache-2.0

//! Test suite focussed on testing the functionality of the `--blind`/`-b` option.

pub mod common;

use crate::common::{TestResult, has_exactly_lines, has_lines, rm_out};

use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn no_paths_exist() -> TestResult {
    let missing_path1 = "missing1";
    let missing_path2 = "missing2";

    with_test_dir(|mut cmd, _test_dir| {
        cmd.args([missing_path1, missing_path2])
            .assert()
            .success()
            .stdout(rm_out::dry_conclusion(0, 0))
            .stderr("");

        cmd.arg("--verbose")
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::skipped_not_found(missing_path1),
                rm_out::skipped_not_found(missing_path2);
                rm_out::newline(),
                rm_out::dry_conclusion(0, 0),
            ))
            .stderr("");

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::skipped_not_found(missing_path1),
                rm_out::skipped_not_found(missing_path2);
                rm_out::newline(),
                rm_out::conclusion(0, 0),
            ))
            .stderr("");

        Ok(())
    })
}

#[test]
fn some_paths_exist() -> TestResult {
    let filename1 = "file1";
    let filename2 = "file2";
    let missing_path1 = "missing1";
    let missing_path2 = "missing2";

    with_test_dir(|mut cmd, test_dir| {
        let file1 = test_dir.child(filename1);
        file1.touch()?;
        let file2 = test_dir.child(filename2);
        file2.touch()?;

        cmd.args([filename1, missing_path1, filename2, missing_path2])
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(filename1),
                rm_out::dry_removed(filename2);
                rm_out::newline(),
                rm_out::dry_conclusion(2, 0),
            ))
            .stderr("");
        file1.assert(predicate::path::exists());
        file2.assert(predicate::path::exists());

        cmd.arg("--verbose")
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::dry_removed(filename1),
                rm_out::dry_removed(filename2),
                rm_out::skipped_not_found(missing_path1),
                rm_out::skipped_not_found(missing_path2);
                rm_out::newline(),
                rm_out::dry_conclusion(2, 0),
            ))
            .stderr("");
        file1.assert(predicate::path::exists());
        file2.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::removed(filename1),
                rm_out::removed(filename2),
                rm_out::skipped_not_found(missing_path1),
                rm_out::skipped_not_found(missing_path2);
                rm_out::newline(),
                rm_out::conclusion(2, 0),
            ))
            .stderr("");
        file1.assert(predicate::path::missing());
        file2.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn all_paths_exist() -> TestResult {
    let filename1 = "file1";
    let filename2 = "file2";

    with_test_dir(|mut cmd, test_dir| {
        let file1 = test_dir.child(filename1);
        file1.touch()?;
        let file2 = test_dir.child(filename2);
        file2.touch()?;

        cmd.args([filename1, filename2])
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(filename1),
                rm_out::dry_removed(filename2);
                rm_out::newline(),
                rm_out::dry_conclusion(2, 0),
            ))
            .stderr("");
        file1.assert(predicate::path::exists());
        file2.assert(predicate::path::exists());

        cmd.arg("--verbose")
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::dry_removed(filename1),
                rm_out::dry_removed(filename2);
                rm_out::newline(),
                rm_out::dry_conclusion(2, 0),
            ))
            .stderr("");
        file1.assert(predicate::path::exists());
        file2.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::removed(filename1),
                rm_out::removed(filename2);
                rm_out::newline(),
                rm_out::conclusion(2, 0),
            ))
            .stderr("");
        file1.assert(predicate::path::missing());
        file2.assert(predicate::path::missing());

        Ok(())
    })
}

/// Run a test with `--blind` enabled.
///
/// See also [`common::with_test_dir`].
fn with_test_dir<C>(callback: C) -> TestResult
where
    C: FnOnce(assert_cmd::Command, &assert_fs::TempDir) -> TestResult,
{
    common::with_test_dir(|mut cmd, test_dir| {
        cmd.arg("--blind");
        callback(cmd, test_dir)
    })
}
