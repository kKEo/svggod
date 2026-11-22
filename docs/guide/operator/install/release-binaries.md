# Release binaries

The repository has a workflow that builds SVGGod for five platforms and attaches the archives to a GitHub release when a version tag is pushed. This page describes what a release will contain and how to use it. Prebuilt binaries become available from the first `v*` tag; until one is published, [build from source](build-from-source.md).

> **Note.** At the time of writing the repository has no tags, and `.github/workflows/release.yml` is not yet committed, so no release exists. Everything below describes what the workflow is written to produce. Treat it as unverified until the first release is out.

## What a release contains

The workflow (`.github/workflows/release.yml`) builds these targets and names each archive `svggod-<version>-<target>`:

| Target | Built on | Archive |
| --- | --- | --- |
| `x86_64-unknown-linux-musl` | ubuntu-latest | `.tar.gz` |
| `aarch64-unknown-linux-musl` | ubuntu-24.04-arm | `.tar.gz` |
| `aarch64-apple-darwin` | macos-latest | `.tar.gz` |
| `x86_64-apple-darwin` | macos-latest | `.tar.gz` |

Each archive unpacks to a folder `svggod-<version>-<target>/` holding the binary `svggod` and `README.md`. The Linux binaries are linked statically with musl, so they run on any distribution without installing libraries. The release also carries a `SHA256SUMS` file listing the checksum of every archive.

The binaries are not signed or notarised; the workflow has no signing step.

## Download and verify

Once a release exists, for version `1.1.0` on Apple silicon:

```bash
gh release download v1.1.0 --repo kKEo/svggod \
  --pattern 'svggod-1.1.0-aarch64-apple-darwin.tar.gz' --pattern SHA256SUMS

shasum -a 256 -c SHA256SUMS --ignore-missing    # macOS; on Linux: sha256sum -c SHA256SUMS --ignore-missing
tar -xzf svggod-1.1.0-aarch64-apple-darwin.tar.gz
install -m 755 svggod-1.1.0-aarch64-apple-darwin/svggod /usr/local/bin/svggod
svggod --version
```

There is no Windows binary.

> **Warning.** A macOS binary downloaded with a browser may be quarantined by Gatekeeper because it is not notarised. Downloading with `gh` or `curl` does not set the quarantine flag. If you must use a browser download, remove the flag with `xattr -d com.apple.quarantine <file>`.

## Pre-releases

A tag with a suffix, such as `v1.2.0-rc.1`, is published as a GitHub pre-release. The tag pattern is `v[0-9]+.[0-9]+.[0-9]+*`.

## Pinning in CI

Pin an exact version and check its checksum, so a pipeline never changes behaviour without a commit:

```bash
VERSION=1.1.0
TARGET=x86_64-unknown-linux-musl
curl -fsSL -o svggod.tar.gz \
  "https://github.com/kKEo/svggod/releases/download/v${VERSION}/svggod-${VERSION}-${TARGET}.tar.gz"
echo "<expected sha256>  svggod.tar.gz" | sha256sum -c -
tar -xzf svggod.tar.gz --strip-components=1 "svggod-${VERSION}-${TARGET}/svggod"
```

Take the expected checksum from the release's `SHA256SUMS` once and commit it. The release-asset URL pattern is GitHub's standard form and has not been exercised for this repository, since no release exists yet; if the repository is private, use `gh release download` with a token instead.
