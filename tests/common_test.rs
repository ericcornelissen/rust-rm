// SPDX-License-Identifier: Apache-2.0

//! Test suite focussed on testing the test utilities from `common.rs`.

pub mod common;

use crate::common::{has_exactly_lines, has_lines, with_test_dir};

use predicates::prelude::*;

#[test]
fn has_lines_basic() {
    let test_str = "\
         line1\n\
         line2\n\
         line3\n\
     ";

    assert!(has_lines!("line1\n", "line2\n").eval(test_str));
    assert!(has_lines!("line2\n", "line3\n").eval(test_str));
    assert!(has_lines!("line1\n", "line2\n", "line3\n").eval(test_str));

    assert!(!has_lines!("line1\n", "foobar\n").eval(test_str));
    assert!(!has_lines!("foobar\n", "line2\n").eval(test_str));
}

#[test]
fn has_lines_advanced() {
    let test_str = "\
         line1\n\
         line2\n\
         line3\n\
     ";

    assert!(has_lines!("line1\n"; "line3\n").eval(test_str));
    assert!(has_lines!("line2\n"; "line3\n").eval(test_str));

    assert!(!has_lines!("line1\n"; "line2\n").eval(test_str));
    assert!(!has_lines!("line3\n"; "line1\n").eval(test_str));
}

#[test]
fn has_exactly_lines_basic() {
    let test_str = "\
         line1\n\
         line2\n\
         line3\n\
     ";

    assert!(has_exactly_lines!("line1\n", "line2\n", "line3\n").eval(test_str));
    assert!(has_exactly_lines!("line2\n", "line3\n", "line1\n").eval(test_str));

    assert!(!has_exactly_lines!("line1\n", "line2\n", "foobar\n").eval(test_str));
    assert!(!has_exactly_lines!("line1\n", "line2\n").eval(test_str));
}

#[test]
fn has_exactly_lines_advanced() {
    let test_str = "\
         line1\n\
         line2\n\
         line3\n\
     ";

    assert!(has_exactly_lines!("line1\n", "line2\n"; "line3\n").eval(test_str));
    assert!(has_exactly_lines!("line2\n", "line1\n"; "line3\n").eval(test_str));

    assert!(!has_exactly_lines!("line1\n"; "line3\n").eval(test_str));
    assert!(!has_exactly_lines!("line1\n", "line3\n"; "line2\n").eval(test_str));
}

#[test]
fn with_test_dir_ok() {
    let out = with_test_dir(|_cmd, _test_dir| Ok(()));

    assert!(out.is_ok());
}

#[test]
fn with_test_dir_err() {
    let out = with_test_dir(|_cmd, _test_dir| {
        let err = Box::new(std::io::Error::new(std::io::ErrorKind::AddrInUse, "for testing"));
        Err(err)
    });

    assert!(out.is_err());
}
