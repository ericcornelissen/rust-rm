// SPDX-License-Identifier: Apache-2.0

//! Test suite focussed on testing the output when `--verbose`/`-v` is used.

pub mod common;

use crate::common::{has_exactly_lines, rm_out, TestResult};

use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn nothing_to_do() -> TestResult {
    with_test_dir(|mut cmd, _test_dir| {
        cmd.assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::start();
                rm_out::newline(),
                rm_out::dry_conclusion(0, 0),
            ))
            .stderr("");

        Ok(())
    })
}

#[test]
fn found_file_dir_and_link() -> TestResult {
    let filename = "file";
    let dirname = "dir";
    let linkname = "link";
    let linked_filename = "linked_file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;
        let link = test_dir.child(linkname);
        test_dir.child(linked_filename).touch()?;
        link.symlink_to_file(linked_filename)?;

        cmd.args(["--dir", filename, dirname, linkname])
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::start(),
                rm_out::found_file(filename),
                rm_out::dry_removed(filename),
                rm_out::found_dir(dirname),
                rm_out::dry_removed(dirname),
                rm_out::found_link(linkname),
                rm_out::dry_removed(linkname);
                rm_out::newline(),
                rm_out::dry_conclusion(3, 0),
            ))
            .stderr("");

        Ok(())
    })
}

#[test]
fn some_not_found() -> TestResult {
    let filename = "file";
    let missing_path = "missing";

    with_test_dir(|mut cmd, test_dir| {
        test_dir.child(filename).touch()?;

        cmd.args([filename, missing_path])
            .assert()
            .failure()
            .stdout(has_exactly_lines!(
                rm_out::start(),
                rm_out::found_file(filename),
                rm_out::dry_removed(filename),
                rm_out::found_nothing(missing_path);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 1),
            ))
            .stderr(rm_out::not_found(missing_path));

        Ok(())
    })
}

#[test]
fn none_found() -> TestResult {
    let missing_path1 = "missing1";
    let missing_path2 = "missing2";

    with_test_dir(|mut cmd, _test_dir| {
        cmd.args([missing_path1, missing_path2])
            .assert()
            .failure()
            .stdout(has_exactly_lines!(
                rm_out::start(),
                rm_out::found_nothing(missing_path1),
                rm_out::found_nothing(missing_path2);
                rm_out::newline(),
                rm_out::dry_conclusion(0, 2),
            ))
            .stderr(has_exactly_lines!(
                rm_out::not_found(missing_path1),
                rm_out::not_found(missing_path2),
            ));

        Ok(())
    })
}

#[test]
fn skipped_paths() -> TestResult {
    let filename = "file1";
    let missing_path = "file2";

    with_test_dir(|mut cmd, test_dir| {
        test_dir.child(filename).touch()?;

        cmd.args(["--blind", filename, missing_path])
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::start(),
                rm_out::found_file(filename),
                rm_out::dry_removed(filename),
                rm_out::found_nothing(missing_path),
                rm_out::skipped_not_found(missing_path);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0),
            ))
            .stderr("");

        Ok(())
    })
}

/// Run a test with `--verbose` enabled and with the `DEBUG` environment variable set.
///
/// See also [`common::with_test_dir`].
fn with_test_dir<C>(callback: C) -> TestResult
where
    C: Fn(assert_cmd::Command, &assert_fs::TempDir) -> TestResult,
{
    common::with_test_dir(|mut cmd, test_dir| {
        cmd.arg("--verbose");
        callback(cmd, test_dir)
    })?;

    common::with_test_dir(|mut cmd, test_dir| {
        cmd.env("DEBUG", "1");
        callback(cmd, test_dir)
    })
}
