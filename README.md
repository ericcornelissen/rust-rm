# `rust-rm`

A CLI like the [`rm(1)`] Unix command but more modern and designed for humans. Aims to provide an
`rm` command that feels familiar yet is safer and more user friendly. To this end it:

- Defaults to a dry run, allowing for review before removing;
- Provides suggestions for next steps, showing how you might continue;
- Supports moving to thrash, thanks to the [`trash` crate];
- Offers an excellent CLI experience, thanks to the [`clap` crate];

[`rm(1)`]: https://man7.org/linux/man-pages/man1/rm.1.html
[`clap` crate]: https://crates.io/crates/clap
[`trash` crate]: https://crates.io/crates/trash

## Usage

### By Humans

Start with a dry run:

```sh
$ rm file1 file2
Would remove file1
Would remove file2

2 would be removed (use '--force' to remove), 0 errors occurred
```

And remove if it looks good:

```sh
$ rm file1 file2 --force
Removed file1
Removed file2

2 removed, 0 errors occurred
```

Or go interactive:

```sh
$ rm file1 file2 --interactive
Removed file1
Remove regular file file2? [Y/n] _
```

### In Scripts

Use `rm` with `--force` - as well as any other flags - to remove things, for example:

```sh
rm --force --quiet file1 file2
```

or, if you want those unfamiliar with `rm` to not be able to read your script:

```sh
rm -fq file1 file2
```

## Build from Source

To build from source you need [Rust] and [Cargo], v1.70 or higher, installed on your system. Then
run the command:

```shell
just build
```

Or, if you don't have [Just] installed, use `cargo build` directly, for example:

```shell
cargo build --release
```

[cargo]: https://doc.rust-lang.org/stable/cargo/
[just]: https://just.systems/
[rust]: https://www.rust-lang.org/

### Build Configuration

The build can be modified to exclude features and obtain a smaller binary. Use:

```shell
just features=[FEATURES] build
```

where `[FEATURES]` is one or more of:

- `classic`: to include support for the [classic mode](#classic-mode).
- `trash`: to include support for the `--trash` option.

For example:

```shell
just features=classic,trash build
```

Or, to omit all optional features:

```shell
just features= build
```

## Classic Mode

The environment variable `RUST_RM_CLASSIC` can be used to enable _classic mode_. This mode aims to
offer some opt-in backwards compatibility with Unix `rm(1)`. It's meant to be useful for scripts. As
such, it aims to be compatible with non-failing and semantically valid use cases.

> **Note**: Classic mode is only available if the `classic` feature was enabled at compile time.

Classic mode will cause `rm` to:

- Remove (unlink) files and directories without `--force` or `--interactive`.
- Behave `--blind` when `--force` is used (and forget the `--blind` flag).
- Be `--quiet` by default (and forget the `--quiet` flag).
- Forget the `--trash` flag.

It won't cause `rm` to:

- Have the same `stdout`/`stderr`/`stdin` as `rm(1)`.
- Support the `-R` flag.
- Support the `-I` or `--interactive=WHEN` flags.

## Philosophy

The development of this software is guided by the following principles:

- Defaults should be safe.
- Program output is for humans.
- Help the user to achieve what they want.

### Sources of Inspiration

- [Command Line Interface Guidelines]: a guide to help write better command-line programs.
- [`git clean`]: a command for removing files, but designed much better.

[command line interface guidelines]: https://clig.dev/
[`git clean`]: https://git-scm.com/docs/git-clean

## License

All source code is licensed under the Apache 2.0 license, see [LICENSE] for the full license text.
The contents of documentation is licensed under [CC BY 4.0].

[cc by 4.0]: https://creativecommons.org/licenses/by/4.0/
[license]: ./LICENSE
