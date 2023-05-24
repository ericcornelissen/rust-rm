// SPDX-License-Identifier: Apache-2.0

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![deny(rustdoc::invalid_html_tags)]
#![deny(rustdoc::bare_urls)]

//! A CLI like the `rm(1)` Unix command but more modern and designed for humans. Aims to provide an
//! `rm` command that feels familiar yet is safer and more user friendly.

use std::process::ExitCode;

/// Run with arguments passed via the CLI.
fn main() -> ExitCode {
    let raw_args = std::env::args();
    let raw_vars = std::env::vars();

    let vars = cli::parse_vars(raw_vars);
    let args = cli::parse_args(raw_args, vars).unwrap_or_else(|err| err.exit());

    match cli::run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}

/// Programmatic interface for the CLI.
mod cli {
    use super::{lang, logging, rm, transform, walk};

    use clap::error::Error;
    use clap::Parser;
    use log::{error, info, trace};
    use owo_colors::OwoColorize;

    #[cfg(test)]
    use proptest_derive::Arbitrary;

    /// Remove (unlink) the PATH(s) - v23.04
    ///
    /// Does not remove anything by default, use either the option --force or --interactive to
    /// perform the removal.
    ///
    /// Also does not remove directories by default, use the option --dir to remove empty
    /// directories or the option --recursive to remove directories and their contents.
    ///
    /// To remove a file whose name starts with a '-', for example '-foo', use either the special
    /// '--' option or prefix the path with './'.
    ///
    /// If you use rm to remove a file, it might be possible to recover some of its contents, given
    /// sufficient expertise and/or time. For greater assurance that the contents are truly
    /// unrecoverable, consider using shred(1).
    #[derive(Parser)]
    #[command(name = "rm", version = None)]
    #[command(about = "Remove (unlink) the PATH(s)", long_about)]
    #[allow(clippy::struct_excessive_bools)]
    pub struct Args {
        /// Ignore nonexistent files and directories.
        #[arg(short = 'b', long)]
        blind: bool,

        /// Remove empty directories.
        #[arg(short = 'd', long, group = "dirs")]
        dir: bool,

        /// Remove without prompt.
        #[arg(short = 'f', long, group = "method")]
        force: bool,

        /// Prompt to remove.
        ///
        /// Answer "Y" or "yes" to remove an entry or "n" or "no" to keep it. Any other input will
        /// be ignored and the entry skipped.
        #[arg(short = 'i', long, group = "method")]
        interactive: bool,

        /// Do not treat the file system root specially.
        #[arg(short = None, long)]
        no_preserve_root: bool,

        /// Don't output to stdout.
        ///
        /// Only has an effect when used with --force.
        #[arg(short = 'q', long, group = "verbosity")]
        quiet: bool,

        /// Recursively remove directories and their contents.
        #[arg(short = 'r', long, group = "dirs")]
        recursive: bool,

        /// Move to the trash bin instead of removing.
        #[cfg(feature = "trash")]
        #[arg(short = 't', long)]
        trash: bool,

        /// Explain what is being done.
        #[arg(short = 'v', long, group = "verbosity")]
        verbose: bool,

        /// The paths to remove.
        paths: Vec<String>,
    }

    /// Tests for the [`Args`] struct.
    #[cfg(test)]
    mod test_args {
        use super::Args;

        use clap::CommandFactory;

        #[test]
        fn clap_verification() {
            Args::command().debug_assert();
        }
    }

    /// The `Result` type for parsing CLI arguments.
    type ParseResult = Result<Args, Error>;

    /// Parse arguments for the CLI.
    ///
    /// # Errors
    ///
    /// If the given arguments couldn't be parsed.
    pub fn parse_args<T>(args: T, vars: Vars) -> ParseResult
    where
        T: IntoIterator<Item = String>,
    {
        let mut args = Args::try_parse_from(args)?;

        #[cfg(feature = "classic")]
        if vars.classic {
            args = parse_args_classic(args)?;
        }

        if vars.debug {
            args.verbose = true;
        }

        Ok(args)
    }

    /// Tests for the [`parse_args`] function.
    #[cfg(test)]
    mod test_parse_args {
        use super::test_helpers::{parse_args, TestArgs, TestArgsAndIndex};

        use super::Vars;

        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[proptest]
        fn paths(args: TestArgs, vars: Vars) {
            let args = args.inner();

            let options = args.iter().take_while(|arg| **arg != "--");
            let operands = args.iter().skip_while(|arg| **arg != "--").skip(1);
            let expected: Vec<String> = options
                .filter(|arg| !arg.starts_with('-'))
                .chain(operands)
                .map(String::from)
                .collect();

            match parse_args(args, vars) {
                Ok(args) => prop_assert_eq!(args.paths, expected),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn blind_long_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!vars.is_classic());

            match parse_args(args.insert("--blind"), vars) {
                Ok(args) => prop_assert!(args.blind),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn blind_short_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!vars.is_classic());

            match parse_args(args.insert("-b"), vars) {
                Ok(args) => prop_assert!(args.blind),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn not_blind(args: TestArgs, vars: Vars) {
            prop_assume!(!args.contains("--blind"));
            prop_assume!(!args.contains("-b"));
            prop_assume!(!vars.is_classic());

            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(!args.blind),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn dir_long_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));

            match parse_args(args.insert("--dir"), vars) {
                Ok(args) => prop_assert!(args.dir),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn dir_short_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));

            match parse_args(args.insert("-d"), vars) {
                Ok(args) => prop_assert!(args.dir),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn not_dir(args: TestArgs, vars: Vars) {
            prop_assume!(!args.contains("--dir"));
            prop_assume!(!args.contains("-d"));

            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(!args.dir),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn force_long_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!vars.is_classic());

            match parse_args(args.insert("--force"), vars) {
                Ok(args) => prop_assert!(args.force),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn force_short_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!vars.is_classic());

            match parse_args(args.insert("-f"), vars) {
                Ok(args) => prop_assert!(args.force),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn not_force(args: TestArgs, vars: Vars) {
            prop_assume!(!args.contains("--force"));
            prop_assume!(!args.contains("-f"));
            prop_assume!(!vars.is_classic());

            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(!args.force),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn interactive_long_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));

            match parse_args(args.insert("--interactive"), vars) {
                Ok(args) => prop_assert!(args.interactive),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn interactive_short_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));

            match parse_args(args.insert("-i"), vars) {
                Ok(args) => prop_assert!(args.interactive),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn not_interactive(args: TestArgs, vars: Vars) {
            prop_assume!(!args.contains("--interactive"));
            prop_assume!(!args.contains("-i"));

            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(!args.interactive),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn no_preserve_root_long_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));

            match parse_args(args.insert("--no-preserve-root"), vars) {
                Ok(args) => prop_assert!(args.no_preserve_root),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn not_no_preserve_root(args: TestArgs, vars: Vars) {
            prop_assume!(!args.contains("--no-preserve-root"));

            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(!args.no_preserve_root),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn quiet_long_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!vars.is_classic());

            match parse_args(args.insert("--quiet"), vars) {
                Ok(args) => prop_assert!(args.quiet),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn quiet_short_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!vars.is_classic());

            match parse_args(args.insert("-q"), vars) {
                Ok(args) => prop_assert!(args.quiet),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn not_quiet(args: TestArgs, vars: Vars) {
            prop_assume!(!args.contains("--quiet"));
            prop_assume!(!args.contains("-q"));
            prop_assume!(!vars.is_classic());

            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(!args.quiet),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn recursive_long_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));

            match parse_args(args.insert("--recursive"), vars) {
                Ok(args) => prop_assert!(args.recursive),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn recursive_short_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));

            match parse_args(args.insert("-r"), vars) {
                Ok(args) => prop_assert!(args.recursive),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn not_recursive(args: TestArgs, vars: Vars) {
            prop_assume!(!args.contains("--recursive"));
            prop_assume!(!args.contains("-r"));

            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(!args.recursive),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        #[cfg(feature = "trash")]
        fn trash_long_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!vars.is_classic());

            match parse_args(args.insert("--trash"), vars) {
                Ok(args) => prop_assert!(args.trash),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        #[cfg(feature = "trash")]
        fn trash_short_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!vars.is_classic());

            match parse_args(args.insert("-t"), vars) {
                Ok(args) => prop_assert!(args.trash),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        #[cfg(feature = "trash")]
        fn not_trash(args: TestArgs, vars: Vars) {
            prop_assume!(!args.contains("--trash"));
            prop_assume!(!args.contains("-t"));
            prop_assume!(!vars.is_classic());

            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(!args.trash),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn verbose_long_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!vars.debug);

            match parse_args(args.insert("--verbose"), vars) {
                Ok(args) => prop_assert!(args.verbose),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn verbose_short_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!vars.debug);

            match parse_args(args.insert("-v"), vars) {
                Ok(args) => prop_assert!(args.verbose),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn not_verbose(args: TestArgs, vars: Vars) {
            prop_assume!(!args.contains("--verbose"));
            prop_assume!(!args.contains("-v"));
            prop_assume!(!vars.debug);

            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(!args.verbose),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn debug_not_verbose(args: TestArgs, vars: Vars) {
            prop_assume!(!args.contains("--verbose"));
            prop_assume!(!args.contains("-v"));
            prop_assume!(vars.debug);

            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(args.verbose),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn debug_and_verbose(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(vars.debug);

            match parse_args(args.insert("--verbose"), vars) {
                Ok(args) => prop_assert!(args.verbose),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn disallow_dir_with_recursive(vars: Vars) {
            let out = test_combination_errors(("dir", 'd'), ("recursive", 'r'), vars);
            prop_assert!(out.is_err());
        }

        #[proptest]
        fn disallow_force_with_interactive(vars: Vars) {
            let out = test_combination_errors(("force", 'f'), ("interactive", 'i'), vars);
            prop_assert!(out.is_err());
        }

        #[proptest]
        fn disallow_quiet_with_verbose(vars: Vars) {
            let out = test_combination_errors(("quiet", 'q'), ("verbose", 'v'), vars);
            prop_assert!(out.is_err());
        }

        /// Type representing the long and short names of a flag.
        type FlagPair<'a> = (&'a str, char);

        /// Test that parsing arguments with both `flag1` and `flag2` present always errors.
        ///
        /// # Example
        ///
        /// ```no_run
        /// use cli::Vars;
        ///
        /// let vars = Vars { classic: false, debug: false };
        /// test_combination_errors(("long-a", 'a'), ("long-b", 'b'), vars)?;
        /// ```
        fn test_combination_errors(flag1: FlagPair, flag2: FlagPair, vars: Vars) -> Result<(), ()> {
            let cases = [
                vec![format!("--{}", flag1.0), format!("--{}", flag2.0)],
                vec![format!("--{}", flag1.0), format!("-{}", flag2.1)],
                vec![format!("-{}", flag1.1), format!("--{}", flag2.0)],
                vec![format!("-{}", flag1.1), format!("-{}", flag2.1)],
                vec![format!("--{}", flag2.0), format!("--{}", flag1.0)],
                vec![format!("--{}", flag2.0), format!("-{}", flag1.1)],
                vec![format!("-{}", flag2.1), format!("--{}", flag1.0)],
                vec![format!("-{}", flag2.1), format!("-{}", flag1.1)],
                vec![format!("-{}{}", flag1.1, flag2.1)],
                vec![format!("-{}{}", flag2.1, flag1.1)],
            ];

            for args in cases {
                let out = parse_args(args.clone(), vars);
                if out.is_err() {
                    return Err(());
                }
            }

            Ok(())
        }
    }

    /// Parse arguments for the CLI with classic mode enabled, modifying the given `args` in place.
    ///
    /// # Errors
    ///
    /// If an unsupported flags is used, but only if the `force` option isn't set.
    #[cfg(feature = "classic")]
    fn parse_args_classic(mut args: Args) -> ParseResult {
        use clap::error::ErrorKind;

        macro_rules! check_use_of_invalid_flag {
            ($flag:ident) => {
                if args.$flag {
                    return Err(Error::raw(
                        ErrorKind::UnknownArgument,
                        format!("option --{} not supported in classic mode\n", stringify!($flag)),
                    ));
                }
            };
        }

        if !args.force {
            check_use_of_invalid_flag!(blind);
            check_use_of_invalid_flag!(quiet);
            #[cfg(feature = "trash")]
            check_use_of_invalid_flag!(trash);
        }

        args.blind = args.force; // rm(1) behaves blindly with --force
        args.force = !args.interactive; // rm(1) removes unless --interactive
        args.quiet = true; // rm(1) is always --quiet
        #[cfg(feature = "trash")]
        {
            args.trash = false; // rm(1) does not support --trash
        }

        Ok(args)
    }

    /// Tests for the [`parse_args_classic`] function.
    #[cfg(test)]
    #[cfg(feature = "classic")]
    mod test_parse_args_classic {
        use super::test_helpers::{TestArgs, TestArgsAndIndex};

        use super::Vars;

        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[proptest]
        fn blind_when_force_long_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));

            match parse_args(args.insert("--force"), vars) {
                Ok(args) => prop_assert!(args.blind),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn blind_when_force_short_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));

            match parse_args(args.insert("-f"), vars) {
                Ok(args) => prop_assert!(args.blind),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn not_blind_when_not_force(args: TestArgs, vars: Vars) {
            prop_assume!(!args.contains("--force"));
            prop_assume!(!args.contains("-f"));

            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(!args.blind),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn dir_when_dir_long_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));

            match parse_args(args.insert("--dir"), vars) {
                Ok(args) => prop_assert!(args.dir),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn dir_when_dir_short_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));

            match parse_args(args.insert("-d"), vars) {
                Ok(args) => prop_assert!(args.dir),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn not_dir_when_not_dir(args: TestArgs, vars: Vars) {
            prop_assume!(!args.contains("--dir"));
            prop_assume!(!args.contains("-d"));

            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(!args.dir),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn force_when_not_interactive(args: TestArgs, vars: Vars) {
            prop_assume!(!args.contains("--interactive"));
            prop_assume!(!args.contains("-i"));

            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(args.force),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn not_force_when_interactive_long_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));

            match parse_args(args.insert("--interactive"), vars) {
                Ok(args) => prop_assert!(!args.force),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn not_force_when_interactive_short_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));

            match parse_args(args.insert("-i"), vars) {
                Ok(args) => prop_assert!(!args.force),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn always_quiet(args: TestArgs, vars: Vars) {
            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(args.quiet),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        #[cfg(feature = "trash")]
        fn never_trash(args: TestArgs, vars: Vars) {
            match parse_args(args.inner(), vars) {
                Ok(args) => prop_assert!(!args.trash),
                Err(_) => prop_assume!(false),
            }
        }

        #[proptest]
        fn disallow_blind_full_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!args.contains("--force"));
            prop_assume!(!args.contains("-f"));

            prop_assert!(parse_args(args.insert("--blind"), vars).is_err());
        }

        #[proptest]
        fn disallow_blind_short_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!args.contains("--force"));
            prop_assume!(!args.contains("-f"));

            prop_assert!(parse_args(args.insert("-b"), vars).is_err());
        }

