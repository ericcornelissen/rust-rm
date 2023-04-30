// SPDX-License-Identifier: Apache-2.0

//! Test suite focussed on testing the functionality of the `--interactive`/`-i` option.

pub mod common;

use crate::common::{has_exactly_lines, has_lines, rm_ask, rm_out, TestResult};

use std::path::MAIN_SEPARATOR;

use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn remove_file_no() -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.arg(filename)
            .write_stdin(format!("{NO}{ENTER}"))
            .assert()
            .success()
            .stdout(rm_out::conclusion(0, 0))
            .stderr(rm_ask::file(filename));
        file.assert(predicate::path::exists());

        cmd.arg("--verbose")
            .write_stdin(format!("{NO}{ENTER}"))
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::skipped_kept(filename);
                rm_out::newline(),
                rm_out::conclusion(0, 0),
            ))
            .stderr(rm_ask::file(filename));
        file.assert(predicate::path::exists());

        Ok(())
    })
}

#[test]
fn remove_symlink_no() -> TestResult {
    let linkname = "link";

    with_test_dir(|mut cmd, test_dir| {
        let linked_file = test_dir.child("linked_file");
        linked_file.touch()?;
        let link = test_dir.child(linkname);
        link.symlink_to_file(&linked_file)?;

        cmd.arg(linkname)
            .write_stdin(format!("{NO}{ENTER}"))
            .assert()
            .success()
            .stdout(rm_out::conclusion(0, 0))
            .stderr(rm_ask::link(linkname));
        linked_file.assert(predicate::path::exists());
        link.assert(predicate::path::exists());

        cmd.arg("--verbose")
            .write_stdin(format!("{NO}{ENTER}"))
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::skipped_kept(linkname);
                rm_out::newline(),
                rm_out::conclusion(0, 0),
            ))
            .stderr(rm_ask::link(linkname));
        linked_file.assert(predicate::path::exists());
        link.assert(predicate::path::exists());

        Ok(())
    })
}

#[test]
fn remove_empty_dir_no() -> TestResult {
    let dirname = "dir";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;

        cmd.args(["--dir", dirname])
            .write_stdin(format!("{NO}{ENTER}"))
            .assert()
            .success()
            .stdout(rm_out::conclusion(0, 0))
            .stderr(rm_ask::empty_dir(dirname));
        dir.assert(predicate::path::exists());

        cmd.arg("--verbose")
            .write_stdin(format!("{NO}{ENTER}"))
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::skipped_kept(dirname);
                rm_out::newline(),
                rm_out::conclusion(0, 0),
            ))
            .stderr(rm_ask::empty_dir(dirname));
        dir.assert(predicate::path::exists());

        Ok(())
    })
}

#[test]
fn remove_empty_dir_recursive_no() -> TestResult {
    let dirname = "dir";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;

        cmd.args(["--recursive", dirname])
            .write_stdin(format!("{NO}{ENTER}"))
            .assert()
            .success()
            .stdout(rm_out::conclusion(0, 0))
            .stderr(rm_ask::empty_dir(dirname));
        dir.assert(predicate::path::exists());

        cmd.arg("--verbose")
            .write_stdin(format!("{NO}{ENTER}"))
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::skipped_kept(dirname);
                rm_out::newline(),
                rm_out::conclusion(0, 0),
            ))
            .stderr(rm_ask::empty_dir(dirname));
        dir.assert(predicate::path::exists());

        Ok(())
    })
}

#[test]
fn remove_filled_dir_recursive_no() -> TestResult {
    let dirname = "dir";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;
        dir.child("file").touch()?;

        cmd.args(["--recursive", dirname])
            .write_stdin(format!("{NO}{ENTER}"))
            .assert()
            .success()
            .stdout(rm_out::conclusion(0, 0))
            .stderr(rm_ask::descend(dirname));
        dir.assert(predicate::path::exists());

        cmd.arg("--verbose")
            .write_stdin(format!("{NO}{ENTER}"))
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::skipped_kept(dirname);
                rm_out::newline(),
                rm_out::conclusion(0, 0),
            ))
            .stderr(rm_ask::descend(dirname));
        dir.assert(predicate::path::exists());

        Ok(())
    })
}

