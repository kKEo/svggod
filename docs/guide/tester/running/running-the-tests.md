# Running the tests

This page takes about five minutes: it builds SVGGod, runs every test once, and shows how to run a single test.

## Before you start

You need a Rust toolchain with `cargo` and `clippy` (the CI uses the stable channel). No network access, browser or other runtime is needed to run the tests; the fonts the tests draw with are compiled into the binary.

> **Tip.** On the maintainer's macOS setup, run `export PATH=/opt/homebrew/opt/rustup/bin:$PATH` first so that `cargo` is found.

## Run everything once

```bash
cargo test                                # unit, library, command-line and MCP tests (debug build)
cargo test --release r15                  # the timing budgets, which are strict only in release builds
cargo clippy --locked --all-targets -- -D warnings   # the lint gate CI also runs
```

On version 1.0.0, `cargo test` prints three result blocks, all passing:

| Block printed by cargo | Source | Tests |
|---|---|---|
| `Running unittests src/lib.rs` | `#[cfg(test)]` modules inside `src/` | 21 |
| `Running tests/cli.rs` | `tests/cli.rs`, drives the real binary | 55 |
| `Running tests/mcp.rs` | `tests/mcp.rs`, drives `svggod mcp` over stdio | 7 |

Since the crate became a library as well, cargo prints two more blocks, `Running tests/lib.rs` (the public API, used as another crate uses it) and `Doc-tests svggod` (the examples in the API docs), and the unit tests run under `src/lib.rs`; `src/main.rs` has none.

That is 83 tests. The `tests/cli.rs` block takes about nine seconds on a laptop in a debug build; the other two take well under a second. The release-mode command runs two tests (`r15_cold_start_is_fast` and `r15_a_200_kb_file_validates_quickly`) and finishes in under a second.

> **Note.** `cargo test --release r15` builds the project a second time in release mode, so the first run is slow even though the tests are fast.

## Run one test, or one group

`cargo test` accepts a substring of the test name. Add `-- --nocapture` to see what a test prints.

```bash
cargo test --test cli r4_exit_codes            # one test in tests/cli.rs
cargo test --test cli preview                  # every test with "preview" in its name
cargo test --test mcp                          # the whole MCP file
cargo test --lib ops::                         # unit tests in src/ops.rs
cargo test --test lib                          # the library API, tests/lib.rs
cargo test --doc                               # the examples in the API docs
cargo test --test cli r8 -- --nocapture        # show output while it runs
```

Test names start with the requirement they check (`r1_`, `r4_`, `r8_`…) or describe the behaviour in words. To list names without running anything:

```bash
cargo test -- --list                           # every test, grouped by binary
```

## How the tests run the program

The tests in `tests/cli.rs` and `tests/mcp.rs` do not call Rust functions. They start the compiled `svggod` binary (the `CARGO_BIN_EXE_svggod` path that cargo provides), pass arguments, and parse the single JSON object printed on stdout. This is deliberate: the tests see what a user or an AI agent sees, including exit codes (`r4_exit_codes`) and the fact that no command waits for input (`r1_every_command_completes_without_stdin_or_terminal`).

Many tests copy a fixture from `tests/fixtures/` into a temporary directory (`tempfile::tempdir`) and run the binary there, so a test never changes a file in the repository. The only exception is the golden-file test, which writes into `tests/golden/` only when you ask it to ([Golden files](../adding/golden-files.md)).

## If a test fails

1. Re-run just that test with `--nocapture` and read the assertion message. Most messages print the whole JSON envelope.
2. If `r8_output_matches_the_golden_files` fails, read [Golden files](../adding/golden-files.md) before regenerating anything.
3. If only `r15_*` fails, you are probably on a loaded machine or in a debug build; see [Continuous integration gates](../gates/ci.md#timing-budgets).
4. If a test fails on one operating system in CI but not on yours, see [Continuous integration gates](../gates/ci.md#when-one-system-fails).

## The damaged-input test and its limits

`tests/mutate.rs` takes every fixture, golden file and example, damages it at random, and hands it to the library: cut pieces, repeated pieces, absurd numbers such as `1e308`, stray characters. The sequence is fixed, so a failure repeats. It checks one thing: the library answers with findings or an error.

```bash
cargo test --test mutate                                   # 12 changes per file, a few seconds
SVGGOD_MUTATIONS=1500 cargo test --release --test mutate   # about 100,000 inputs, about a minute
SVGGOD_MUTATE_FILE=bad.svg cargo test --release --test mutate one_saved_input -- --nocapture
```

The test limits itself, because the inputs it makes are exactly the kind that can take a machine down:

| Limit | Value | What happens when it is passed |
|---|---|---|
| Memory | 256 MB for the whole test, counted by its own allocator | the allocation fails and the test ends at once |
| Time | 20 seconds for one input | the test ends itself |

Before each input is tried it is written to the system temp directory (`svggod-svg-current.txt` or `svggod-spec-current.txt`; the test prints the path), so the input that ended the test is still there. Try it again alone with `SVGGOD_MUTATE_FILE`, which prints each step as it starts.

> **Warning.** To see what a hostile or damaged file does, use `SVGGOD_MUTATE_FILE`, which runs under the limits above. The `svggod` program limits its own memory too (`--max-memory`, 1 GB unless set; pass a small one such as `--max-memory 64MB` when trying such a file), but it has no time limit, and the library used from another program has neither. A watchdog that looks at memory from outside is too slow to stop a fast allocation, so do not rely on one. Inputs that were bombs once are kept in `known_bombs_stay_defused` in the same file, for the same reason.