        #[proptest]
        fn disallow_quiet_full_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!args.contains("--force"));
            prop_assume!(!args.contains("-f"));

            prop_assert!(parse_args(args.insert("--quiet"), vars).is_err());
        }

        #[proptest]
        fn disallow_quiet_short_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!args.contains("--force"));
            prop_assume!(!args.contains("-f"));

            prop_assert!(parse_args(args.insert("-q"), vars).is_err());
        }

        #[proptest]
        #[cfg(feature = "trash")]
        fn disallow_trash_full_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!args.contains("--force"));
            prop_assume!(!args.contains("-f"));

            prop_assert!(parse_args(args.insert("--trash"), vars).is_err());
        }

        #[proptest]
        #[cfg(feature = "trash")]
        fn disallow_trash_short_name(args: TestArgsAndIndex, vars: Vars) {
            prop_assume!(!args.has_arg_before_index("--"));
            prop_assume!(!args.contains("--force"));
            prop_assume!(!args.contains("-f"));

            prop_assert!(parse_args(args.insert("-t"), vars).is_err());
        }

        /// Convenience wrapper to parse arguments using [`super::parse_args`]. Always sets
        /// `vars.classic` to `true`.
        ///
        /// See also [`super::test_helpers::parse_args`].
        fn parse_args(args: Vec<String>, vars: Vars) -> super::test_helpers::ParseResult {
            super::test_helpers::parse_args(args, Vars { classic: true, ..vars })
        }
    }

    /// The environment variable name to enable compatibility mode with the `rm(1)` Unix command.
    #[cfg(feature = "classic")]
    const CLASSIC_MODE: &str = "RUST_RM_CLASSIC";

    /// A standard environment variable name to enable verbose mode.
    const DEBUG_MODE: &str = "DEBUG";

    /// Struct representing parsed environment configuration values.
    #[cfg_attr(test, derive(Arbitrary, Clone, Copy, Debug))]
    pub struct Vars {
        /// The environment configuration value for classic mode.
        #[cfg(feature = "classic")]
        classic: bool,

        /// The environment configuration value for debug mode.
        debug: bool,
    }

    /// Parse environment variables for the CLI.
    pub fn parse_vars<T>(vars: T) -> Vars
    where
        T: IntoIterator<Item = (String, String)>,
    {
        let vars: Vec<String> = vars.into_iter().map(|(name, _)| name).collect();
        Vars {
            #[cfg(feature = "classic")]
            classic: vars.contains(&CLASSIC_MODE.to_owned()),
            debug: vars.contains(&DEBUG_MODE.to_owned()),
        }
    }

    /// Tests for the [`parse_vars`] function.
    #[cfg(test)]
    mod test_parse_vars {
        use super::test_helpers::{TestVars, TestVarsAndIndex};

        use super::parse_vars;

        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[proptest]
        #[cfg(feature = "classic")]
        fn classic_not_set(vars: TestVars) {
            prop_assume!(!vars.contains_key(super::CLASSIC_MODE));

            let out = parse_vars(vars.inner());
            prop_assert!(!out.classic);
        }

        #[proptest]
        #[cfg(feature = "classic")]
        fn classic_set(vars: TestVarsAndIndex, val: String) {
            let out = parse_vars(vars.insert((super::CLASSIC_MODE, &val)));
            prop_assert!(out.classic);
        }

        #[proptest]
        fn debug_not_set(vars: TestVars) {
            prop_assume!(!vars.contains_key(super::DEBUG_MODE));

            let out = parse_vars(vars.inner());
            prop_assert!(!out.debug);
        }

        #[proptest]
        fn debug_set(vars: TestVarsAndIndex, val: String) {
            let out = parse_vars(vars.insert((super::DEBUG_MODE, &val)));
            prop_assert!(out.debug);
        }
    }

    /// Run the CLI with the given (parsed) arguments.
    ///
    /// See also [`parse_args`].
    ///
    /// # Errors
    ///
    /// If there is a CLI runtime error.
    pub fn run(args: &Args) -> Result<(), ()> {
        let dry_run = !args.force && !args.interactive;

        logging::configure(&if args.quiet && !dry_run {
            logging::Verbosity::Quiet
        } else if args.verbose {
            logging::Verbosity::Verbose
        } else {
            logging::Verbosity::Normal
        });

        let transformers: [transform::Transformer; 5] = [
            transform::disallow_current_and_parent_dir,
            if args.no_preserve_root { transform::identity } else { transform::disallow_root },
            if args.blind { transform::skip_missing_entry } else { transform::tip_not_found },
            match (args.dir, args.recursive) {
                (false, false) => transform::disallow_all_dirs,
                (true, false) => transform::disallow_filled_dirs,
                (_, true) => transform::identity,
            },
            if args.interactive { transform::interactive } else { transform::identity },
        ];

        #[cfg(feature = "trash")]
        let walk = if args.recursive && !args.trash {
            walk::recurse(transformers)
        } else {
            walk::given(transformers)
        };
        #[cfg(not(feature = "trash"))]
        let walk =
            if args.recursive { walk::recurse(transformers) } else { walk::given(transformers) };

        #[cfg(feature = "trash")]
        let remove = match (dry_run, args.trash) {
            (false, false) => rm::remove,
            (false, true) => rm::dispose,
            (true, false) => rm::show_remove,
            (true, true) => rm::show_dispose,
        };
        #[cfg(not(feature = "trash"))]
        let remove = if dry_run { rm::show_remove } else { rm::remove };

        trace!("start processing");
        let (removed, errored): (usize, usize) = args
            .paths
            .iter()
            .flat_map(|path| walk(path))
            .inspect(|result| {
                if let Ok(entry) = result {
                    if let Some(reason) = entry.skip_reason() {
                        trace!("skipped {entry}: {reason}");
                    }
                }
            })
            .filter(|result| match result {
                Ok(entry) => !entry.is_skipped(),
                Err(_) => true,
            })
            .map(|result| match result {
                Ok(entry) => remove(entry),
                Err(err) => Err(err),
            })
            .inspect(|result| match result {
                Ok(msg) => info!("{msg}"),
                Err(err) => error!("{err}"),
            })
            .fold((0, 0), |(oks, errs), result| match result {
                Ok(_) => (oks.checked_add(1).unwrap_or(usize::MAX), errs),
                Err(_) => (oks, errs.checked_add(1).unwrap_or(usize::MAX)),
            });

        info!(
            "{}{removed} {}{}, {} occurred",
            if removed > 0 || errored > 0 || args.verbose { "\n" } else { "" },
            if dry_run { "would be removed" } else { "removed" },
            if dry_run && removed > 0 {
                format!(" {}", "(use '--force' to remove)".italic())
            } else {
                String::new()
            },
            lang::pluralize("error", errored),
        );

        if errored > 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Helpers for writing unit tests in or using this module.
    #[cfg(test)]
    mod test_helpers {
        use super::Vars;

        use proptest::prelude::*;

        /// Utility functionality for working with [`Vars`] in tests.
        impl Vars {
            /// Check if [`Vars::classic`] is set to true.
            #[cfg(feature = "classic")]
            pub fn is_classic(self) -> bool {
                self.classic
            }

            /// Always returns `false` (because the "classic" feature is off).
            #[cfg(not(feature = "classic"))]
            pub fn is_classic(self) -> bool {
                false
            }
        }

        /// The `Result` type for parsing args for tests.
        pub type ParseResult = Result<super::Args, ()>;

        /// Convenience wrapper to parse arguments using [`super::parse_args`] for testing purposes.
        ///
        /// # Errors
        ///
        /// If the given arguments couldn't be parsed.
        ///
        /// # Example
        ///
        /// ```no_run
        /// use cli::Vars;
        ///
        /// let args = vec!["--foo", "bar"];
        /// let vars = Vars { classic: false, debug: false };
        /// let out = parse_args(args, vars);
        /// assert!(out.is_err());
        /// ```
        pub fn parse_args(mut args: Vec<String>, vars: Vars) -> ParseResult {
            args.insert(0, "rm".to_owned());
            match super::parse_args(args, vars) {
                Ok(args) => Ok(args),
                Err(_) => Err(()),
            }
        }

        /// Struct wrapping a [`String`] that implements [`Arbitrary`] to generate a valid argument
        /// for the CLI.
        #[derive(Clone, Debug)]
        struct TestArg(String);

        impl TestArg {
            /// Returns the contained value, consuming the `self` value.
            fn inner(self) -> String {
                self.0
            }
        }

        impl Arbitrary for TestArg {
            type Parameters = ();
            type Strategy = BoxedStrategy<Self>;

            fn arbitrary_with(_: ()) -> Self::Strategy {
                const KNOWN_FLAG_PATTERN: &str = "\
                    --blind|-b|\
                    --dir|-d|\
                    --force|-f|\
                    --interactive|-i|\
                    --no_preserver_root|\
                    --one_file_system|\
                    --quiet|-q|\
                    --recursive|-r|\
                    --trash|-t|\
                    --verbose|-v|\
                    --\
                ";
                const NON_FLAG_PATTERN: &str = "[^-].*";

                let strategies = vec![(1, KNOWN_FLAG_PATTERN), (10, NON_FLAG_PATTERN)];

                prop::strategy::Union::new_weighted(strategies).prop_map(Self).boxed()
            }
        }

        /// Struct wrapping a list of [`String`]s that implements [`Arbitrary`] to generate valid
        /// lists of arguments for the CLI.
        ///
        /// See also [`TestArg`].
        #[derive(Clone, Debug)]
        pub struct TestArgs(Vec<String>);

        impl TestArgs {
            /// Returns `true` if the contained list contains the given value.
            pub fn contains(&self, arg: &str) -> bool {
                self.0.contains(&arg.to_owned())
            }

            /// Returns the contained value, consuming the `self` value.
            pub fn inner(self) -> Vec<String> {
                self.0
            }
        }

        impl Arbitrary for TestArgs {
            type Parameters = ();
            type Strategy = BoxedStrategy<Self>;

            fn arbitrary_with(_: ()) -> Self::Strategy {
                let size_range = 1..=16;
                prop::collection::vec(TestArg::arbitrary(), size_range)
                    .prop_map(|v| Self(v.into_iter().map(TestArg::inner).collect()))
                    .boxed()
            }
        }

        /// Struct wrapping a list of [`String`]s that implements [`Arbitrary`] to generate valid
        /// lists of arguments for the CLI. To use the contained value one more [`String`] has to be
        /// inserted.
        ///
        /// See also [`TestArgs`].
        #[derive(Clone, Debug)]
        pub struct TestArgsAndIndex(Vec<String>, usize);

        impl TestArgsAndIndex {
            /// Returns the contained value with the given value at the associated index, consuming
            /// the `self` value.
            pub fn insert(self, arg: &str) -> Vec<String> {
                let Self(mut args, index) = self;
                args.insert(index, arg.to_owned());
                args
            }

            /// Returns `true` if the given value occurs in the list of arguments, and `false`
            /// otherwise.
            pub fn contains(&self, val: &str) -> bool {
                self.0.iter().any(|arg| arg == val)
            }

            /// Returns `true` if the given value occurs in the list of arguments before the
            /// associated index, and `false` otherwise.
            pub fn has_arg_before_index(&self, val: &str) -> bool {
                self.0.iter().take(self.1).any(|arg| arg == val)
            }
        }

        impl Arbitrary for TestArgsAndIndex {
            type Parameters = ();
            type Strategy = BoxedStrategy<Self>;

            fn arbitrary_with(_: ()) -> Self::Strategy {
                let size_range = 1..=16;
                prop::collection::vec(TestArg::arbitrary(), size_range)
                    .prop_flat_map(|vec| (0..vec.len(), Just(vec)))
                    .prop_map(|(i, vec)| Self(vec.into_iter().map(TestArg::inner).collect(), i))
                    .boxed()
            }
        }

        /// Struct wrapping a [`String`]-based (key, value) pair that implements [`Arbitrary`] to
        /// generate a valid environment variable for the CLI.
        #[derive(Clone, Debug)]
        struct TestVar((String, String));

        impl TestVar {
            /// Returns the contained value, consuming the `self` value.
            fn inner(self) -> (String, String) {
                self.0
            }
        }

        impl Arbitrary for TestVar {
            type Parameters = ();
            type Strategy = BoxedStrategy<Self>;

            fn arbitrary_with(_: ()) -> Self::Strategy {
                const KNOWN_VAR_PATTERN: &str = "RUST_RM_CLASSIC|DEBUG";
                const GENERAL_VAR_PATTERN: &str = "[a-zA-Z_]+";

                let strategies = vec![(1, KNOWN_VAR_PATTERN), (10, GENERAL_VAR_PATTERN)];

                (prop::strategy::Union::new_weighted(strategies), String::arbitrary())
                    .prop_map(|(key, val)| TestVar((key, val)))
                    .boxed()
            }
        }

        /// Struct wrapping a list of [`String`]-based (key, value) pairs that implements
        /// [`Arbitrary`] to generate valid environment variables for the CLI.
        ///
        /// See also [`TestVar`].
        #[derive(Clone, Debug)]
        pub struct TestVars(Vec<(String, String)>);

        impl TestVars {
            /// Returns `true` if the contained list contains the given key.
            pub fn contains_key(&self, key: &str) -> bool {
                self.0.iter().map(|(name, _)| name).any(|name| name == key)
            }

            /// Returns the contained value, consuming the `self` value.
            pub fn inner(self) -> Vec<(String, String)> {
                self.0
            }
        }

        impl Arbitrary for TestVars {
            type Parameters = ();
            type Strategy = BoxedStrategy<Self>;

            fn arbitrary_with(_: ()) -> Self::Strategy {
                let size_range = 1..=16;
                prop::collection::vec(TestVar::arbitrary(), size_range)
                    .prop_map(|v| Self(v.into_iter().map(TestVar::inner).collect()))
                    .boxed()
            }
        }

        /// Struct wrapping a list of [`String`]-based (key, value) pairs that implements
        /// [`Arbitrary`] to generate valid environment variables for the CLI. To use the contained
        /// value one more pair has to be inserted.
        ///
        /// See also [`TestVars`].
        #[derive(Clone, Debug)]
        pub struct TestVarsAndIndex(Vec<(String, String)>, usize);

        impl TestVarsAndIndex {
            /// Returns the contained value with the given value at the associated index, consuming
            /// the `self` value.
            pub fn insert(self, env_var: (&str, &str)) -> Vec<(String, String)> {
                let Self(mut vars, index) = self;
                vars.insert(index, (env_var.0.to_owned(), env_var.1.to_owned()));
                vars
            }
        }

        impl Arbitrary for TestVarsAndIndex {
            type Parameters = ();
            type Strategy = BoxedStrategy<Self>;

            fn arbitrary_with(_: ()) -> Self::Strategy {
                let size_range = 1..=16;
                prop::collection::vec(TestVar::arbitrary(), size_range)
                    .prop_flat_map(|vec| (0..vec.len(), Just(vec)))
                    .prop_map(|(i, vec)| Self(vec.into_iter().map(TestVar::inner).collect(), i))
                    .boxed()
            }
        }
    }
}

