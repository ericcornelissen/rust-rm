# Contributing Guidelines

The _rust-rm_ project welcomes contributions and corrections of all forms. This includes
improvements to the documentation or code base, new tests, bug fixes, and implementations of new
features. We recommend you open an issue before making any substantial changes so you can be sure
your work won't be rejected. But for small changes, such as fixing a typo, you can open a Pull
Request directly.

If you plan to make a contribution, please do make sure to read through the relevant sections of
this document.

- [Reporting Issues](#reporting-issues)
  - [Security Reports](#security-reports)
  - [Bug Reports](#bug-reports)
  - [Feature Requests](#feature-requests)
- [Making Changes](#making-changes)
  - [Prerequisites](#prerequisites)
  - [Workflow](#workflow)
  - [Development](#development)
  - [Building](#building)
  - [Formatting](#formatting)
  - [Testing](#testing)
    - [Test Organization](#test-organization)
    - [Types of Tests](#types-of-tests)
    - [Test Coverage](#test-coverage)
    - [Mutation Testing](#mutation-testing)
    - [Special Tests](#special-tests)
  - [Documenting](#documenting)
  - [Vetting](#vetting)
  - [Auditing](#auditing)
  - [License Compliance](#license-compliance)
- [Releasing](#releasing)
  - [Release Numbering](#release-numbering)
  - [Release Process](#release-process)
- [IDEs](#ides)
  - [Visual Studio Code](#visual-studio-code)

---

## Reporting Issues

### Security Reports

For security related issues, please refer to the [security policy].

[security policy]: ./SECURITY.md

### Bug Reports

If you have problems with the software or think you've found a bug, please report it to the
developers. We ask you to always open an issue describing the bug as soon as possible so that we,
and others, are aware of the bug.

Before reporting a bug, make sure you've actually found a real bug. Carefully read the documentation
and see if it really says you can do what you're trying to do. If it's not clear whether you should
be able to do something or not, report that too; it's a bug in the documentation! Also, make sure
the bug has not already been reported.

When preparing to report a bug, try to isolate it to a small working example that reproduces the
problem. Then, create a bug report including this example and its results as well as any error or
warning messages. Please don't paraphrase these messages: it's best to copy and paste them into your
report. Finally, be sure to explain what you expected to happen; this will help us decide whether it
is a bug or a problem with the documentation.

Once you have a precise problem you can report it as a [bug report].

[bug report]: https://github.com/ericcornelissen/rust-rm/issues/new

### Feature Requests

If you want the software to do something it currently does not, please report it to the developers.
We ask you to always open an issue describing the feature as soon as possible so that we, and
others, can consider and discuss it.

Before reporting a feature request, make sure you can't easily achieve what you want using existing
features. Carefully read the documentation and experiment with the software. If it's not clear how
to do something, report that too; it's a gap in the documentation! Also, make sure the feature has
not already been requested.

Once you have a clear idea of what you need you can submit a [feature request].

[feature request]: https://github.com/ericcornelissen/rust-rm/issues/new

---

## Making Changes

You are free to contribute by working on one of the confirmed or accepted and unassigned [open
issues] and opening a Pull Request for it.

It is advised to indicate that you will be working on a issue by commenting on that issue. This is
so others don't start working on the same issue as you are. Also, don't start working on an issue
which someone else is working on - give everyone a chance to make contributions.

When you open a Pull Request that implements an issue make sure to link to that issue in the Pull
Request description and explain how you implemented the issue as clearly as possible.

> **Note**: If you, for whatever reason, can no longer continue your contribution please share this
> in the issue or your Pull Request. This gives others the opportunity to work on it. If we don't
> hear from you for an extended period of time we may decide to allow others to work on the issue
> you were assigned to.

[open issues]: https://github.com/ericcornelissen/rust-rm/issues?q=is%3Aissue+is%3Aopen+no%3Aassignee

### Prerequisites

To be able to contribute you need the following tooling:

- [git] v2;
- [Just] v1;
- [Rust] and [Cargo] v1.72 (edition 2021) with [Clippy], [rustfmt] (see `rust-toolchain.toml`);
- (Optional) [cargo-all-features] v1.7.0 or later;
- (Optional) [cargo-deny] v0.13.0 or later;
- (Optional) [cargo-mutants] v23.0.0 or later;
- (Optional) [cargo-tarpaulin] v0.25.0 or later;
- (Suggested) a code editor with [EditorConfig] support;

[cargo]: https://doc.rust-lang.org/stable/cargo/
[clippy]: https://rust-lang.github.io/rust-clippy/
[editorconfig]: https://editorconfig.org/
[git]: https://git-scm.com/
[just]: https://just.systems/
[rust]: https://www.rust-lang.org/
[rustfmt]: https://rust-lang.github.io/rustfmt/
[cargo-all-features]: https://github.com/frewsxcv/cargo-all-features
[cargo-deny]: https://github.com/EmbarkStudios/cargo-deny
[cargo-mutants]: https://github.com/sourcefrog/cargo-mutants
[cargo-tarpaulin]: https://github.com/xd009642/tarpaulin

### Workflow

If you decide to make a contribution, please use the following workflow:

- Fork the repository.
- Create a new branch from the latest `main`.
- Make your changes on the new branch.
- Commit to the new branch and push the commit(s).
- Open a Pull Request against `main`.

### Development

When contributing make sure your changes [build](#building) and are [formatted](#formatting),
[tested](#testing), [documented](#documenting), and [vetted](#vetting). A simple way to verify if
this is the case is using the command:

```shell
just verify
```

When making changes to the dependency tree be sure to also [audit](#auditing) and validate [license
compliance](#license-compliance) against your changes using the command:

```shell
just audit compliance
```

### Building

As a [Rust] codebase, this project is build using [Cargo]. Ensure all changes you commit can be
compiled. To compile the source code you can use the command:

```shell
just build
```

If your changes relate to a compile-time feature you can use the `features` variable to adjust the
build, for example:

```shell
just features=gnu-mode build
```

For greater certainty that your changes are valid you can also use the build command used in
continuous integration. This is more extensive but as a result also slower.

```shell
just ci-build
```

### Formatting

This project is formatted using [rustfmt]. Ensure all changes you commit are formatted. To format
the source code run the command:

```shell
just fmt
```

Formatting is configured explicitly in the `rustfmt.toml` file.

### Testing

This project is extensively tested. Most source code changes should be accompanied by new tests or
changes to the existing tests. To run all tests use the command:

```shell
just test
```

If necessary you can configure the feature set to test with using the `features` variable to adjust
the build, for example:

```shell
just features=gnu-mode test
```

For greater certainty that your changes are valid you can also use the test command used in
continuous integration. This is more extensive but as a result also slower.

> **Note**: This includes running [Special Tests](#special-tests), caution is advised.

```shell
just ci-test
```

To manually test something you can compile and run the program with any arguments using the
`just run` command, for example:

```shell
just run --help
```

#### Tests and the File System

Due to the nature of this project many tests interact with the file system. Every test that does
interact with the file system uses its own temporary directory. By default, these are cleaned up
automatically (regardless of success). To preserve all temporary test directories you can set the
environment `RUST_RM_DEBUG_TEST`.

#### Test Organization

Following [Rust]'s official [test organization] guidelines, this project has:

- **Unit tests**: for testing individual units of code. These tests are located next to the source
  code it tests. You can run only these tests using the command:

  ```shell
  just test-unit
  ```

- **Integration tests**: for testing the binary. These tests are located in the `tests/` directory.
  You can run only these tests using the command:

  ```shell
  just test-integration
  ```

[test organization]: https://doc.rust-lang.org/book/ch11-03-test-organization.html

#### Types of Tests

A test can be written in different ways. This project uses three ways to write tests:

- **Oracle test**: Simple tests that validate the behavior of the system for a given input-output pair
  (the _oracle_).

  These tests are useful when the input domain is small (e.g. booleans only), to test edge cases, or
  if running the test is expensive (e.g. it interacts with the file system). If these conditions
  don't hold it is advised to use more advanced types of tests.

- **Parameterized test**: Extends oracle tests by running a test with multiple input-output pairs.

  These tests are useful to test the behavior of the system for related inputs, for different
  input-output pairs with similar behavior, or to test edge cases.

  In this project parameterized tests are written using a helper function that runs the test on
  given inputs, and individual `#[test]`s that call this helper function with specific inputs.

- **Property test** (using [proptest]): Generalized tests that test for properties and invariants.

  These tests are useful to test a wide range of inputs. They are also expressive in that they can
  be used to specify the general behavior of a system.

  Note that property tests are bad at testing edge cases and that they should never be used for
  tests that interacts with the file system.

[proptest]: https://crates.io/crates/proptest

#### Test Coverage

Test coverage can be used as a guide to writing tests - if a certain code path isn't covered by any
tests it's behavior can't be validated automatically. To aid in writing tests this project is set up
with test coverage powered by [cargo-tarpaulin]. To generate a coverage report for tests use the
commands:

```shell
just coverage
```

This will generate a coverage report which can be found in the `_reports/coverage/` directory.

If necessary you can configure the feature set to test using the `features` variable to adjust the
build, for example:

```shell
just features=gnu-mode coverage
```

#### Mutation Testing

Mutation testing can be used as a guide to improve the test suite - if a mutation in the source code
isn't detected by any test it indicates a gap in the suite. To aid in writing tests this project is
set up with mutation testing powered by [cargo-mutants]. To generate a mutation report use the
command:

```shell
just mutation
```

This will generate a mutation report which can be found in the `_reports/mutants.out/` directory.

If necessary you can configure the feature set to mutation test using the `features` variable to
adjust the build, for example:

```shell
just features=gnu-mode mutation
```

#### Special Tests

##### `--trash` Tests

Some tests need to test the `--trash` functionality. These tests aren't run by default to avoid
filling the trash with test files.

The following configuration must be used on all tests that (potentially) dispose of files to the
trash:

```rust
#[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
```

To run tests involving the trash use the command:

```shell
just test_features=test-trash test
```

##### Dangerous Tests

Some test (try to) perform potentially dangerous operations. These tests aren't run by default to
avoid accidentally doing something disastrous.

The following configuration can be used to mark a test as dangerous:

```rust
#[cfg_attr(not(feature = "test-dangerous"), ignore = "Only run with the test-dangerous feature")]
```

To run dangerous tests use the command:

```shell
just test_features=test-dangerous test
```

##### Symlink Tests

Some tests work with symbolic links (symlinks). Working with symlinks on Windows requires elevated
permissions, so these tests aren't run by default on Windows to avoid unexpected errors.

The following configuration must be used on all tests that (potentially) create symbolic links:

```rust
#[cfg_attr(
    all(windows, not(feature = "test-symlink")),
    ignore = "Only run with the test-symlink feature"
)]
```

To run tests involving symbolic links on Windows ensure you have elevated privileges and use the
command:

```shell
just test_features=test-symlink test
```

> **Note**: On non-Windows systems tests involving symlinks are run by default (regardless of the
> `test-symlink` feature).

### Documenting

This project is extensively documented. All source code should be documented to aid reuse without
having to understand all code. All documentation is written for [rustdoc]. You can generate HTML
formatted documentation for this project and its dependencies using the command:

```shell
just docs
```

Documentation in this project follows the following general structure:

```rust
/// [short sentence explaining what it is or does]
///
/// [more detailed explanation, optional]
///
/// [# Errors]
///
/// [describe when the function errors, if applicable]
///
/// [# Panics]
///
/// [describe when the function panics, if applicable]
///
/// [# Examples]
///
/// [code example, optional]
```

[rustdoc]: https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html

### Vetting

This project is vetted using standard [Cargo] commands as well as [Clippy]. To vet the source code
run the command:

```shell
just vet
```

Clippy is configured explicitly through CLI arguments (see `Justfile)` and the `clippy.toml` file.

For greater certainty that your changes are valid you can also use the vet command used in
continuous integration. This is more extensive but as a result also slower.

```shell
just ci-vet
```

### Auditing

This project uses [cargo-deny] to audit dependencies for known vulnerabilities. To audit the
project dependencies run the command:

```shell
just audit
```

The configuration used for auditing is stored in `deny.toml`.

### License Compliance

This project uses [cargo-deny] to enforce a license policy on the project dependencies. To verify
all dependency licenses are in compliance run the command:

```shell
just compliance
```

The license compliance policy is stored in `deny.toml`.

---

## Releasing

### Release Numbering

Releases of this project are numbered based on the year and month in which they were released. Both
the year and month must always be represented with two digits, for example `23.01` for January 2023.

In the event a patch has to be released when a release already took place that month an incremental
integer suffix is added to the release number, for example `23.01-1` (i.e. `23.01 == 23.01-0`).

### Release Process

To create a new release, first update the release number in the CLI help text by changing the
following line, for example:

```diff
- /// Remove (unlink) the PATH(s) - v23.06
+ /// Remove (unlink) the PATH(s) - v23.07
```

Commit this change with a message along the lines of "Version bump", for example:

```shell
git commit -m "Version bump"
```

Get this commit onto the project's default branch. When it is, tag the commit with same value used
for the CLI help text. Give the tag an annotation with a list of changes since the last release, for
example:

```shell
git tag -a v23.07
```

Push the tag to the GitHub repository, for example:

```shell
git push origin v23.07
```

This will trigger the [`publish.yml` workflow] which will create a GitHub Release for the release
and compiled binaries for various platforms and architectures.

[`publish.yml` workflow]: ./.github/workflows/publish.yml

---

## IDEs

To get the most out of your code editor when contributing to this project you can use the tips
provided here.

### Visual Studio Code

Add the following options to the `.vscode/settings.json` file for this project:

```jsonc
{
  // Control what build features rust-analyzer for VSCode considers to be enabled. Useful if you are
  // working on code related to one of these features.
  "rust-analyzer.cargo.noDefaultFeatures": true,
  "rust-analyzer.cargo.features": [
    "gnu-mode",
    "trash",
  ],
}
```
