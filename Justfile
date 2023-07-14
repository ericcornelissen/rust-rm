# Check out Just at: https://just.systems/

alias b := build
alias t := test
alias v := vet

[private]
@default:
	just --list

# Audit the project for known vulnerabilities
@audit:
	cargo deny check advisories \
		--config ./deny.toml

# Build the rm binary
@build:
	cargo build \
		{{BUILD_ARGS}} \
		{{FEATURES}}

[private]
@build-each:
	cargo build-all-features \
		{{BUILD_ARGS}}

# Reset the repository to a clean state
@clean: _clean_cargo _clean_git

@_clean_cargo:
	cargo clean

@_clean_git:
	git clean -fx \
		_reports/ \
		mutants.out*/ \
		profile_fs/ \
		cobertura* \
		lcov* \
		loc.rs \
		perf.data \
		perf.data.old \
		perf.svg \
		tarpaulin*

# Check license compliance
@compliance:
	cargo deny check licenses \
		--config ./deny.toml

# Produce a coverage report for all tests
@coverage:
	cargo tarpaulin \
		{{COVERAGE_ARGS}} \
		{{TEST_FEATURES}} \
		{{FEATURES}}
	mv _reports/coverage/tarpaulin-report.html _reports/coverage/coverage-all.html

# Produce a coverage report for integration tests
@coverage-integration:
	cargo tarpaulin \
		{{COVERAGE_ARGS}} \
		{{TEST_INTEGRATION_ARGS}} \
		{{TEST_FEATURES}} \
		{{FEATURES}}
	mv _reports/coverage/tarpaulin-report.html _reports/coverage/coverage-integration.html

# Produce a coverage report for unit tests
@coverage-unit:
	cargo tarpaulin \
		{{COVERAGE_ARGS}} \
		{{TEST_UNIT_ARGS}} \
		{{TEST_FEATURES}} \
		{{FEATURES}}
	mv _reports/coverage/tarpaulin-report.html _reports/coverage/coverage-unit.html

# Generate documentation for the project and dependencies
@docs:
	cargo doc \
		{{DOCS_ARGS}}

# Format the source code
@fmt:
	cargo fmt

[private]
@fmt-check:
	cargo fmt --check

# Get the (minimum) number of lines of source code
@loc:
	perl \
		-0777 -pe \
		's/#\[cfg\(test\)\]\s+(?:#\[cfg\(feature\s*=\s*"[^"]+"\)\]\s+)?(?:pub\s+)?(?:mod [a-z_]+ (\{(?:(?>[^{}]+)|(?1))*\})|use [A-Za-z_\:;]+)//g' \
		src/main.rs \
	| sed \
		-e '/^ *$/d' \
		-e '/^ *\/\//d' \
		-e '/^ *#\[allow(/d' \
		-e '/^ *#\[cfg_attr(test,/d' \
		-e '/^ *#\[cfg(not(tarpaulin_include))]/d' \
		-e '/^ *#!\[deny/d' \
	> loc.rs
	wc -l loc.rs

# Run mutation tests
@mutation:
	cargo mutants \
		--output _reports/ \
		--exclude-re cli::run \
		--exclude-re logging \
		--exclude-re 'impl Display' \
		-- \
		{{TEST_UNIT_ARGS}} \
		{{TEST_FEATURES}}

# Profile with visualization using <https://github.com/brendangregg/FlameGraph>
[private]
@profile: _profile_prepare
	cargo build {{FEATURES}}
	perf record -F99 --call-graph dwarf -- \
		just run --dir --recursive --force profile_fs
	perf script | ./stackcollapse-perf.pl | ./flamegraph.pl > perf.svg

_profile_prepare:
	#!/usr/bin/env perl
	`rm -rf profile_fs`;
	`mkdir profile_fs`;
	for(1..1000) { `touch profile_fs/file-$_`; }
	`mkdir profile_fs/nested_dir`;
	for(1..750) { `touch profile_fs/nested_dir/file-$_`; }

# Run rm with the given arguments
@run *ARGS:
	cargo run \
		{{FEATURES}} \
		-- \
		{{ARGS}}

# Run all tests
@test:
	cargo test \
		{{TEST_ARGS}} \
		{{TEST_FEATURES}} \
		{{FEATURES}}

# Run all integration tests
@test-integration:
	cargo test \
		{{TEST_ARGS}} \
		{{TEST_INTEGRATION_ARGS}} \
		{{TEST_FEATURES}} \
		{{FEATURES}}