/// File system utilities.
mod fs {
    use std::ffi::OsString;
    use std::fs::{read_dir, symlink_metadata, File};
    use std::io::{self, Read};
    use std::path::{Path, PathBuf};

    use log::trace;
    use owo_colors::OwoColorize;

    #[cfg(test)]
    use proptest_derive::Arbitrary;

    /// The `Result` type for interacting with the file system.
    pub type Result = std::result::Result<Entry, Error>;

    /// Open a handle for a file system [`Entry`].
    ///
    /// # Errors
    ///
    /// If nothing is accessible at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result {
        match symlink_metadata(&path) {
            Ok(metadata) if metadata.is_file() => {
                trace!("found file at {}", path.as_ref().display());
                Ok(Entry::new(path, EntryKind::File))
            },
            Ok(metadata) if metadata.is_dir() => {
                trace!("found directory at {}", path.as_ref().display());
                Ok(Entry::new(path, EntryKind::Dir))
            },
            Ok(metadata) if metadata.is_symlink() => {
                trace!("found symbolic link at {}", path.as_ref().display());
                Ok(Entry::new(path, EntryKind::Symlink))
            },
            Err(err) => {
                trace!("found nothing at {}", path.as_ref().display());
                Err(Error::new(path, err.kind().into()))
            },
            Ok(_) => unreachable!(),
        }
    }

    /// Tests for the [`open`] function.
    #[cfg(test)]
    mod test_open {
        use crate::test_helpers::{with_test_dir, TestResult};

        use super::{open, Entry, EntryKind, Error, ErrorKind};

        use assert_fs::prelude::*;

        #[test]
        fn file() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;

                let path = file.path();

                let out = open(path);
                assert_eq!(out, Ok(Entry::new(path, EntryKind::File)));

                Ok(())
            })
        }

        #[test]
        fn dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;

                let path = dir.path();

                let out = open(path);
                assert_eq!(out, Ok(Entry::new(path, EntryKind::Dir)));

                Ok(())
            })
        }

        #[test]
        fn symlink() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;
                let link = test_dir.child("link");
                link.symlink_to_file(&file)?;

                let path = link.path();

                let out = open(path);
                assert_eq!(out, Ok(Entry::new(path, EntryKind::Symlink)));

                Ok(())
            })
        }

        #[test]
        fn not_found() -> TestResult {
            with_test_dir(|test_dir| {
                let path = test_dir.child("missing");

                let out = open(&path);
                assert_eq!(out, Err(Error::new(path, ErrorKind::NotFound)));

                Ok(())
            })
        }
    }

    /// Check if the [`Entry`] is an empty file or directory.
    pub fn is_empty(entry: &Entry) -> bool {
        match entry.kind() {
            EntryKind::Dir => {
                read_dir(entry.path()).map_or(true, |mut content| content.next().is_none())
            },
            EntryKind::File => File::open(entry.path())
                .map_or(true, |mut f| f.read(&mut [0; 1]).map_or(true, |n| n == 0)),
            EntryKind::Symlink => true,
        }
    }

    #[cfg(test)]
    mod test_is_empty {
        use crate::test_helpers::{with_test_dir, TestResult};

        use super::{is_empty, Entry, EntryKind};

        use assert_fs::prelude::*;

        #[test]
        fn file_empty() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;

                let path = file.path();

                let entry = Entry::new(path, EntryKind::File);
                assert!(is_empty(&entry));

                Ok(())
            })
        }

        #[test]
        fn file_filled() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.write_str("Hello world!")?;

                let path = file;

                let entry = Entry::new(path, EntryKind::File);
                assert!(!is_empty(&entry));

                Ok(())
            })
        }

        #[test]
        fn missing() -> TestResult {
            with_test_dir(|test_dir| {
                let path = test_dir.child("missing");

                let entry = Entry::new(path, EntryKind::File);
                assert!(is_empty(&entry));

                Ok(())
            })
        }

        #[test]
        fn dir_empty() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;

                let path = dir.path();

                let entry = Entry::new(path, EntryKind::Dir);
                assert!(is_empty(&entry));

                Ok(())
            })
        }

        #[test]
        fn dir_filled() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                dir.child("file").touch()?;

                let path = dir.path();

                let entry = Entry::new(path, EntryKind::Dir);
                assert!(!is_empty(&entry));

                Ok(())
            })
        }

        #[test]
        fn dir_missing() -> TestResult {
            with_test_dir(|test_dir| {
                let path = test_dir.child("missing");

                let entry = Entry::new(path, EntryKind::Dir);
                assert!(is_empty(&entry));

                Ok(())
            })
        }

        #[test]
        fn symlink_to_empty_file() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;
                let link = test_dir.child("link");
                link.symlink_to_file(&file)?;

                let path = link.path();

                let entry = Entry::new(path, EntryKind::Symlink);
                assert!(is_empty(&entry));

                Ok(())
            })
        }

        #[test]
        fn symlink_to_filled_file() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.write_str("Hello world!")?;
                let link = test_dir.child("link");
                link.symlink_to_file(&file)?;

                let path = link.path();

                let entry = Entry::new(path, EntryKind::Symlink);
                assert!(is_empty(&entry));

                Ok(())
            })
        }

        #[test]
        fn symlink_to_empty_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                let link = test_dir.child("link");
                link.symlink_to_file(&dir)?;

                let path = link.path();

                let entry = Entry::new(path, EntryKind::Symlink);
                assert!(is_empty(&entry));

                Ok(())
            })
        }

        #[test]
        fn symlink_to_filled_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                dir.child("file").touch()?;
                let link = test_dir.child("link");
                link.symlink_to_file(&dir)?;

                let path = link.path();

                let entry = Entry::new(path, EntryKind::Symlink);
                assert!(is_empty(&entry));

                Ok(())
            })
        }

        #[test]
        fn symlink_missing() -> TestResult {
            with_test_dir(|test_dir| {
                let path = test_dir.child("missing");

                let entry = Entry::new(path, EntryKind::Symlink);
                assert!(is_empty(&entry));

                Ok(())
            })
        }
    }

    /// Struct representing a file system entry.
    #[cfg_attr(test, derive(Arbitrary, Clone, Debug, Eq, PartialEq))]
    pub struct Entry {
        /// The kind of file system entry.
        kind: EntryKind,

        /// The path to the file system entry.
        path: OsString,

        /// Why, if at all, the CLI must skip the file system entry.
        skip_reason: Option<String>,

        /// Whether or not the CLI visited the file system entry before.
        visited: bool,
    }

    impl Entry {
        /// Create a new [`Entry`].
        fn new<P: AsRef<Path>>(path: P, kind: EntryKind) -> Self {
            Self {
                kind,
                path: path.as_ref().as_os_str().to_owned(),
                skip_reason: None,
                visited: false,
            }
        }

        /// Convert the [`Entry`] into an [`Error`] for the [`Entry`]'s path with the given
        /// [`ErrorKind`].
        pub fn into_err(self, kind: ErrorKind) -> Error {
            Error::new(self.path(), kind)
        }

        /// Convert the [`Entry`] into an [`Entry`] that will not be removed. Must be provided with
        /// the reason why it is skipped.
        pub fn into_skipped(mut self, reason: &str) -> Self {
            self.skip_reason = Some(reason.to_owned());
            self
        }

        /// Convert the [`Entry`] into an [`Entry`] that's marked as visited.
        pub fn into_visited(mut self) -> Self {
            self.visited = true;
            self
        }

        /// Returns `true` if the [`Entry`] is a directory.
        pub fn is_dir(&self) -> bool {
            matches!(self.kind, EntryKind::Dir)
        }

        /// Returns `true` if the [`Entry`] should not be removed.
        pub fn is_skipped(&self) -> bool {
            self.skip_reason.is_some()
        }

        /// Returns `true` if the [`Entry`] has been (marked as) visited.
        pub fn is_visited(&self) -> bool {
            self.visited
        }

        /// Get the kind of the [`Entry`].
        pub fn kind(&self) -> EntryKind {
            self.kind.clone()
        }

        /// Get the path to the [`Entry`].
        pub fn path(&self) -> PathBuf {
            Path::new(&self.path).to_owned()
        }

        /// Returns the reason why the [`Entry`] should not be removed, if any.
        ///
        /// See also `is_skipped`.
        pub fn skip_reason(&self) -> Option<String> {
            self.skip_reason.clone()
        }
    }

    impl std::fmt::Display for Entry {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.path().display())
        }
    }

    /// Enum representing the kind of a file system [`Entry`].
    #[derive(Clone, Eq, PartialEq)]
    #[cfg_attr(test, derive(Arbitrary, Debug))]
    pub enum EntryKind {
        /// An [`Entry`] that is a directory.
        Dir,

        /// An [`Entry`] that is a file.
        File,

        /// An [`Entry`] that is a symbolic link.
        Symlink,
    }

    /// Tests for the [`Entry`] struct.
    #[cfg(test)]
    mod test_entry {
        use super::{Entry, EntryKind, Error, ErrorKind};

        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[proptest]
        fn new(path: String, kind: EntryKind) {
            prop_assert_eq!(
                Entry::new(&path, kind.clone()),
                Entry { path: path.into(), kind, skip_reason: None, visited: false }
            );
        }

        #[proptest]
        fn display(entry: Entry) {
            prop_assert_eq!(entry.to_string(), format!("{}", entry.path().display()));
        }

        #[proptest]
        fn into_err(entry: Entry, err_kind: ErrorKind) {
            let path = entry.path.clone();

            let err = entry.into_err(err_kind.clone());
            prop_assert_eq!(err, Error { kind: err_kind, path, tip: None });
        }

        #[proptest]
        fn into_skipped(entry: Entry, reason: String) {
            let path = entry.path.clone();
            let kind = entry.kind.clone();
            let visited = entry.visited;

            let err = entry.into_skipped(&reason);
            prop_assert_eq!(err, Entry { kind, path, skip_reason: Some(reason), visited });
        }

        #[proptest]
        fn into_visited(entry: Entry) {
            let path = entry.path.clone();
            let kind = entry.kind.clone();
            let skip_reason = entry.skip_reason.clone();

            let err = entry.into_visited();
            prop_assert_eq!(err, Entry { kind, path, skip_reason, visited: true });
        }

        #[proptest]
        fn is_dir(entry: Entry) {
            prop_assert_eq!(entry.is_dir(), matches!(entry.kind, EntryKind::Dir));
        }

        #[proptest]
        fn is_skipped(entry: Entry) {
            prop_assert_eq!(entry.is_skipped(), entry.skip_reason.is_some());
        }

        #[proptest]
        fn is_visited(entry: Entry) {
            prop_assert_eq!(entry.is_visited(), entry.visited);
        }

        #[proptest]
        fn kind(entry: Entry) {
            prop_assert_eq!(entry.kind(), entry.kind);
        }

        #[proptest]
        fn path(entry: Entry) {
            prop_assert_eq!(entry.path(), entry.path);
        }

        #[proptest]
        fn skip_reason(entry: Entry) {
            prop_assert_eq!(entry.skip_reason(), entry.skip_reason);
        }
    }

    /// Struct representing a file system error.
    #[derive(Debug)]
    #[cfg_attr(test, derive(Arbitrary, Clone, Eq, PartialEq))]
    pub struct Error {
        /// The kind of error that occurred.
        kind: ErrorKind,

        /// The path for which the error occurred.
        path: OsString,

        /// A tip to deal with the error, if any.
        tip: Option<String>,
    }

    impl Error {
        /// Create a new [`Error`] with a given `path` and [`ErrorKind`].
        fn new<P: AsRef<Path>>(path: P, kind: ErrorKind) -> Self {
            Self { kind, path: path.as_ref().as_os_str().to_owned(), tip: None }
        }

        /// Get the kind of the [`Error`].
        pub fn kind(&self) -> ErrorKind {
            self.kind.clone()
        }

        /// Convert the [`Error`] into a skipped [`Entry`].
        pub fn into_skipped(self, reason: &str) -> Entry {
            // Note: the `EntryKind` can't be known as the error may be for a missing path.
            Entry::new(self.path(), EntryKind::File).into_skipped(reason)
        }

        /// Get the file system path this [`Error`] is associated with.
        pub fn path(&self) -> PathBuf {
            Path::new(&self.path).to_owned()
        }

        /// Convert this [`Error`] into an [`Error`] with the provided tip associated to it.
        pub fn with_tip(mut self, tip: &str) -> Self {
            self.tip = Some(tip.to_owned());
            self
        }
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if let Some(tip) = &self.tip {
                write!(
                    f,
                    "Cannot remove {}: {} {}",
                    self.path().display().bold(),
                    self.kind,
                    format!("({tip})").italic()
                )
            } else {
                write!(f, "Cannot remove {}: {}", self.path().display().bold(), self.kind)
            }
        }
    }

    impl std::error::Error for Error {}

    /// Tests for the [`Error`] struct.
    #[cfg(test)]
    mod test_error {
        use super::{Entry, EntryKind, Error, ErrorKind};

        use owo_colors::OwoColorize;
        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[proptest]
        fn new(path: String, kind: ErrorKind) {
            let err = Error::new(&path, kind.clone());
            prop_assert_eq!(err, Error { kind, path: path.into(), tip: None });
        }

        #[proptest]
        fn display_with_tip(err: Error) {
            prop_assume!(err.tip.is_some());

            prop_assert_eq!(
                err.to_string(),
                format!(
                    "Cannot remove {}: {} {}",
                    err.path().display().bold(),
                    err.kind(),
                    format!("({})", err.tip.expect("is_some() should be asserted")).italic(),
                )
            );
        }

        #[proptest]
        fn display_without_tip(err: Error) {
            prop_assume!(err.tip.is_none());

            prop_assert_eq!(
                err.to_string(),
                format!("Cannot remove {}: {}", err.path().display().bold(), err.kind())
            );
        }

        #[proptest]
        fn kind(err: Error) {
            prop_assert_eq!(err.kind(), err.kind);
        }

        #[proptest]
        fn into_skipped(err: Error, reason: String) {
            let path = err.path();

            let entry = err.into_skipped(&reason);
            prop_assert_eq!(entry, Entry::new(path, EntryKind::File).into_skipped(&reason));
        }

        #[proptest]
        fn path(err: Error) {
            prop_assert_eq!(err.path(), err.path);
        }

        #[proptest]
        fn with_tip(err: Error, tip: String) {
            let kind = err.kind();
            let path = err.path();

            prop_assert_eq!(err.with_tip(&tip), Error { kind, path: path.into(), tip: Some(tip) });
        }
    }

    /// Enum representing kinds of file system [`Error`]s.
    #[derive(Clone, Debug, Eq, PartialEq)]
    #[cfg_attr(test, derive(Arbitrary))]
    pub enum ErrorKind {
        /// This kind corresponds to an error due to a directory not being empty.
        DirectoryNotEmpty,

        /// This kind corresponds to an error due to an [`Entry`] being a directory.
        IsADirectory,

        /// This kind corresponds to an [`Entry`] not being found on the system.
        NotFound,

        /// This kind corresponds to the user not having access to an [`Entry`].
        PermissionDenied,

        /// This kind corresponds to the CLI refusing to remove an [`Entry`] because removing it
        /// is potentially dangerous (e.g. it is the current directory).
        Refused,

        /// This kind is a catch all for any unknown error.
        Unknown,
    }

    impl std::fmt::Display for ErrorKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::DirectoryNotEmpty => write!(f, "Directory not empty"),
                Self::IsADirectory => write!(f, "Is a directory"),
                Self::NotFound => write!(f, "Not found"),
                Self::PermissionDenied => write!(f, "Permission denied"),
                Self::Refused => write!(f, "Refused to remove"),
                Self::Unknown => write!(f, "Unknown error"),
            }
        }
    }

    impl From<io::ErrorKind> for ErrorKind {
        fn from(val: io::ErrorKind) -> Self {
            match val {
                io::ErrorKind::NotFound => Self::NotFound,
                io::ErrorKind::PermissionDenied => Self::PermissionDenied,
                _ => Self::Unknown,
            }
        }
    }

    #[cfg(feature = "trash")]
    impl From<trash::Error> for ErrorKind {
        fn from(val: trash::Error) -> Self {
            match val {
                trash::Error::CouldNotAccess { .. } => Self::PermissionDenied,
                #[cfg(all(unix, not(target_os = "macos")))]
                trash::Error::FileSystem { kind, .. } => kind.into(),
                trash::Error::TargetedRoot => Self::Refused,
                _ => Self::Unknown,
            }
        }
    }

    /// Tests for the [`ErrorKind`] enum.
    #[cfg(test)]
    mod test_error_kind {
        use super::ErrorKind;

        use std::io;

        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[test]
        fn from_io_not_found() {
            assert_eq!(ErrorKind::NotFound, io::ErrorKind::NotFound.into());
        }

        #[test]
        fn from_io_permission_denied() {
            assert_eq!(ErrorKind::PermissionDenied, io::ErrorKind::PermissionDenied.into());
        }

        #[proptest]
        #[cfg(feature = "trash")]
        fn from_trash_could_not_access(target: String) {
            let err = trash::Error::CouldNotAccess { target };
            prop_assert_eq!(ErrorKind::PermissionDenied, err.into());
        }

        #[proptest]
        #[cfg(feature = "trash")]
        #[cfg(all(unix, not(target_os = "macos")))]
        fn from_trash_file_system(kind: std::io::ErrorKind, path: String) {
            let err = trash::Error::FileSystem { kind, path: std::path::Path::new(&path).into() };
            let expected: ErrorKind = kind.into();
            prop_assert_eq!(expected, err.into());
        }

        #[test]
        #[cfg(feature = "trash")]
        fn from_trash_targeted_root() {
            assert_eq!(ErrorKind::Refused, trash::Error::TargetedRoot.into());
        }
    }

    /// Helpers for writing unit tests in or using this module.
    #[cfg(test)]
    pub mod test_helpers {
        use super::{Entry, EntryKind, Error};

        use std::path::Path;

        impl Error {
            /// Get the tip associated with this [`Error`], if any.
            pub fn tip(&self) -> Option<&str> {
                match &self.tip {
                    Some(tip) => Some(tip),
                    None => None,
                }
            }
        }

        /// Create an [`Entry`] representing a file for testing purposes.
        pub fn new_file<P: AsRef<Path>>(path: P) -> Entry {
            Entry::new(&path, EntryKind::File)
        }

        /// Create an [`Entry`] representing a directory for testing purposes.
        pub fn new_dir<P: AsRef<Path>>(path: P) -> Entry {
            Entry::new(&path, EntryKind::Dir)
        }

        /// Create an [`Entry`] representing a symbolic link for testing purposes.
        pub fn new_symlink<P: AsRef<Path>>(path: P) -> Entry {
            Entry::new(&path, EntryKind::Symlink)
        }
    }
}

