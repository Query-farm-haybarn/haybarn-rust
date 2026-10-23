# CLAUDE.md — haybarn-rust

Guidance for Claude Code working in this repository.

## What this repo is

**haybarn-rust** is the Rust client for
[Haybarn](https://github.com/Query-farm-haybarn/haybarn) — an independent
derived distribution of DuckDB ("Haybarn, powered by DuckDB"), published by
Query Farm LLC. This repo is a **hard fork** of `duckdb/duckdb-rs`, based on
upstream tag **`v1.20000.0`** (the crate version encodes the engine version as
`1.MAJOR_MINOR_PATCH.x`, so `1.20000.x` == DuckDB **2.0.0**). This is the
**`haybarn-2.0`** branch; the 1.5 line (`1.10506.x`) is the `haybarn` branch —
never mix the two.

All Haybarn-specific changes are a small, curated **commit stack** on top of the
upstream tag — not scattered edits. Keep it that way: the stack must stay easy to
rebase onto future `duckdb-rs` tags.

## Hard rules

- **Never rename the C API or the `duckdb::` C++ namespace.** Those are the
  upstream ABI surface and Haybarn is intentionally ABI-compatible. In particular
  the bindgen comments in `crates/libduckdb-sys/src/bindgen_bundled_version*.rs`
  reference the C++ `duckdb::` namespace — leave them. The `DUCKDB_*` env vars
  and the `.duckdb-release` file name also stay.
- **On-disk crate directories keep their upstream names** (`crates/duckdb`,
  `crates/libduckdb-sys`, `crates/duckdb-loadable-macros`, `crates/duckdb-neo`). Only the published
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
| `duckdb-loadable-macros` | `haybarn-loadable-macros` | emits `::haybarn::ffi`, `::haybarn_neo::` |
| `duckdb-neo` | `haybarn-neo` | `haybarn_neo` (version `0.20000.x`) |

Migration for downstreams (no source change): `duckdb = { package = "haybarn" }`.

## The bundled sources = the trust root (most important invariant)

The `bundled` feature compiles `crates/libduckdb-sys/duckdb.tar.gz`, the
`package_build.py` source tree (2.0 has no amalgamation) assembled by
`crates/libduckdb-sys/update_sources.py` from the
`crates/libduckdb-sys/duckdb-sources` git submodule. **That submodule is
re-pointed at `Query-farm-haybarn/haybarn`.** The tree includes
`src/main/extension/extension_helper.cpp`, which carries the Haybarn trust root,
so the bundled build embeds the single Haybarn RSA key (core + community) and the
`haybarn-extensions.query.farm` URLs. DuckDB-signed extensions will not load.

### Regenerating the bundled sources (after an engine bump)

**The release-version stamp is mandatory.** In 2.0, `package_build.py` stamps
`v<X.Y>.0-dev<commit count>` unless `OVERRIDE_GIT_DESCRIBE` (taken *verbatim* —
no describe parsing any more) or `DUCKDB_VERSION` is set; upstream's own archive
ships `v2.0.0-dev88614`. A `-dev` engine resolves extensions under the git
source id instead of `/core/v2.0.0/`. `crates/libduckdb-sys/upgrade.sh` sets it
from the crate version; by hand:

```shell
cd crates/libduckdb-sys
git -C duckdb-sources fetch origin tag haybarn-v<version>
git -C duckdb-sources checkout haybarn-v<version>        # e.g. haybarn-v2.0.0-rc5
OVERRIDE_GIT_DESCRIBE=v2.0.0 python3 update_sources.py   # rewrites duckdb.tar.gz
```

Also bump `crates/libduckdb-sys/.duckdb-release` (commit, release URL, tag) to
the same engine build, and commit the archive, the submodule pointer and the pin
together. Verify:

```shell
tar xzf duckdb.tar.gz -O duckdb/src/function/table/version/pragma_version.cpp \
  | grep -E '#define DUCKDB_(VERSION|SOURCE_ID)'      # VERSION must be "v2.0.0", no -dev
tar xzf duckdb.tar.gz -O duckdb/src/main/extension/extension_helper.cpp \
  | grep -q HAYBARN_TRUST_ROOT && echo "trust root OK"
```

The pregenerated bindings only need regenerating (`upgrade.sh`, needs libclang)
when the C API headers change; Haybarn's headers differ from upstream only by
`HAYBARN*` macros, which bindgen's `duckdb` allowlist skips.

## Supported build paths

- **`bundled`** — compiles the vendored Haybarn sources (carries the trust root).
- **Linked (default without `bundled`)** — downloads `libhaybarn-<platform>.zip`
  from the Haybarn GitHub release pinned in `crates/libduckdb-sys/.duckdb-release`
  and links `haybarn`. **Never point the pin at DuckDB's binaries.**
- **`bundled-cmake`** — experimental, git checkouts only; links `haybarn_static`.

## Releasing

Publishing uses crates.io **Trusted Publishing** (GitHub OIDC, no stored token).
`.github/workflows/haybarn-rust.yml` publishes `libhaybarn-sys` →
`haybarn-loadable-macros` → `haybarn` on a `haybarn-v*` tag. The 2.0 line's
tags are `haybarn-v1.20000.*`; the workflow refuses to publish them until the
2.0 publish gate in it is lifted (see the comment there). `haybarn-neo` is
published last, only once the crate exists on crates.io: its first version
needs one manual `cargo publish -p haybarn-neo --no-verify` (after the other
three are on the index) and a Trusted Publishing link; until then the workflow
skips it with a notice.

Release bump for an engine rc: re-pin `duckdb-sources` + regenerate
`duckdb.tar.gz` (stamp `v2.0.0`), update `.duckdb-release`, bump the
`1.20000.0-rc.N` (and neo `0.20000.0-rc.N`) strings in `Cargo.toml` /
`crates/duckdb-neo/Cargo.toml`, `cargo update -w`, commit, tag
`haybarn-v1.20000.0-rc.N`. The crate rc number follows the engine rc.