# Run all unit tests
@test-unit:
	cargo test \
		{{TEST_ARGS}} \
		{{TEST_UNIT_ARGS}} \
		{{TEST_FEATURES}} \
		{{FEATURES}}

[private]
@test-each:
	cargo test-all-features \
		{{TEST_ARGS}} \
		{{TEST_FEATURES}}

# Run all checks that should always succeed
@verify: build-each compliance docs fmt-check test-each vet

# Statically analyze the source code
@vet: _vet_check _vet_clippy _vet_verify_project

@_vet_check:
	echo 'Running "cargo check"...'
	cargo {{ if ci == TRUE { "check-all-features" } else { "check" } }} \
		{{CI_ONLY_CARGO_ARGS}}

@_vet_clippy:
	echo 'Running "cargo clippy"...'
	cargo clippy \
		--no-deps \
		--tests \
		-- \
		--deny clippy::cargo \
		--deny clippy::complexity \
		--deny clippy::correctness \
		--deny clippy::pedantic \
		--deny clippy::perf \
		--deny clippy::style \
		--deny clippy::suspicious \
		\
		--deny clippy::allow_attributes_without_reason \
		--deny clippy::arithmetic_side_effects \
		--deny clippy::dbg_macro \
		--deny clippy::disallowed_script_idents \
		--deny clippy::expect_used \
		--deny clippy::let_underscore_untyped \
		--deny clippy::if_then_some_else_none \
		--deny clippy::impl_trait_in_params \
		--deny clippy::indexing_slicing \
		--deny clippy::missing_docs_in_private_items \
		--deny clippy::missing_enforced_import_renames \
		--deny clippy::print_stderr \
		--deny clippy::print_stdout \
		--deny clippy::rc_buffer \
		--deny clippy::rc_mutex \
		--deny clippy::ref_patterns \
		--deny clippy::unwrap_used

@_vet_verify_project:
	echo 'Running "cargo verify-project"...'
	cargo verify-project \
		--quiet \
		{{CI_ONLY_CARGO_ARGS}}

# --------------------------------------------------------------------------------------------------

[private]
@ci-audit:
	just ci={{TRUE}} audit

[private]
@ci-build:
	just ci={{TRUE}} build-each

[private]
@ci-compliance:
	just ci={{TRUE}} compliance

[private]
@ci-coverage:
	just ci={{TRUE}} \
		test_features={{ALL_TEST_FEATURES}} \
		coverage

[private]
@ci-docs:
	just ci={{TRUE}} docs

[private]
@ci-fmt:
	just ci={{TRUE}} fmt-check

[private]
@ci-mutation:
	just ci={{TRUE}} \
		test_features={{ALL_TEST_FEATURES}} \
		mutation

[private]
@ci-test:
	just ci={{TRUE}} \
		test_features={{ALL_TEST_FEATURES}} \
		test-each

[private]
@ci-vet:
	just ci={{TRUE}} vet

# --------------------------------------------------------------------------------------------------

TRUE := "1"
FALSE := "0"

ci := FALSE

STD_BUILD_ARGS := "--release"
STD_COVERAGE_ARGS :=  "--count --line --engine llvm --out html --output-dir _reports/coverage/"
STD_DOCS_ARGS := "--document-private-items"
STD_TEST_ARGS := ""

CI_ONLY_CARGO_ARGS := if ci == TRUE { "--locked" } else { "" }
CI_ONLY_COVERAGE_ARGS := if ci == TRUE { "--out lcov" } else { "" }
CI_ONLY_TEST_ARGS := if ci == TRUE { "--no-fail-fast" } else { "" }

ALL_TEST_FEATURES := "test-dangerous,test-symlink,test-trash"

BUILD_ARGS := STD_BUILD_ARGS + " " + CI_ONLY_CARGO_ARGS
COVERAGE_ARGS := STD_COVERAGE_ARGS + " " + CI_ONLY_CARGO_ARGS + " " + CI_ONLY_COVERAGE_ARGS
DOCS_ARGS := STD_DOCS_ARGS + " " + CI_ONLY_CARGO_ARGS
TEST_ARGS := STD_TEST_ARGS + " " + CI_ONLY_TEST_ARGS + " " + CI_ONLY_CARGO_ARGS
TEST_INTEGRATION_ARGS := "--test '*'"
TEST_UNIT_ARGS := "--bins"

features := FALSE
FEATURES := if features == FALSE {
	""
} else {
	"--no-default-features " + if features == "" { "" } else { "--features " + features }
}

test_features := ""
TEST_FEATURES := if test_features == "" { "" } else { "--features " + test_features }