/// File system walking strategies.
mod walk {
    use super::{fs, transform};

    use std::fs::read_dir;
    use std::iter;
    use std::path::Path;

    /// The return type of a file system [`Walker`].
    type FileIterator = Box<dyn Iterator<Item = fs::Result>>;

    /// The type of [`transform::Transformer`] supported by this module.
    type Transformers = [transform::Transformer; 5];

    /// A "file system walker" - a function that iterates over entries on a file system.
    pub type Walker = Box<dyn Fn(&dyn AsRef<Path>) -> FileIterator>;

    /// Create a [`Walker`] that only visits the given file system entry.
    pub fn given(transformers: Transformers) -> Walker {
        Box::new(move |path| {
            Box::new(iter::once(transform_entry(
                fs::open(path).map(fs::Entry::into_visited),
                transformers,
            )))
        })
    }

    /// Tests for the [`given`] function.
    #[cfg(test)]
    mod test_given {
        use crate::test_helpers::{with_test_dir, TestResult};

        use super::{fs, transform};

        use assert_fs::prelude::*;

        #[test]
        fn file() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;

                let path = file.path();

                let out = given(path);
                assert_eq!(out, vec![fs::open(path).map(fs::Entry::into_visited)]);

                Ok(())
            })
        }

        #[test]
        fn empty_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;

                let path = dir.path();

                let out = given(path);
                assert_eq!(out, vec![fs::open(path).map(fs::Entry::into_visited)]);

                Ok(())
            })
        }

        #[test]
        fn filled_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                dir.child("file").touch()?;

                let path = dir.path();

                let out = given(path);
                assert_eq!(out, vec![fs::open(path).map(fs::Entry::into_visited)]);

                Ok(())
            })
        }

        #[test]
        fn symlink_to_file() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;
                let link = test_dir.child("link");
                link.symlink_to_file(&file)?;

                let path = link.path();

                let out = given(path);
                assert_eq!(out, vec![fs::open(path).map(fs::Entry::into_visited)]);

                Ok(())
            })
        }

        #[test]
        fn symlink_to_empty_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                let link = test_dir.child("link");
                link.symlink_to_file(&dir)?;

                let path = link.path();

                let out = given(path);
                assert_eq!(out, vec![fs::open(path).map(fs::Entry::into_visited)]);

                Ok(())
            })
        }

        #[test]
        fn symlink_to_filled_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                dir.child("file").touch()?;
                let link = test_dir.child("link");
                link.symlink_to_file(&dir)?;

                let path = link.path();

                let out = given(path);
                assert_eq!(out, vec![fs::open(path).map(fs::Entry::into_visited)]);

                Ok(())
            })
        }

        #[test]
        fn not_found() -> TestResult {
            with_test_dir(|test_dir| {
                let path = test_dir.child("missing");

                let out = given(&path);
                assert_eq!(out, vec![fs::open(path)]);

                Ok(())
            })
        }

        /// Convenience wrapper around [`super::given`] for use in tests.
        fn given<P: AsRef<std::path::Path>>(path: P) -> Vec<fs::Result> {
            let given_closure = super::given([
                transform::identity,
                transform::identity,
                transform::identity,
                transform::identity,
                transform::identity,
            ]);

            given_closure(&path).collect()
        }
    }

    /// Walk the subsection of the file system with `path` as root.
    fn recurse_path<P: AsRef<Path>>(path: P, transformers: Transformers) -> FileIterator {
        match transform_entry(fs::open(path), transformers) {
            Ok(dir) if !dir.is_skipped() && dir.is_dir() && !fs::is_empty(&dir) => {
                match read_dir(dir.path()) {
                    Ok(content) => Box::new(
                        content
                            .into_iter()
                            .map_while(Result::ok)
                            .map(|entry| entry.path())
                            .flat_map(move |path| recurse_path(path, transformers))
                            .chain(iter::once_with(move || transform_entry(Ok(dir), transformers))),
                    ),
                    Err(err) => Box::new(iter::once(Err(dir.into_err(err.kind().into())))),
                }
            },
            entry => Box::new(iter::once(entry)),
        }
    }

    /// Create a [`Walker`] that recurse directories in order to visits entries on the file system.
    pub fn recurse(transformers: Transformers) -> Walker {
        Box::new(move |path| recurse_path(path, transformers))
    }

    /// Tests for the [`recurse`] function.
    #[cfg(test)]
    mod test_recurse {
        use crate::test_helpers::{with_test_dir, TestResult};

        use super::{fs, transform};

        use assert_fs::prelude::*;

        #[test]
        fn file() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;

                let path = file.path();

                let out = recurse(path);
                assert_eq!(out, vec![fs::open(path)]);

                Ok(())
            })
        }

        #[test]
        fn empty_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;

                let path = dir.path();

                let out = recurse(path);
                assert_eq!(out, vec![fs::open(path)]);

                Ok(())
            })
        }

        #[test]
        fn filled_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                let file = dir.child("file");
                file.touch()?;

                let dir_path = dir.path();
                let file_path = file.path();

                let out = recurse(dir_path);
                assert_eq!(out, vec![fs::open(file_path), fs::open(dir_path)]);

                Ok(())
            })
        }

        #[test]
        fn nested_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                let nested_dir = dir.child("nested_dir");
                nested_dir.create_dir_all()?;
                let nested_file = nested_dir.child("file1");
                nested_file.touch()?;
                let file = dir.child("file2");
                file.touch()?;

                let dir_path = dir.path();
                let nested_dir_path = nested_dir.path();
                let nested_file_path = nested_file.path();
                let file_path = file.path();

                let out = recurse(dir_path);
                assert_eq!(out.len(), 4);
                assert!(out.contains(&fs::open(file_path)));
                assert!(out.contains(&fs::open(nested_file_path)));
                assert!(out.contains(&fs::open(nested_dir_path)));
                assert!(out.contains(&fs::open(dir_path)));

                assert!(
                    out.iter()
                        .filter_map(|x| if let Ok(x) = x { Some(x) } else { None })
                        .position(|x| x.path() == nested_file_path)
                        < out
                            .iter()
                            .filter_map(|x| if let Ok(x) = x { Some(x) } else { None })
                            .position(|x| x.path() == nested_dir_path)
                );
                assert_eq!(out.last(), Some(&fs::open(dir_path)));

                Ok(())
            })
        }

        #[test]
        fn symlink_to_file() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;
                let link = test_dir.child("link");
                link.symlink_to_file(&file)?;

                let path = link.path();

                let out = recurse(path);
                assert_eq!(out, vec![fs::open(path)]);

                Ok(())
            })
        }

        #[test]
        fn symlink_to_empty_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                let link = test_dir.child("link");
                link.symlink_to_file(&dir)?;

                let path = link.path();

                let out = recurse(path);
                assert_eq!(out, vec![fs::open(path)]);

                Ok(())
            })
        }

        #[test]
        fn symlink_to_filled_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                dir.child("file").touch()?;
                let link = test_dir.child("link");
                link.symlink_to_file(&dir)?;

                let path = link.path();

                let out = recurse(path);
                assert_eq!(out, vec![fs::open(path)]);

                Ok(())
            })
        }

        #[test]
        fn not_found() -> TestResult {
            with_test_dir(|test_dir| {
                let path = test_dir.child("missing");

                let out = recurse(&path);
                assert_eq!(out, vec![fs::open(&path)]);

                Ok(())
            })
        }

        /// Convenience wrapper around [`super::recurse`] for use in tests.
        fn recurse<P: AsRef<std::path::Path>>(path: P) -> Vec<fs::Result> {
            let recurse_closure = super::recurse([
                transform::identity,
                transform::identity,
                transform::identity,
                transform::identity,
                transform::identity,
            ]);

            recurse_closure(&path).collect()
        }
    }

    /// Apply [`Transformers`] to the given [`fs::Result`].
    fn transform_entry(entry: fs::Result, transformers: Transformers) -> fs::Result {
        transformers.iter().fold(entry, |entry, transform| transform(entry))
    }

    /// Tests for the [`transform_entry`] function.
    #[cfg(test)]
    mod test_transform_entry {
        use super::{fs, transform_entry, Transformers};

        use proptest::prelude::*;
        use proptest_attr_macro::proptest;
        use proptest_derive::Arbitrary;

        #[proptest]
        #[allow(clippy::indexing_slicing)]
        fn transforms(entry: fs::Result, index: TransformersIndex) {
            let mut transformers: Transformers = [
                transform_identity,
                transform_identity,
                transform_identity,
                transform_identity,
                transform_identity,
            ];

            transformers[index.0] = transform_fixed;

            prop_assert_eq!(transform_entry(entry.clone(), transformers), transform_fixed(entry));
        }

        /// A [`super::transform::Transformer`] that does not transform the given value.
        fn transform_identity(entry: fs::Result) -> fs::Result {
            entry
        }

        /// A [`super::transform::Transformer`] that transforms all values into the same value.
        fn transform_fixed(_: fs::Result) -> fs::Result {
            Err(fs::test_helpers::new_file("file").into_err(fs::ErrorKind::Unknown))
        }

        /// Struct wrapping a [`usize`] that implements [`Arbitrary`] to generate a valid index for
        /// a [`Transformers`] instance.
        #[derive(Arbitrary, Debug)]
        struct TransformersIndex(#[proptest(strategy = "0usize..=4")] usize);
    }
}