#[test]
fn remove_file_yes() -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.arg(filename)
            .write_stdin(format!("{YES}{ENTER}"))
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(filename);
                rm_out::newline(),
                rm_out::conclusion(1, 0),
            ))
            .stderr(rm_ask::file(filename));
        file.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn remove_symlink_yes() -> TestResult {
    let linkname = "link";

    with_test_dir(|mut cmd, test_dir| {
        let linked_file = test_dir.child("linked_file");
        linked_file.touch()?;
        let link = test_dir.child(linkname);
        link.symlink_to_file(&linked_file)?;

        cmd.arg(linkname)
            .write_stdin(format!("{YES}{ENTER}"))
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(linkname);
                rm_out::newline(),
                rm_out::conclusion(1, 0),
            ))
            .stderr(rm_ask::link(linkname));
        linked_file.assert(predicate::path::exists());
        link.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn remove_empty_dir_yes() -> TestResult {
    let dirname = "dir";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;

        cmd.args(["--dir", dirname])
            .write_stdin(format!("{YES}{ENTER}"))
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(dirname);
                rm_out::newline(),
                rm_out::conclusion(1, 0),
            ))
            .stderr(rm_ask::empty_dir(dirname));
        dir.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn remove_empty_dir_recursive_yes() -> TestResult {
    let dirname = "dir";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;

        cmd.args(["--recursive", dirname])
            .write_stdin(format!("{YES}{ENTER}"))
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(dirname);
                rm_out::newline(),
                rm_out::conclusion(1, 0),
            ))
            .stderr(rm_ask::empty_dir(dirname));
        dir.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn remove_filled_dir_recursive_yes_to_all() -> TestResult {
    let dirname = "dir";
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;
        dir.child(filename).touch()?;

        cmd.args(["--recursive", dirname])
            .write_stdin(format!(
                "\
                {YES}{ENTER}\
                {YES}{ENTER}\
                {YES}{ENTER}\
                "
            ))
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(format!("{dirname}{MAIN_SEPARATOR}{filename}")),
                rm_out::removed(dirname);
                rm_out::newline(),
                rm_out::conclusion(2, 0)
            ))
            .stderr(has_exactly_lines!(
                rm_ask::descend(dirname),
                rm_ask::file(format!("{dirname}{MAIN_SEPARATOR}{filename}")),
                rm_ask::empty_dir(dirname),
            ));
        dir.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn remove_filled_dir_recursive_descend_but_keep_dir() -> TestResult {
    let dirname = "dir";
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;
        let file = dir.child(filename);
        file.touch()?;

        cmd.args(["--recursive", dirname])
            .write_stdin(format!(
                "\
                {YES}{ENTER}\
                {YES}{ENTER}\
                {NO}{ENTER}\
                "
            ))
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(format!("{dirname}{MAIN_SEPARATOR}{filename}"));
                rm_out::newline(),
                rm_out::conclusion(1, 0),
            ))
            .stderr(has_exactly_lines!(
                rm_ask::descend(dirname),
                rm_ask::file(format!("{dirname}{MAIN_SEPARATOR}{filename}")),
                rm_ask::empty_dir(dirname),
            ));
        dir.assert(predicate::path::exists());
        file.assert(predicate::path::missing());

        file.touch()?;

        cmd.arg("--verbose")
            .write_stdin(format!(
                "\
                {YES}{ENTER}\
                {YES}{ENTER}\
                {NO}{ENTER}\
                "
            ))
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::removed(format!("{dirname}{MAIN_SEPARATOR}{filename}")),
                rm_out::skipped_kept(dirname);
                rm_out::newline(),
                rm_out::conclusion(1, 0),
            ))
            .stderr(has_exactly_lines!(
                rm_ask::descend(dirname),
                rm_ask::file(format!("{dirname}{MAIN_SEPARATOR}{filename}")),
                rm_ask::empty_dir(dirname),
            ));
        dir.assert(predicate::path::exists());
        file.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn remove_filled_dir_recursive_descend_but_keep_all() -> TestResult {
    let dirname = "dir";
    let filename1 = "file1";
    let filename2 = "file2";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;
        let file1 = dir.child(filename1);
        file1.touch()?;
        let file2 = dir.child(filename2);
        file2.touch()?;

        cmd.args(["--recursive", dirname])
            .write_stdin(format!(
                "\
                {YES}{ENTER}\
                {NO}{ENTER}\
                {NO}{ENTER}\
                {NO}{ENTER}\
                "
            ))
            .assert()
            .success()
            .stdout(rm_out::conclusion(0, 0))
            .stderr(has_exactly_lines!(
                rm_ask::descend(dirname),
                rm_ask::file(format!("{dirname}{MAIN_SEPARATOR}{filename1}")),
                rm_ask::file(format!("{dirname}{MAIN_SEPARATOR}{filename2}")),
                rm_ask::dir(dirname),
            ));
        dir.assert(predicate::path::exists());
        file1.assert(predicate::path::exists());
        file2.assert(predicate::path::exists());

        cmd.arg("--verbose")
            .write_stdin(format!(
                "\
                {YES}{ENTER}\
                {NO}{ENTER}\
                {NO}{ENTER}\
                {NO}{ENTER}\
                "
            ))
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::skipped_kept(format!("{dirname}{MAIN_SEPARATOR}{filename1}")),
                rm_out::skipped_kept(format!("{dirname}{MAIN_SEPARATOR}{filename2}")),
                rm_out::skipped_kept(dirname);
                rm_out::newline(),
                rm_out::conclusion(0, 0),
            ))
            .stderr(has_exactly_lines!(
                rm_ask::descend(dirname),
                rm_ask::file(format!("{dirname}{MAIN_SEPARATOR}{filename1}")),
                rm_ask::file(format!("{dirname}{MAIN_SEPARATOR}{filename2}")),
                rm_ask::dir(dirname),
            ));
        dir.assert(predicate::path::exists());
        file1.assert(predicate::path::exists());
        file2.assert(predicate::path::exists());

        Ok(())
    })
}

