# CLAUDE.md — haybarn-rust

Guidance for Claude Code working in this repository.

## What this repo is

**haybarn-rust** is the Rust client for
[Haybarn](https://github.com/Query-farm-haybarn/haybarn) — an independent
derived distribution of DuckDB ("Haybarn, powered by DuckDB"), published by
Query Farm LLC. This repo is a **hard fork** of `duckdb/duckdb-rs`, based on
upstream tag **`v1.10503.1`** (the crate version encodes the engine version as
`1.MAJOR_MINOR_PATCH.x`, so `1.10503.x` == DuckDB **1.5.3**).

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

```shell
cd crates/libduckdb-sys
git -C duckdb-sources fetch origin
git -C duckdb-sources checkout haybarn-v<version>   # e.g. haybarn-v1.5.3-rc7
python3 update_sources.py                            # rewrites duckdb.tar.gz
```

Commit the updated `duckdb.tar.gz` and the submodule pointer together. Verify the
new tarball carries the Haybarn key:

```shell
tar xzf duckdb.tar.gz -O duckdb/src/main/extension/extension_helper.cpp \
  | grep -q HAYBARN_TRUST_ROOT && echo OK
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