/// File system removal strategies.
mod rm {
    use super::fs;

    use log::trace;
    use owo_colors::OwoColorize;

    /// The `Result` type for removing an [`fs::Entry`].
    pub type Result = std::result::Result<String, fs::Error>;

    /// Dispose of the [`fs::Entry`]; move it to the trash.
    ///
    /// # Errors
    ///
    /// If the [`fs::Entry`] can't be moved to the trash.
    #[cfg(feature = "trash")]
    pub fn dispose(entry: fs::Entry) -> Result {
        trace!("dispose of {entry}");

        match trash::delete(entry.path()) {
            Ok(()) => Ok(format!("Moved {} to trash", entry.bold())),
            Err(err) => Err(entry.into_err(err.into())),
        }
    }

    /// Tests for the [`dispose`] function.
    #[cfg(test)]
    #[cfg(feature = "trash")]
    mod test_dispose {
        use crate::test_helpers::{with_test_dir, TestResult};

        use super::{dispose, fs};

        use assert_fs::prelude::*;
        use owo_colors::OwoColorize;
        use predicates::prelude::*;

        #[test]
        #[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
        fn file() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;

                let path = file.path();
                let entry = fs::test_helpers::new_file(path);

                let out = dispose(entry);
                assert_eq!(out, Ok(format!("Moved {} to trash", path.display().bold())));

                file.assert(predicate::path::missing());