#[test]
fn remove_filled_dir_recursive_descend_but_keep_content_and_remove_dir() -> TestResult {
    let dirname = "dir";
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;
        let file = dir.child(filename);
        file.touch()?;

        cmd.args(["--recursive", dirname])
            .write_stdin(format!(
                "\
                {YES}{ENTER}\
                {NO}{ENTER}\
                {YES}{ENTER}\
                "
            ))
            .assert()
            .failure()
            .stdout(has_exactly_lines!(
                ;
                rm_out::newline(),
                rm_out::conclusion(0, 1),
            ))
            .stderr(has_exactly_lines!(
                rm_ask::descend(dirname),
                rm_ask::file(format!("{dirname}{MAIN_SEPARATOR}{filename}")),
                rm_ask::dir(dirname),
                rm_out::dir_not_empty_no_tip(dirname),
            ));
        dir.assert(predicate::path::exists());
        file.assert(predicate::path::exists());

        cmd.arg("--verbose")
            .write_stdin(format!(
                "\
                {YES}{ENTER}\
                {NO}{ENTER}\
                {YES}{ENTER}\
                "
            ))
            .assert()
            .failure()
            .stdout(has_lines!(
                rm_out::skipped_kept(format!("{dirname}{MAIN_SEPARATOR}{filename}"));
                rm_out::newline(),
                rm_out::conclusion(0, 1),
            ))
            .stderr(has_exactly_lines!(
                rm_ask::descend(dirname),
                rm_ask::file(format!("{dirname}{MAIN_SEPARATOR}{filename}")),
                rm_ask::dir(dirname),
                rm_out::dir_not_empty_no_tip(dirname),
            ));
        dir.assert(predicate::path::exists());
        file.assert(predicate::path::exists());

        Ok(())
    })
}

