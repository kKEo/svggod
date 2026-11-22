# Part II: Install and deploy

SVGGod is one self-contained executable. There is nothing to configure on a server, no service to start and no runtime to install: installing it means putting one file on the machines that need it. This part covers the two ways to get that file and the one piece of wiring that needs care, the MCP server.

| Chapter | Read it when you need to… |
| --- | --- |
| [Build from source](build-from-source.md) | Build the binary yourself, or install it with `cargo install svggod`. |
| [Release binaries](release-binaries.md) | Use the prebuilt archives from a GitHub release. |
| [Setting up the MCP server](mcp-setup.md) | Let an AI agent client call SVGGod as a set of tools. |

## What you are deploying

- **A binary of about 9 MB** (the 1.0.0 release build on macOS arm64 is 9,357,216 bytes). The two fonts are inside it.
- **No network use.** SVGGod never opens a connection, so it runs in an air-gapped build.
- **No required files.** An optional `svggod.toml` next to your diagrams changes which rules apply; see [Configuration](../reference/configuration.md).
- **Four supported targets** in the release workflow: Linux x86_64 and arm64 (static), and macOS Apple silicon and Intel. CI runs the test suite on Linux and macOS. Windows is not built or tested.

## Choosing a path

| Situation | Use |
| --- | --- |
| Developer machine, or a CI image that already has Rust | [Build from source](build-from-source.md), or `cargo install --path .` |
| CI image without Rust | [Release binaries](release-binaries.md), or `brew install kKEo/tap/svggod` |
| An agent client such as a coding assistant | Either of the above, then [MCP setup](mcp-setup.md) |

> **Note.** SVGGod is published as the `svggod` crate on crates.io (`cargo install svggod`, or `cargo binstall svggod` for the prebuilt binary) and as a Homebrew formula (`brew install kKEo/tap/svggod`), from version 1.1.0 on. Older versions exist only under the previous name, `svgdog`. There is no npm wrapper or container image.