                Ok(())
            })
        }

        #[test]
        #[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
        #[cfg(all(unix, not(target_os = "macos")))]
        fn file_not_found_toctou() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("missing");

                let path = file.path();
                let entry = fs::test_helpers::new_file(path);

                let out = dispose(entry);
                assert!(out.is_err());

                let err = out.expect_err("is_err() should be asserted");
                assert_eq!(err.kind(), fs::ErrorKind::NotFound);
                assert_eq!(err.path(), path);

                Ok(())
            })
        }

        #[test]
        #[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
        fn dir_empty() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;

                let path = dir.path();
                let entry = fs::test_helpers::new_dir(path);

                let out = dispose(entry);
                assert_eq!(out, Ok(format!("Moved {} to trash", path.display().bold())));

                dir.assert(predicate::path::missing());

                Ok(())
            })
        }

        #[test]
        #[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
        fn dir_filled() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                dir.child("file").touch()?;

                let path = dir.path();
                let entry = fs::test_helpers::new_dir(path);

                let out = dispose(entry);
                assert_eq!(out, Ok(format!("Moved {} to trash", path.display().bold())));

                dir.assert(predicate::path::missing());

                Ok(())
            })
        }

        #[test]
        #[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
        #[cfg(all(unix, not(target_os = "macos")))]
        fn dir_not_found_toctou() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("missing");

                let path = dir.path();
                let entry = fs::test_helpers::new_dir(path);

                let out = dispose(entry);
                assert!(out.is_err());

                let err = out.expect_err("is_err() should be asserted");
                assert_eq!(err.kind(), fs::ErrorKind::NotFound);
                assert_eq!(err.path(), path);

                Ok(())
            })
        }

        #[test]
        #[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
        fn symlink_to_file() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;
                let link = test_dir.child("link");
                link.symlink_to_file(&file)?;

                let path = link.path();
                let entry = fs::test_helpers::new_symlink(path);

                let out = dispose(entry);
                assert_eq!(out, Ok(format!("Moved {} to trash", path.display().bold())));

                file.assert(predicate::path::exists());
                link.assert(predicate::path::missing());

                Ok(())
            })
        }

        #[test]
        #[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
        fn symlink_to_empty_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                let link = test_dir.child("link");
                link.symlink_to_dir(&dir)?;

                let path = link.path();
                let entry = fs::test_helpers::new_symlink(path);

                let out = dispose(entry);
                assert_eq!(out, Ok(format!("Moved {} to trash", path.display().bold())));

                dir.assert(predicate::path::exists());
                link.assert(predicate::path::missing());

                Ok(())
            })
        }

        #[test]
        #[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
        fn symlink_to_filled_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                let nested_file = dir.child("file");
                nested_file.touch()?;
                let link = test_dir.child("link");
                link.symlink_to_dir(&dir)?;

                let path = link.path();
                let entry = fs::test_helpers::new_symlink(path);

                let out = dispose(entry);
                assert_eq!(out, Ok(format!("Moved {} to trash", path.display().bold())));

                dir.assert(predicate::path::exists());
                nested_file.assert(predicate::path::exists());
                link.assert(predicate::path::missing());

                Ok(())
            })
        }

        #[test]
        #[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
        #[cfg(all(unix, not(target_os = "macos")))]
        fn symlink_not_found_toctou() -> TestResult {
            with_test_dir(|test_dir| {
                let link = test_dir.child("missing");

                let path = link.path();
                let entry = fs::test_helpers::new_symlink(path);

                let out = dispose(entry);
                assert!(out.is_err());

                let err = out.expect_err("is_err() should be asserted");
                assert_eq!(err.kind(), fs::ErrorKind::NotFound);
                assert_eq!(err.path(), path);

                Ok(())
            })
        }

        #[test]
        #[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
        fn symlink_to_file_at_location_of_a_file_toctou() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;
                let link = test_dir.child("link");
                link.symlink_to_file(&file)?;

                let path = link.path();
                let entry = fs::test_helpers::new_file(path);

                let out = dispose(entry);
                assert_eq!(out, Ok(format!("Moved {} to trash", path.display().bold())));

                file.assert(predicate::path::exists());
                link.assert(predicate::path::missing());

                Ok(())
            })
        }

        #[test]
        #[cfg_attr(not(feature = "test-trash"), ignore = "Only run with the test-trash feature")]
        fn symlink_to_dir_at_location_of_a_dir_toctou() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                let link = test_dir.child("link");
                link.symlink_to_dir(&dir)?;

                let path = link.path();
                let entry = fs::test_helpers::new_dir(path);

                let out = dispose(entry);
                assert_eq!(out, Ok(format!("Moved {} to trash", path.display().bold())));

                dir.assert(predicate::path::exists());
                link.assert(predicate::path::missing());

                Ok(())
            })
        }
    }

    /// Remove the [`fs::Entry`] from the file system.
    ///
    /// # Errors
    ///
    /// If the [`fs::Entry`] can't be removed.
    pub fn remove(entry: fs::Entry) -> Result {
        use std::fs::{remove_dir, remove_file};

        trace!("remove {entry}");

        if entry.is_dir() && !fs::is_empty(&entry) {
            // This case is handled explicitly because, as of Rust 1.69, the `io::ErrorKind` variant
            // is still experimental (gate "io_error_more") and so would result in an unknown error.
            // This implementation leaves a possibility for a TOCTOU issue, but this will be handled
            // safely as `std::fs::remove_dir` doesn't remove non-empty directories.
            return Err(entry.into_err(fs::ErrorKind::DirectoryNotEmpty));
        }

        let path = entry.path();
        let result = match entry.kind() {
            fs::EntryKind::Dir => remove_dir(path),
            fs::EntryKind::File | fs::EntryKind::Symlink => remove_file(path),
        };

        match result {
            Ok(()) => Ok(format!("Removed {}", entry.bold())),
            Err(err) => Err(entry.into_err(err.kind().into())),
        }
    }

    /// Tests for the [`remove`] function.
    #[cfg(test)]
    mod test_remove {
        use crate::test_helpers::{with_test_dir, TestResult};

        use super::{fs, remove};

        use assert_fs::prelude::*;
        use owo_colors::OwoColorize;
        use predicates::prelude::*;

        #[test]
        fn file() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;

                let path = file.path();
                let entry = fs::test_helpers::new_file(path);

                let out = remove(entry);
                assert_eq!(out, Ok(format!("Removed {}", path.display().bold())));

                file.assert(predicate::path::missing());

                Ok(())
            })
        }

        #[test]
        fn file_not_found_toctou() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("missing");

                let path = file.path();
                let entry = fs::test_helpers::new_file(path);

                let out = remove(entry);
                assert!(out.is_err());

                let err = out.expect_err("is_err() should be asserted");
                assert_eq!(err.kind(), fs::ErrorKind::NotFound);
                assert_eq!(err.path(), path);

                Ok(())
            })
        }

        #[test]
        fn dir_empty() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;

                let path = dir.path();
                let entry = fs::test_helpers::new_dir(path);

                let out = remove(entry);
                assert_eq!(out, Ok(format!("Removed {}", path.display().bold())));

                dir.assert(predicate::path::missing());

                Ok(())
            })
        }

        #[test]
        fn dir_filled_toctou() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                dir.child("file").touch()?;

                let path = dir.path();
                let entry = fs::test_helpers::new_dir(path);

                let out = remove(entry);
                assert!(out.is_err());

                let err = out.expect_err("is_err() should be asserted");
                assert_eq!(err.kind(), fs::ErrorKind::DirectoryNotEmpty);
                assert_eq!(err.path(), path);

                dir.assert(predicate::path::exists());

                Ok(())
            })
        }

        #[test]
        fn dir_not_found_toctou() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("missing");

                let path = dir.path();
                let entry = fs::test_helpers::new_dir(path);

                let out = remove(entry);
                assert!(out.is_err());

                let err = out.expect_err("is_err() should be asserted");
                assert_eq!(err.kind(), fs::ErrorKind::NotFound);
                assert_eq!(err.path(), path);

                Ok(())
            })
        }

        #[test]
        fn symlink_to_file() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;
                let link = test_dir.child("link");
                link.symlink_to_file(&file)?;

                let path = link.path();
                let entry = fs::test_helpers::new_symlink(path);

                let out = remove(entry);
                assert_eq!(out, Ok(format!("Removed {}", path.display().bold())));

                file.assert(predicate::path::exists());
                link.assert(predicate::path::missing());

                Ok(())
            })
        }

        #[test]
        #[cfg_attr(windows, ignore = "TODO: investigate symlink test errors on Windows")]
        fn symlink_to_empty_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                let link = test_dir.child("link");
                link.symlink_to_dir(&dir)?;

                let path = link.path();
                let entry = fs::test_helpers::new_symlink(path);

                let out = remove(entry);
                assert_eq!(out, Ok(format!("Removed {}", path.display().bold())));

                dir.assert(predicate::path::exists());
                link.assert(predicate::path::missing());

                Ok(())
            })
        }

        #[test]
        #[cfg_attr(windows, ignore = "TODO: investigate symlink test errors on Windows")]
        fn symlink_to_filled_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                let nested_file = dir.child("file");
                nested_file.touch()?;
                let link = test_dir.child("link");
                link.symlink_to_dir(&dir)?;

                let path = link.path();
                let entry = fs::test_helpers::new_symlink(path);

                let out = remove(entry);
                assert_eq!(out, Ok(format!("Removed {}", path.display().bold())));

                dir.assert(predicate::path::exists());
                nested_file.assert(predicate::path::exists());
                link.assert(predicate::path::missing());

                Ok(())
            })
        }

        #[test]
        fn symlink_not_found_toctou() -> TestResult {
            with_test_dir(|test_dir| {
                let link = test_dir.child("missing");

                let path = link.path();
                let entry = fs::test_helpers::new_symlink(path);

                let out = remove(entry);
                assert!(out.is_err());

                let err = out.expect_err("is_err() should be asserted");
                assert_eq!(err.kind(), fs::ErrorKind::NotFound);
                assert_eq!(err.path(), path);

                Ok(())
            })
        }

        #[test]
        fn symlink_to_file_at_location_of_a_file_toctou() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;
                let link = test_dir.child("link");
                link.symlink_to_file(&file)?;

                let path = link.path();
                let entry = fs::test_helpers::new_file(path);

                let out = remove(entry);
                assert_eq!(out, Ok(format!("Removed {}", path.display().bold())));

                file.assert(predicate::path::exists());
                link.assert(predicate::path::missing());

                Ok(())
            })
        }

        #[test]
        #[cfg_attr(windows, ignore = "TODO: investigate symlink test errors on Windows")]
        fn symlink_to_dir_at_location_of_a_dir_toctou() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                let link = test_dir.child("link");
                link.symlink_to_dir(&dir)?;

                let path = link.path();
                let entry = fs::test_helpers::new_dir(path);

                let out = remove(entry.clone());
                assert_eq!(out, Err(entry.into_err(fs::ErrorKind::Unknown)));

                dir.assert(predicate::path::exists());
                link.assert(predicate::path::exists());

                Ok(())
            })
        }
    }

    /// Pretend to dispose of the [`fs::Entry`].
    ///
    /// See also [`dispose`].
    ///
    /// # Errors
    ///
    /// This function will never return an error.
    #[cfg(feature = "trash")]
    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    pub fn show_dispose(entry: fs::Entry) -> Result {
        Ok(format!("Would move {} to trash", entry.bold()))
    }

    /// Tests for the [`show_dispose`] function.
    #[cfg(test)]
    #[cfg(feature = "trash")]
    mod test_show_dispose {
        use super::{fs, show_dispose};

        use owo_colors::OwoColorize;
        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[proptest]
        fn anything(entry: fs::Entry) {
            let path = entry.path();
            let out = show_dispose(entry);
            prop_assert_eq!(out, Ok(format!("Would move {} to trash", path.display().bold())));
        }
    }

    /// Pretend to remove the [`fs::Entry`].
    ///
    /// See also [`remove`].
    ///
    /// # Errors
    ///
    /// This function will never return an error.
    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    pub fn show_remove(entry: fs::Entry) -> Result {
        Ok(format!("Would remove {}", entry.bold()))
    }

    /// Tests for the [`show_remove`] function.
    #[cfg(test)]
    mod test_show_remove {
        use super::{fs, show_remove};

        use owo_colors::OwoColorize;
        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[proptest]
        fn anything(entry: fs::Entry) {
            let path = entry.path();
            let out = show_remove(entry);
            prop_assert_eq!(out, Ok(format!("Would remove {}", path.display().bold())));
        }
    }
}

/// Transformers for [`fs::Result`]s.
mod transform {
    use super::fs;

    use std::io;

    use owo_colors::OwoColorize;

    /// A function that may change a [`fs::Result`] into a different-but-related [`fs::Result`].
    pub type Transformer = fn(fs::Result) -> fs::Result;

    /// Does nothing, returns any value untouched.
    pub fn identity(entry: fs::Result) -> fs::Result {
        entry
    }

    /// Tests for the [`identity`] function.
    #[cfg(test)]
    mod test_identity {
        use super::{fs, identity};

        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[proptest]
        fn entry(entry: fs::Entry) {
            let out = identity(Ok(entry.clone()));
            prop_assert_eq!(out, Ok(entry));
        }

        #[proptest]
        fn error(err: fs::Error) {
            let out = identity(Err(err.clone()));
            prop_assert_eq!(out, Err(err));
        }
    }

    /// The tip for avoiding [`fs::ErrorKind::IsADirectory`] errors.
    const TIP_IS_DIR: &str = "use '--dir' to remove";

    /// Transform all directories into a [`fs::ErrorKind::IsADirectory`] error. Return all other
    /// values untouched.
    pub fn disallow_all_dirs(entry: fs::Result) -> fs::Result {
        match entry {
            Ok(entry) if entry.is_dir() => {
                Err(entry.into_err(fs::ErrorKind::IsADirectory).with_tip(TIP_IS_DIR))
            },
            _ => entry,
        }
    }

    /// Tests for the [`disallow_all_dirs`] function.
    #[cfg(test)]
    mod test_disallow_all_dirs {
        use super::{disallow_all_dirs, fs, TIP_IS_DIR};

        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[proptest]
        fn entry_non_directory(entry: fs::Entry) {
            prop_assume!(!entry.is_dir());

            let out = disallow_all_dirs(Ok(entry.clone()));
            prop_assert_eq!(out, Ok(entry));
        }

        #[proptest]
        fn entry_directory(entry: fs::Entry) {
            prop_assume!(entry.is_dir());

            let path = entry.path();

            let out = disallow_all_dirs(Ok(entry));
            prop_assert!(out.is_err());

            let err = out.expect_err("is_err() should be asserted");
            prop_assert_eq!(err.kind(), fs::ErrorKind::IsADirectory);
            prop_assert_eq!(err.path(), path);
            prop_assert_eq!(err.tip(), Some(TIP_IS_DIR));
        }

        #[proptest]
        fn error(err: fs::Error) {
            let out = disallow_all_dirs(Err(err.clone()));
            prop_assert_eq!(out, Err(err));
        }
    }

    /// Transform current directory and parent directory into a [`fs::ErrorKind::Refused`] error.
    /// Return all other values untouched.
    pub fn disallow_current_and_parent_dir(entry: fs::Result) -> fs::Result {
        match entry {
            Ok(entry) if entry.path().ends_with(".") || entry.path().ends_with("..") => {
                Err(entry.into_err(fs::ErrorKind::Refused))
            },
            _ => entry,
        }
    }

