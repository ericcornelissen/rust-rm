//! Test suite focussed on testing the functionality of the `--force`/`-f` option.

pub mod common;

use crate::common::{has_exactly_lines, rm_out, with_test_dir, TestResult};

use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn zero_paths() -> TestResult {
    with_test_dir(|mut cmd, _test_dir| {
        cmd.assert().success().stdout(rm_out::dry_conclusion(0, 0)).stderr("");

        cmd.arg("--force").assert().success().stdout(rm_out::conclusion(0, 0)).stderr("");

        Ok(())
    })
}

#[test]
fn one_file() -> TestResult {
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
fn one_empty_dir() -> TestResult {
    let dirname = "dir";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;

        cmd.arg(dirname)
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::dry_conclusion(0, 1)))
            .stderr(rm_out::is_a_dir(dirname));
        dir.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::conclusion(0, 1)))
            .stderr(rm_out::is_a_dir(dirname));
        dir.assert(predicate::path::exists());

        Ok(())
    })
}

#[test]
fn one_filled_dir() -> TestResult {
    let dirname = "dir";

    with_test_dir(|mut cmd, test_dir| {
        let dir = test_dir.child(dirname);
        dir.create_dir_all()?;
        dir.child("file").touch()?;

        cmd.arg(dirname)
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::dry_conclusion(0, 1)))
            .stderr(rm_out::is_a_dir(dirname));
        dir.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::conclusion(0, 1)))
            .stderr(rm_out::is_a_dir(dirname));
        dir.assert(predicate::path::exists());

        Ok(())
    })
}

#[test]
fn multiple_files() -> TestResult {
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

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
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

#[test]
fn missing_path() -> TestResult {
    let missing_path = "missing";

    with_test_dir(|mut cmd, _test_dir| {
        cmd.arg(missing_path)
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::dry_conclusion(0, 1)))
            .stderr(rm_out::not_found(missing_path));

        cmd.arg("--force")
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::conclusion(0, 1)))
            .stderr(rm_out::not_found(missing_path));

        Ok(())
    })
}

#[test]
fn found_path_and_missing_path() -> TestResult {
    let filename = "file";
    let missing_path = "missing";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.args([filename, missing_path])
            .assert()
            .failure()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(filename);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 1),
            ))
            .stderr(rm_out::not_found(missing_path));
        file.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .failure()
            .stdout(has_exactly_lines!(
                rm_out::removed(filename);
                rm_out::newline(),
                rm_out::conclusion(1, 1),
            ))
            .stderr(rm_out::not_found(missing_path));
        file.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn missing_path_and_found_path() -> TestResult {
    let filename = "file";
    let missing_path = "missing";

    with_test_dir(|mut cmd, test_dir| {
        let file = test_dir.child(filename);
        file.touch()?;

        cmd.args([missing_path, filename])
            .assert()
            .failure()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(filename);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 1),
            ))
            .stderr(rm_out::not_found(missing_path));
        file.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .failure()
            .stdout(has_exactly_lines!(
                rm_out::removed(filename);
                rm_out::newline(),
                rm_out::conclusion(1, 1),
            ))
            .stderr(rm_out::not_found(missing_path));
        file.assert(predicate::path::missing());

        Ok(())
    })
}
