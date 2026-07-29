# CLAUDE.md — haybarn-rust

Guidance for Claude Code working in this repository.

## What this repo is

**haybarn-rust** is the Rust client for
[Haybarn](https://github.com/Query-farm-haybarn/haybarn) — an independent
derived distribution of DuckDB ("Haybarn, powered by DuckDB"), published by
Query Farm LLC. This repo is a **hard fork** of `duckdb/duckdb-rs`, based on
upstream tag **`v1.10505.0`** (the crate version encodes the engine version as
`1.MAJOR_MINOR_PATCH.x`, so `1.10505.x` == DuckDB **1.5.5**).

All Haybarn-specific changes are a small, curated **commit stack** on top of the
upstream tag — not scattered edits. Keep it that way: the stack must stay easy to
rebase onto future `duckdb-rs` tags.

## Hard rules

- **Never rename the C API or the `duckdb::` C++ namespace.** Those are the
  upstream ABI surface and Haybarn is intentionally ABI-compatible. In particular
  the bindgen comments in `crates/libduckdb-sys/src/bindgen_bundled_version*.rs`
  reference the C++ `duckdb::` namespace — leave them.
- **On-disk crate directories keep their upstream names** (`crates/duckdb`,
  `crates/libduckdb-sys`, `crates/duckdb-loadable-macros`). Only the published
  package `name` is rebranded. This keeps the diff against upstream minimal and
  the submodule path (`crates/libduckdb-sys/duckdb-sources`) stable.
- **Trademark compliance is mandatory.** Product name is always "Haybarn", never
  "DuckDB Haybarn". DuckDB appears only descriptively. Keep the MIT `LICENSE`
  verbatim; keep `NOTICE` accurate. https://duckdb.org/trademark_guidelines.
- **One commit = one concern.**

## Crate name mapping

| Upstream | Haybarn | Rust path |
|---|---|---|
| `duckdb` | `haybarn` | `haybarn` (`[lib] name`) |
| `libduckdb-sys` | `libhaybarn-sys` | `libhaybarn_sys`, `links = "haybarn"` |
| `duckdb-loadable-macros` | `haybarn-loadable-macros` | emits `::haybarn::ffi` |

Migration for downstreams (no source change): `duckdb = { package = "haybarn" }`.

## The bundled amalgamation = the trust root (most important invariant)

The `bundled` feature compiles `crates/libduckdb-sys/duckdb.tar.gz`, an
amalgamation assembled by `crates/libduckdb-sys/update_sources.py` from the
`crates/libduckdb-sys/duckdb-sources` git submodule. **That submodule is
re-pointed at `Query-farm-haybarn/haybarn`.** The assembled tree includes
`src/main/extension/extension_helper.cpp`, which carries the Haybarn trust root,
so the bundled build embeds the single Haybarn RSA key (core + community) and the
`haybarn-extensions.query.farm` URLs. DuckDB-signed extensions will not load.

### Regenerating the amalgamation (after an engine bump)

**`OVERRIDE_GIT_DESCRIBE` is mandatory.** The engine reports its version from a
`git describe`, and an rc commit sits *ahead* of the release tag (e.g.
`haybarn-v1.5.3-rc7` is ~128 commits past the `v1.5.3` base tag), so a bare
describe yields a `-dev` version like `v1.5.4-dev128`. DuckDB's
`ExtensionHelper::GetVersionDirectoryName()` then treats the engine as a dev
build and resolves extensions under the **git source id** instead of the version
directory — so `INSTALL`/`LOAD` would look under a SHA that doesn't exist on
`haybarn-extensions.query.farm`. Stamp the **release** version instead (matching
where Haybarn extensions are published, `/core/v<X.Y.Z>/` + `/community/...`):

```shell
cd crates/libduckdb-sys
git -C duckdb-sources fetch origin
git -C duckdb-sources checkout haybarn-v<version>   # e.g. haybarn-v1.5.3-rc7
# Stamp DUCKDB_VERSION = the RELEASE (no rc/-dev), keep the real commit as source id:
SHA=$(git -C duckdb-sources rev-parse --short=10 HEAD)
OVERRIDE_GIT_DESCRIBE="v1.5.3-0-g${SHA}" python3 update_sources.py   # rewrites duckdb.tar.gz
```

The `-0-` (zero commits since the tag) is what makes `package_build.py` emit the
clean `v1.5.3` rather than a `-devN` string. Commit the updated `duckdb.tar.gz`
and the submodule pointer together. Verify the result:

```shell
tar xzf duckdb.tar.gz -O duckdb/src/function/table/version/pragma_version.cpp \
  | grep -E '#define DUCKDB_(VERSION|SOURCE_ID)'      # VERSION must be "v1.5.3", no -dev
tar xzf duckdb.tar.gz -O duckdb/src/main/extension/extension_helper.cpp \
  | grep -q HAYBARN_TRUST_ROOT && echo "trust root OK"
```

## Supported build paths

- **`bundled`** — the supported, branded path (carries the trust root).
- **Non-bundled** (system `pkg-config`/`vcpkg`, `DUCKDB_DOWNLOAD_LIB`) — inherited
  from upstream, still references DuckDB, **unsupported** until rebranded. Do not
  advertise these for Haybarn.

## Releasing

Publishing uses crates.io **Trusted Publishing** (GitHub OIDC, no stored token).
Each crate name must be created with one manual `cargo publish` first, then the
repo is linked on crates.io. After that, `.github/workflows/haybarn-rust.yml`
publishes on a `haybarn-v*` tag. Publish order: `libhaybarn-sys` →
`haybarn-loadable-macros` → `haybarn`.