    /// Tests for the [`disallow_current_and_parent_dir`] function.
    #[cfg(test)]
    mod test_disallow_current_and_parent_dir {
        use super::{disallow_current_and_parent_dir, fs};

        use std::path::{Path, MAIN_SEPARATOR};

        use proptest::prelude::*;
        use proptest_attr_macro::proptest;
        use proptest_derive::Arbitrary;

        #[proptest]
        fn entry_not_current_nor_parent_dir(entry: fs::Entry) {
            prop_assume!(!entry.path().ends_with("."));
            prop_assume!(!entry.path().ends_with(".."));

            let out = disallow_current_and_parent_dir(Ok(entry.clone()));
            prop_assert_eq!(out, Ok(entry));
        }

        #[proptest]
        fn entry_current_directory(path: CurrentDirPath) {
            let path = path.0.replace('/', &MAIN_SEPARATOR.to_string());
            let entry = fs::test_helpers::new_dir(&path);

            let out = disallow_current_and_parent_dir(Ok(entry));
            prop_assert!(out.is_err());

            let err = out.expect_err("is_err() should be asserted");
            prop_assert_eq!(err.kind(), fs::ErrorKind::Refused);
            prop_assert_eq!(err.path(), Path::new(&path));
        }

        #[proptest]
        fn entry_parent_directory(path: ParentDirPath) {
            let path = path.0.replace('/', &MAIN_SEPARATOR.to_string());
            let entry = fs::test_helpers::new_dir(&path);

            let out = disallow_current_and_parent_dir(Ok(entry));
            prop_assert!(out.is_err());

            let err = out.expect_err("is_err() should be asserted");
            prop_assert_eq!(err.kind(), fs::ErrorKind::Refused);
            prop_assert_eq!(err.path(), Path::new(&path));
        }

        #[proptest]
        fn error(err: fs::Error) {
            let out = disallow_current_and_parent_dir(Err(err.clone()));
            prop_assert_eq!(out, Err(err));
        }

        /// Struct wrapping a [`String`] that implements [`Arbitrary`] to generate a current
        /// directory path.
        ///
        /// Note: the generated path will always use Unix path separators.
        #[derive(Arbitrary, Debug)]
        struct CurrentDirPath(#[proptest(regex = "(\\.\\.?/)*\\.")] String);

        /// Struct wrapping a [`String`] that implements [`Arbitrary`] to generate a parent
        /// directory path.
        ///
        /// Note: the generated path will always use Unix path separators.
        #[derive(Arbitrary, Debug)]
        struct ParentDirPath(#[proptest(regex = "(\\.\\.?/)*\\.\\.")] String);
    }

    /// The tip for avoiding [`fs::ErrorKind::DirectoryNotEmpty`] errors.
    const TIP_DIR_NOT_EMPTY: &str = "use '--recursive' to remove";

    /// Transform filled directories into a [`fs::ErrorKind::DirectoryNotEmpty`] error. Return all
    /// other values untouched.
    pub fn disallow_filled_dirs(entry: fs::Result) -> fs::Result {
        match entry {
            Ok(entry) if entry.is_dir() && !fs::is_empty(&entry) => {
                Err(entry.into_err(fs::ErrorKind::DirectoryNotEmpty).with_tip(TIP_DIR_NOT_EMPTY))
            },
            _ => entry,
        }
    }

    /// Tests for the [`disallow_filled_dirs`] function.
    #[cfg(test)]
    mod test_disallow_filled_dirs {
        use crate::test_helpers::{with_test_dir, TestResult};

        use super::{disallow_filled_dirs, fs, TIP_DIR_NOT_EMPTY};

        use assert_fs::prelude::*;
        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[proptest]
        fn entry_non_dir(entry: fs::Entry) {
            prop_assume!(!entry.is_dir());

            let out = disallow_filled_dirs(Ok(entry.clone()));
            prop_assert_eq!(out, Ok(entry));
        }

        #[test]
        fn entry_empty_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;

                let path = dir.path();
                let entry = fs::test_helpers::new_dir(path);

                let out = disallow_filled_dirs(Ok(entry.clone()));
                assert_eq!(out, Ok(entry));

                Ok(())
            })
        }

        #[test]
        fn entry_filled_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                dir.child("file").touch()?;

                let path = dir.path();
                let entry = fs::test_helpers::new_dir(path);

                let out = disallow_filled_dirs(Ok(entry));
                assert!(out.is_err());

                let err = out.expect_err("is_err() should be asserted");
                assert_eq!(err.kind(), fs::ErrorKind::DirectoryNotEmpty);
                assert_eq!(err.path(), path);
                assert_eq!(err.tip(), Some(TIP_DIR_NOT_EMPTY));

