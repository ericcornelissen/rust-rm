// SPDX-License-Identifier: Apache-2.0

//! Test suite focussed on testing potentially dangerous scenarios. For example, trying to remove
//! the file system root.
//!
//! These tests only run when the "test-dangerous" feature is enabled. You can use the following
//! command to run these tests: `cargo test --features test-dangerous`

pub mod common;

use crate::common::{TestResult, has_exactly_lines, rm_out, with_test_dir};

use predicates::prelude::*;

#[test]
#[cfg_attr(not(feature = "test-dangerous"), ignore = "Only run with the test-dangerous feature")]
fn file_system_root() -> TestResult {
    with_test_dir(|mut cmd, _test_dir| {
        cmd.arg("/")
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::dry_conclusion(0, 1)))
            .stderr(rm_out::refused("/"));

        cmd.arg("--recursive")
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::dry_conclusion(0, 1)))
            .stderr(rm_out::refused("/"));

        cmd.arg("--force")
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::conclusion(0, 1)))
            .stderr(rm_out::refused("/"));

        Ok(())
    })
}

#[test]
#[cfg_attr(not(feature = "test-dangerous"), ignore = "Only run with the test-dangerous feature")]
fn current_directory() -> TestResult {
    with_test_dir(|mut cmd, _test_dir| {
        cmd.arg(".")
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::dry_conclusion(0, 1)))
            .stderr(rm_out::refused("."));

        cmd.arg("--recursive")
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::dry_conclusion(0, 1)))
            .stderr(rm_out::refused("."));

        cmd.arg("--force")
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::conclusion(0, 1)))
            .stderr(rm_out::refused("."));

        Ok(())
    })
}

#[test]
#[cfg_attr(not(feature = "test-dangerous"), ignore = "Only run with the test-dangerous feature")]
fn parent_directory() -> TestResult {
    with_test_dir(|mut cmd, _test_dir| {
        cmd.arg("..")
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::dry_conclusion(0, 1)))
            .stderr(rm_out::refused(".."));

        cmd.arg("--recursive")
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::dry_conclusion(0, 1)))
            .stderr(rm_out::refused(".."));

        cmd.arg("--force")
            .assert()
            .failure()
            .stdout(has_exactly_lines!(; rm_out::newline(), rm_out::conclusion(0, 1)))
            .stderr(rm_out::refused(".."));

        Ok(())
    })
}
