// SPDX-License-Identifier: Apache-2.0

//! Test suite focussed on testing the removal of soft (a.k.a. symbolic) and hard links.

pub mod common;

use crate::common::{has_exactly_lines, rm_out, with_test_dir, TestResult};

use std::fs;

use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn symlink_to_a_file_remove_link() -> TestResult {
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
                rm_out::dry_conclusion(1, 0)
            ))
            .stderr("");
        linked_file.assert(predicate::path::exists());
        link.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(linkname);
                rm_out::newline(),
                rm_out::conclusion(1, 0)
            ))
            .stderr("");
        linked_file.assert(predicate::path::exists());
        link.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn symlink_to_a_file_remove_file() -> TestResult {
    let filename = "linked_file";
    let linkname = "link";

    with_test_dir(|mut cmd, test_dir| {
        let linked_file = test_dir.child(filename);
        linked_file.touch()?;
        let link = test_dir.child(linkname);
        link.symlink_to_file(&linked_file)?;

        cmd.arg(filename)
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(filename);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0)
            ))
            .stderr("");
        linked_file.assert(predicate::path::exists());
        link.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(filename);
                rm_out::newline(),
                rm_out::conclusion(1, 0)
            ))
            .stderr("");
        linked_file.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg_attr(windows, ignore = "TODO: investigate symlink test errors on Windows")]
fn symlink_to_an_empty_dir_remove_link() -> TestResult {
    let linkname = "link";

    with_test_dir(|mut cmd, test_dir| {
        let linked_dir = test_dir.child("linked_dir");
        linked_dir.create_dir_all()?;
        let link = test_dir.child(linkname);
        link.symlink_to_dir(&linked_dir)?;

        cmd.arg(linkname)
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(linkname);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0)
            ))
            .stderr("");
        linked_dir.assert(predicate::path::exists());
        link.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(linkname);
                rm_out::newline(),
                rm_out::conclusion(1, 0)
            ))
            .stderr("");
        linked_dir.assert(predicate::path::exists());
        link.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn symlink_to_an_empty_dir_remove_dir() -> TestResult {
    let dirname = "linked_dir";
    let linkname = "link";

    with_test_dir(|mut cmd, test_dir| {
        let linked_dir = test_dir.child(dirname);
        linked_dir.create_dir_all()?;
        let link = test_dir.child(linkname);
        link.symlink_to_dir(&linked_dir)?;

        cmd.args(["--dir", dirname])
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(dirname);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0)
            ))
            .stderr("");
        linked_dir.assert(predicate::path::exists());
        link.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(dirname);
                rm_out::newline(),
                rm_out::conclusion(1, 0)
            ))
            .stderr("");
        linked_dir.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
#[cfg_attr(windows, ignore = "TODO: investigate symlink test errors on Windows")]
fn symlink_to_a_filled_dir_remove_link() -> TestResult {
    let linkname = "link";

    with_test_dir(|mut cmd, test_dir| {
        let linked_dir = test_dir.child("dir");
        linked_dir.create_dir_all()?;
        let nested_file = linked_dir.child("file");
        nested_file.touch()?;
        let link = test_dir.child(linkname);
        link.symlink_to_dir(&linked_dir)?;

        cmd.arg(linkname)
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(linkname);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0)
            ))
            .stderr("");
        linked_dir.assert(predicate::path::exists());
        nested_file.assert(predicate::path::exists());
        link.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(linkname);
                rm_out::newline(),
                rm_out::conclusion(1, 0)
            ))
            .stderr("");
        linked_dir.assert(predicate::path::exists());
        nested_file.assert(predicate::path::exists());
        link.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn hard_link_to_a_file_remove_link() -> TestResult {
    let linkname = "link";

    with_test_dir(|mut cmd, test_dir| {
        let linked_file = test_dir.child("linked_file");
        linked_file.touch()?;
        let link = test_dir.child(linkname);
        fs::hard_link(&linked_file, &link)?;

        cmd.arg(linkname)
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(linkname);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0)
            ))
            .stderr("");
        linked_file.assert(predicate::path::exists());
        link.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(linkname);
                rm_out::newline(),
                rm_out::conclusion(1, 0)
            ))
            .stderr("");
        linked_file.assert(predicate::path::exists());
        link.assert(predicate::path::missing());

        Ok(())
    })
}

#[test]
fn hard_link_to_a_file_remove_file() -> TestResult {
    let filename = "linked_file";
    let linkname = "link";

    with_test_dir(|mut cmd, test_dir| {
        let linked_file = test_dir.child(filename);
        linked_file.touch()?;
        let link = test_dir.child(linkname);
        fs::hard_link(&linked_file, &link)?;

        cmd.arg(filename)
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::dry_removed(filename);
                rm_out::newline(),
                rm_out::dry_conclusion(1, 0)
            ))
            .stderr("");
        linked_file.assert(predicate::path::exists());
        link.assert(predicate::path::exists());

        cmd.arg("--force")
            .assert()
            .success()
            .stdout(has_exactly_lines!(
                rm_out::removed(filename);
                rm_out::newline(),
                rm_out::conclusion(1, 0)
            ))
            .stderr("");
        linked_file.assert(predicate::path::missing());
        link.assert(predicate::path::exists());

        Ok(())
    })
}