                Ok(())
            })
        }

        #[proptest]
        fn error(err: fs::Error) {
            let out = disallow_filled_dirs(Err(err.clone()));
            prop_assert_eq!(out, Err(err));
        }
    }

    /// Transform root directories into a [`fs::ErrorKind::Refused`] error. Return all other values
    /// untouched.
    pub fn disallow_root(entry: fs::Result) -> fs::Result {
        match entry {
            Ok(entry) if entry.path().parent().is_none() => {
                Err(entry.into_err(fs::ErrorKind::Refused))
            },
            _ => entry,
        }
    }

    /// Tests for the [`disallow_root`] function.
    #[cfg(test)]
    mod test_disallow_root {
        use super::{disallow_root, fs};

        use std::path::Path;

        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[proptest]
        #[cfg_attr(windows, ignore = "TODO: investigate symlink test errors on Windows")]
        fn entry_non_root(entry: fs::Entry) {
            prop_assume!(entry.path() != Path::new("/"));
            prop_assume!(!entry.path().as_os_str().is_empty());

            let out = disallow_root(Ok(entry.clone()));
            prop_assert_eq!(out, Ok(entry));
        }

        #[test]
        fn entry_root() {
            let path = Path::new("/");
            let entry = fs::test_helpers::new_dir(path);

            let out = disallow_root(Ok(entry));
            assert!(out.is_err());

            let err = out.expect_err("is_err() should be asserted");
            assert_eq!(err.kind(), fs::ErrorKind::Refused);
            assert_eq!(err.path(), path);
        }

        #[proptest]
        fn error(err: fs::Error) {
            let out = disallow_root(Err(err.clone()));
            prop_assert_eq!(out, Err(err));
        }
    }

    /// The explanation for why a missing [`fs::Entry`] is skipped.
    const SKIP_REASON_NOT_FOUND: &str = "Not found";

    /// Transform [`fs::ErrorKind::NotFound`] errors into skipped [`fs::Entry`]s. Return all other
    /// values untouched.
    pub fn skip_missing_entry(entry: fs::Result) -> fs::Result {
        match entry {
            Err(err) if err.kind() == fs::ErrorKind::NotFound => {
                Ok(err.into_skipped(SKIP_REASON_NOT_FOUND))
            },
            _ => entry,
        }
    }

    /// Tests for the [`skip_missing_entry`] function.
    #[cfg(test)]
    mod test_skip_missing_entry {
        use super::{fs, skip_missing_entry};

        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[proptest]
        fn entry(entry: fs::Entry) {
            let out = skip_missing_entry(Ok(entry.clone()));
            prop_assert_eq!(out, Ok(entry));
        }

        #[proptest]
        fn error_not_found(entry: fs::Entry) {
            let err = entry.into_err(fs::ErrorKind::NotFound);
            let path = err.path();

            let out = skip_missing_entry(Err(err));
            prop_assert!(out.is_ok());

            let entry = out.expect("is_ok() should be asserted");
            prop_assert_eq!(entry.skip_reason(), Some(super::SKIP_REASON_NOT_FOUND.to_owned()));
            prop_assert_eq!(entry.path(), path);
        }

        #[proptest]
        fn error_other_than_not_found(err: fs::Error) {
            prop_assume!(err.kind() != fs::ErrorKind::NotFound);

            let out = skip_missing_entry(Err(err.clone()));
            prop_assert_eq!(out, Err(err));
        }
    }

    /// The tip for avoiding [`fs::ErrorKind::NotFound`] errors.
    const TIP_NOT_FOUND: &str = "use '--blind' to ignore";

    /// Transform [`fs::ErrorKind::NotFound`] errors into equivalent errors with an associated tip
    /// for how to avoid it. Return all other values untouched.
    pub fn tip_not_found(entry: fs::Result) -> fs::Result {
        match entry {
            Err(err) if err.kind() == fs::ErrorKind::NotFound => Err(err.with_tip(TIP_NOT_FOUND)),
            _ => entry,
        }
    }

    /// Tests for the [`tip_not_found`] function.
    #[cfg(test)]
    mod test_tip_not_found {
        use super::{fs, tip_not_found, TIP_NOT_FOUND};

        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[proptest]
        fn entry(entry: fs::Entry) {
            let out = tip_not_found(Ok(entry.clone()));
            prop_assert_eq!(out, Ok(entry));
        }

        #[proptest]
        fn error_not_found(entry: fs::Entry) {
            let err = entry.into_err(fs::ErrorKind::NotFound);
            let path = err.path();

            let out = tip_not_found(Err(err));
            prop_assert!(out.is_err());

            let err = out.expect_err("is_err() should be asserted");
            prop_assert_eq!(err.kind(), fs::ErrorKind::NotFound);
            prop_assert_eq!(err.path(), path);
            prop_assert_eq!(err.tip(), Some(TIP_NOT_FOUND));
        }

        #[proptest]
        fn error_other_than_not_found(err: fs::Error) {
            prop_assume!(err.kind() != fs::ErrorKind::NotFound);

            let out = tip_not_found(Err(err.clone()));
            prop_assert_eq!(out, Err(err));
        }
    }

    /// The explanation for when an [`fs::Entry`] is skipped as a result of the user answering "no".
    const SKIP_REASON_ANSWER_NO: &str = "Kept by user";

    /// The explanation for when an [`fs::Entry`] is skipped as a result of unrecognized user input.
    const SKIP_REASON_ANSWER_UNKNOWN: &str = "Unrecognized input";

    /// The explanation for when an [`fs::Entry`] is skipped as a result of an I/O error.
    const SKIP_REASON_IO_ERROR: &str = "I/O error";

    /// Transform (not skipped) [`fs::Entry`]s based on user input. Return all other values
    /// untouched.
    pub fn interactive(entry: fs::Result) -> fs::Result {
        match entry {
            Ok(entry) if !entry.is_skipped() => Ok(interact_transform(
                prompt(&new_prompt_for(&entry), &mut io::stdin().lock(), &mut anstream::stderr()),
                entry,
            )),
            _ => entry,
        }
    }

    /// Create a user prompt for what to do with the given [`fs::Entry`].
    fn new_prompt_for(entry: &fs::Entry) -> String {
        let question = match entry.kind() {
            fs::EntryKind::Dir => {
                if fs::is_empty(entry) {
                    "Remove empty directory"
                } else if entry.is_visited() {
                    "Remove directory"
                } else {
                    "Descend into directory"
                }
            },
            fs::EntryKind::File => "Remove regular file",
            fs::EntryKind::Symlink => "Remove symbolic link",
        };

        format!("{question} {}? [Y/n] ", entry.bold())
    }

    /// Print the given string to the user, wait for user input, and return the user input.
    ///
    /// # Errors
    ///
    /// If any error is returned by either the reader or the writer.
    fn prompt<R, W>(prompt: &str, reader: &mut R, writer: &mut W) -> io::Result<String>
    where
        R: io::BufRead,
        W: io::Write,
    {
        const ANSWER_BUFFER_SIZE: usize = "yes".len();
        const CLEARLINE: &[u8] = "\u{1b}[1A\u{1b}[2K".as_bytes();

        writer.write_all(prompt.as_bytes())?;
        writer.flush()?;

        let mut answer = String::with_capacity(ANSWER_BUFFER_SIZE);
        reader.read_line(&mut answer)?;

        writer.write_all(CLEARLINE)?;
        writer.flush()?;

        Ok(answer.trim().to_owned())
    }

    /// Transform the given [`fs::Entry`] based on the given user response.
    fn interact_transform(response: io::Result<String>, entry: fs::Entry) -> fs::Entry {
        if let Ok(answer) = response {
            match answer.to_lowercase().as_str() {
                "y" | "yes" => entry.into_visited(),
                "n" | "no" => entry.into_skipped(SKIP_REASON_ANSWER_NO),
                _ => entry.into_skipped(SKIP_REASON_ANSWER_UNKNOWN),
            }
        } else {
            entry.into_skipped(SKIP_REASON_IO_ERROR)
        }
    }

    /// Tests for the [`interactive`] and related functions.
    #[cfg(test)]
    mod test_interactive {
        use crate::test_helpers::{with_test_dir, TestResult};

        use super::{fs, interact_transform, interactive, new_prompt_for, prompt};

        use std::io;

        use assert_fs::prelude::*;
        use owo_colors::OwoColorize;
        use proptest::prelude::*;
        use proptest_attr_macro::proptest;
        use proptest_derive::Arbitrary;

        #[proptest]
        fn interact_with_skipped(entry: fs::Entry, reason: String) {
            prop_assume!(!entry.is_visited());

            let entry = entry.into_skipped(&reason);

            let out = interactive(Ok(entry.clone()));
            prop_assert_eq!(out, Ok(entry));
        }

        #[proptest]
        fn interact_with_error(err: fs::Error) {
            let out = interactive(Err(err.clone()));
            prop_assert_eq!(out, Err(err));
        }

        #[test]
        fn new_prompt_for_file_empty() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;

                let path = file.path();
                let entry = fs::test_helpers::new_file(path);

                let out = new_prompt_for(&entry);
                assert_eq!(out, format!("Remove regular file {}? [Y/n] ", path.display().bold()));

                Ok(())
            })
        }

        #[test]
        fn new_prompt_for_file_filled() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.write_str("Hello world!")?;

                let path = file.path();
                let entry = fs::test_helpers::new_file(path);

                let out = new_prompt_for(&entry);
                assert_eq!(out, format!("Remove regular file {}? [Y/n] ", path.display().bold()));

                Ok(())
            })
        }

        #[test]
        fn new_prompt_for_dir_empty() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;

                let path = dir.path();
                let entry = fs::test_helpers::new_dir(path);

                let out = new_prompt_for(&entry);
                assert_eq!(
                    out,
                    format!("Remove empty directory {}? [Y/n] ", path.display().bold())
                );

                Ok(())
            })
        }

        #[test]
        fn new_prompt_for_visited_dir_empty() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;

                let path = dir.path();
                let entry = fs::test_helpers::new_dir(path).into_visited();

                let out = new_prompt_for(&entry);
                assert_eq!(
                    out,
                    format!("Remove empty directory {}? [Y/n] ", path.display().bold())
                );

                Ok(())
            })
        }

        #[test]
        fn new_prompt_for_dir_filled() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                dir.child("file").touch()?;

                let path = dir.path();
                let entry = fs::test_helpers::new_dir(path);

                let out = new_prompt_for(&entry);
                assert_eq!(
                    out,
                    format!("Descend into directory {}? [Y/n] ", path.display().bold())
                );

                Ok(())
            })
        }

        #[test]
        fn new_prompt_for_visited_dir_filled() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                dir.child("file").touch()?;

                let path = dir.path();
                let entry = fs::test_helpers::new_dir(path).into_visited();

                let out = new_prompt_for(&entry);
                assert_eq!(out, format!("Remove directory {}? [Y/n] ", path.display().bold()));

                Ok(())
            })
        }

        #[test]
        fn new_prompt_for_symlink_to_file() -> TestResult {
            with_test_dir(|test_dir| {
                let file = test_dir.child("file");
                file.touch()?;
                let link = test_dir.child("link");
                link.symlink_to_file(file)?;

                let path = link.path();
                let entry = fs::test_helpers::new_symlink(path);

                let out = new_prompt_for(&entry);
                assert_eq!(out, format!("Remove symbolic link {}? [Y/n] ", path.display().bold()));

                Ok(())
            })
        }

        #[test]
        fn new_prompt_for_symlink_to_empty_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                let link = test_dir.child("link");
                link.symlink_to_dir(dir)?;

                let path = link.path();
                let entry = fs::test_helpers::new_symlink(path);

                let out = new_prompt_for(&entry);
                assert_eq!(out, format!("Remove symbolic link {}? [Y/n] ", path.display().bold()));

                Ok(())
            })
        }

        #[test]
        fn new_prompt_for_symlink_to_filled_dir() -> TestResult {
            with_test_dir(|test_dir| {
                let dir = test_dir.child("dir");
                dir.create_dir_all()?;
                dir.child("file").touch()?;
                let link = test_dir.child("link");
                link.symlink_to_dir(dir)?;

                let path = link.path();
                let entry = fs::test_helpers::new_symlink(path);

                let out = new_prompt_for(&entry);
                assert_eq!(out, format!("Remove symbolic link {}? [Y/n] ", path.display().bold()));

                Ok(())
            })
        }

        #[proptest]
        fn prompt_input(question: String, answer: String) {
            let mut reader = answer.as_bytes();
            let mut writer = io::sink();

            let out = prompt(&question, &mut reader, &mut writer);
            prop_assert!(out.is_ok());

            let user_input = out.expect("is_ok() should be asserted");
            prop_assert_eq!(user_input, answer.trim());
        }

        #[proptest]
        fn prompt_input_err(question: String) {
            let mut reader = FaultyReader;
            let mut writer = io::sink();

            let out = prompt(&question, &mut reader, &mut writer);
            prop_assert!(out.is_err());
        }

        #[proptest]
        fn prompt_output(question: String) {
            let mut reader = io::empty();
            let mut writer = io::BufWriter::new(Vec::new());

            prompt(&question, &mut reader, &mut writer)?;
            prop_assert_eq!(
                String::from_utf8(writer.into_inner()?)?,
                format!("{question}\u{1b}[1A\u{1b}[2K")
            );
        }

        #[proptest]
        fn prompt_output_err(question: String) {
            let mut reader = io::empty();
            let mut writer = FaultyWriter;

            let out = prompt(&question, &mut reader, &mut writer);
            prop_assert!(out.is_err());
        }

        #[proptest]
        fn transform_answer_yes(entry: fs::Entry, answer: AnswerYes) {
            let out = interact_transform(Ok(answer.0), entry.clone());
            prop_assert_eq!(out, entry.into_visited());
        }

        #[proptest]
        fn transform_answer_no(entry: fs::Entry, answer: AnswerNo) {
            let out = interact_transform(Ok(answer.0), entry.clone());
            prop_assert_eq!(out, entry.into_skipped(super::SKIP_REASON_ANSWER_NO));
        }

        #[proptest]
        fn transform_answer_nonsense(entry: fs::Entry, answer: String) {
            prop_assume!(!matches!(answer.to_lowercase().as_ref(), "y" | "yes" | "n" | "no"));

            let out = interact_transform(Ok(answer), entry.clone());
            prop_assert_eq!(out, entry.into_skipped(super::SKIP_REASON_ANSWER_UNKNOWN));
        }

        #[proptest]
        fn transform_io_error(entry: fs::Entry, err: std::io::Error) {
            let out = interact_transform(Err(err), entry.clone());
            prop_assert_eq!(out, entry.into_skipped(super::SKIP_REASON_IO_ERROR));
        }

        /// Struct wrapping a [`String`] that implements [`Arbitrary`] to generate a "no" answer
        /// accepted by the --interactive mode of the CLI.
        #[derive(Arbitrary, Debug)]
        struct AnswerNo(#[proptest(regex = "(?i-u)(n|no)")] String);

        /// Struct wrapping a [`String`] that implements [`Arbitrary`] to generate a "yes" answer
        /// accepted by the --interactive mode of the CLI.
        #[derive(Arbitrary, Debug)]
        struct AnswerYes(#[proptest(regex = "(?i-u)(y|yes)")] String);

        /// Struct providing an erroring implementation of [`io::Read`] and [`io::BufRead`] for
        /// testing purposes.
        struct FaultyReader;

        impl io::Read for FaultyReader {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::from(std::io::ErrorKind::InvalidData))
            }
        }

        impl io::BufRead for FaultyReader {
            fn consume(&mut self, _: usize) {
                // don't need to do anything in a faulty reader
            }

            fn fill_buf(&mut self) -> io::Result<&[u8]> {
                Err(io::Error::from(std::io::ErrorKind::InvalidData))
            }
        }

        /// Struct providing a erroring implementation of [`io::Write`] for testing purposes.
        struct FaultyWriter;

        impl io::Write for FaultyWriter {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::from(std::io::ErrorKind::InvalidData))
            }

            fn flush(&mut self) -> io::Result<()> {
                Err(io::Error::from(std::io::ErrorKind::InvalidData))
            }
        }
    }
}

/// Language tasks utilities.
mod lang {
    /// Pluralize a noun based on the number of associated items. The count is always included in
    /// the return value.
    pub fn pluralize(noun: &str, count: usize) -> String {
        if count == 1 {
            format!("{count} {noun}")
        } else {
            format!("{count} {noun}s")
        }
    }

    /// Tests for the [`pluralize`] function.
    #[cfg(test)]
    mod test_pluralize {
        use super::pluralize;

        use proptest::prelude::*;
        use proptest_attr_macro::proptest;

        #[proptest]
        fn zero(noun: String) {
            assert_eq!(pluralize(&noun, 0), format!("0 {noun}s"));
        }

        #[proptest]
        fn one(noun: String) {
            assert_eq!(pluralize(&noun, 1), format!("1 {noun}"));
        }

        #[proptest]
        fn many(noun: String, count: usize) {
            prop_assume!(count > 1);
            prop_assert_eq!(pluralize(&noun, count), format!("{count} {noun}s"));
        }
    }
}

/// Logging utilities.
///
/// Logging functionality is provided by the [`log`] crate. This project only
/// uses:
/// - [`log::error!`], for outputting errors.
/// - [`log::info!`], for normal messaging (shown unless `--quiet`).
/// - [`log::trace!`], to explain what is being done (shown if `--verbose`).
///
/// # Example
///
/// ```no_run
/// logging::configure(logging::Verbosity::Normal);
/// log::error!("logged");
/// log::info!("logged");
/// log::trace!("not logged");
/// ```
mod logging {
    /// Enum representing the available levels of output verbosity.
    pub enum Verbosity {
        /// The normal verbosity of the CLI: output info and error messages.
        Normal,

        /// The `--quiet` mode of the CLI: output error messages only.
        Quiet,

        /// The `--verbose` mode of the CLI: output trace, info, and error messages.
        Verbose,
    }

    /// Set the [`Verbosity`] of the logging output.
    pub fn configure(verbosity: &Verbosity) {
        match *verbosity {
            Verbosity::Normal => log::set_max_level(log::LevelFilter::Info),
            Verbosity::Quiet => log::set_max_level(log::LevelFilter::Error),
            Verbosity::Verbose => log::set_max_level(log::LevelFilter::Trace),
        }

        _ = log::set_logger(&Logger);
    }

    /// Struct to implement the [`log::Log`] trait.
    struct Logger;

    impl log::Log for Logger {
        #[cfg(not(tarpaulin_include))]
        fn enabled(&self, _: &log::Metadata<'_>) -> bool {
            true // don't need to filter after using `set_max_level`
        }

        #[cfg(not(tarpaulin_include))]
        fn flush(&self) {
            // don't need to flush with `(e)println!`
        }

        fn log(&self, record: &log::Record<'_>) {
            use anstream::{eprintln, println};
            use owo_colors::OwoColorize;

            match record.level() {
                log::Level::Error => eprintln!("{}", record.args()),
                log::Level::Info => println!("{}", record.args()),
                log::Level::Trace => println!("{}", format!("[{}]", record.args()).italic()),
                _ => unreachable!(),
            }
        }
    }
}

/// Helpers for writing unit tests.
#[cfg(test)]
mod test_helpers {
    use assert_fs::TempDir;

    /// The environment variable name to enable debugging mode for tests.
    const TEST_DEBUG_MODE: &str = "RUST_RM_DEBUG_TEST";

    /// The `Result` type used by [`with_test_dir`].
    pub type TestResult = Result<(), Box<dyn std::error::Error>>;

    /// Run a test with access to a (temporary) testing directory.
    ///
    /// # Errors
    ///
    /// Any error returned by the test callback is returned by this function.
    ///
    /// An error may also occur if the test could not be set up.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use test_helpers::{with_test_dir, TestResult};
    ///
    /// use assert_fs::prelude::*;
    ///
    /// #[test]
    /// fn example_test() -> TestResult {
    ///     with_test_dir(|test_dir| {
    ///         // Test something using `test_dir` ...
    ///
    ///         Ok(())
    ///     })
    /// }
    /// ```
    pub fn with_test_dir<C>(callback: C) -> TestResult
    where
        C: FnOnce(&TempDir) -> TestResult,
    {
        let debug = std::env::var_os(TEST_DEBUG_MODE).is_some();
        let temp_dir = TempDir::new()?.into_persistent_if(debug);

        callback(&temp_dir)
    }
}