#[test]
#[cfg(feature = "trash")]
#[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
fn remove_filled_dir_recursive_trash() -> TestResult {
    let dirname = "dir";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;
        dir.child("file").touch()?;

        cmd.args(["--recursive", "--trash", dirname])
            .write_stdin(format!("{YES}{ENTER}"))
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::trashed(dirname),
                rm_out::newline(),
                rm_out::conclusion(1, 0)
            ))
            .stderr(has_exactly_lines!(rm_ask::dir(dirname)));
        dir.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn answer_uppercase_n() -> TestResult {
    test_answer_no("N")
}

#[test]
fn answer_lowercase_n() -> TestResult {
    test_answer_no("n")
}

#[test]
fn answer_no() -> TestResult {
    test_answer_no("no")
}

#[test]
fn answer_uppercase_y() -> TestResult {
    test_answer_yes("Y")
}

#[test]
fn answer_lowercase_y() -> TestResult {
    test_answer_yes("y")
}

#[test]
fn answer_yes() -> TestResult {
    test_answer_yes("yes")
}

#[test]
fn answer_invalid() -> TestResult {
    let answer = "invalid answer";
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.arg(filename)
            .write_stdin(format!("{answer}{ENTER}"))
            .assert()
            .success()
            .stdout(rm_out::conclusion(0, 0))
            .stderr(rm_ask::file(filename));
        file.assert(predicate::path::exists());

        cmd.arg("--verbose")
            .write_stdin(format!("{answer}{ENTER}"))
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::skipped_invalid_input(filename);
                rm_out::newline(),
                rm_out::conclusion(0, 0),
            ))
            .stderr(rm_ask::file(filename));
        file.assert(predicate::path::exists());

        Ok(())
    })
}

#[test]
fn not_found() -> TestResult {
    let missing_path = "missing";

    with_test_dir(|mut cmd, _test_dir| {
        cmd.arg(missing_path)
            .assert()
            .failure()
            .stdout(has_exactly_lines!(
                ;
                rm_out::newline(),
                rm_out::conclusion(0, 1),
            ))
            .stderr(rm_out::not_found(missing_path));

        Ok(())
    })
}

/// Re-usable test for validating the behaviour of a correct negative answer.
fn test_answer_no(answer: &str) -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.arg(filename)
            .write_stdin(format!("{answer}{ENTER}"))
            .assert()
            .success()
            .stdout(rm_out::conclusion(0, 0))
            .stderr(rm_ask::file(filename));
        file.assert(predicate::path::exists());

        cmd.arg("--verbose")
            .write_stdin(format!("{answer}{ENTER}"))
            .assert()
            .success()
            .stdout(has_lines!(
                rm_out::skipped_kept(filename);
                rm_out::newline(),
                rm_out::conclusion(0, 0)
            ))
            .stderr(rm_ask::file(filename));
        file.assert(predicate::path::exists());

        Ok(())
    })
}

/// Re-usable test for validating the behaviour of a correct positive answer.
fn test_answer_yes(answer: &str) -> TestResult {
    let filename = "file";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.arg(filename)
            .write_stdin(format!("{answer}{ENTER}"))
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(filename);
                rm_out::newline(),
                rm_out::conclusion(1, 0)
            ))
            .stderr(rm_ask::file(filename));
        file.assert(predicate::path::missing());

        Ok(())
    })
}

/// Run a test with `--interactive` enabled.
///
/// See also [`common::with_test_dir`].
fn with_test_dir<C>(callback: C) -> TestResult
where
    C: FnOnce(assert_cmd::Command, &assert_fs::TempDir) -> TestResult,
{
    common::with_test_dir(|mut cmd, test_dir| {
        cmd.arg("--interactive");
        callback(cmd, test_dir)
    })
}

/// String used on stdin to provide a line input.
const ENTER: char = '\n';

/// The default negative answer for --interactive tests.
const NO: &str = "n";

/// The default positive answer for --interactive tests.
const YES: &str = "y";
