# haybarn-rust

[![Latest Version](https://img.shields.io/crates/v/haybarn.svg)](https://crates.io/crates/haybarn)
[![Documentation](https://img.shields.io/badge/docs.rs-haybarn-orange)](https://docs.rs/haybarn)
[![MIT License](https://img.shields.io/crates/l/haybarn.svg)](LICENSE)

`haybarn` is an ergonomic Rust wrapper for **Haybarn** — an independent derived
distribution of DuckDB, published by Query Farm LLC. It is a rebrand of the
upstream [`duckdb-rs`](https://github.com/duckdb/duckdb-rs) crate and is
intentionally **ABI-compatible** with DuckDB: the C API, the `duckdb::` C++
namespace, headers, and the `DUCKDB_VERSION` are all preserved. The only
differences are the published crate names, the engine library it builds or
links (`libhaybarn`), and the embedded extension trust root.

You can use it to:

- Query Haybarn with type-safe bindings and an API inspired by
  [rusqlite](https://github.com/rusqlite/rusqlite).
- Read and write Arrow, Parquet, JSON, and CSV formats natively.
- Build Haybarn extensions in Rust with custom scalar and table functions.

## Crates

| Crate | Replaces | Purpose |
|---|---|---|
| [`haybarn`](https://crates.io/crates/haybarn) | `duckdb` | ergonomic wrapper |
| [`libhaybarn-sys`](https://crates.io/crates/libhaybarn-sys) | `libduckdb-sys` | low-level C-API FFI |
| [`haybarn-loadable-macros`](https://crates.io/crates/haybarn-loadable-macros) | `duckdb-loadable-macros` | extension proc-macros |
| [`haybarn-neo`](crates/duckdb-neo/README.md) | `duckdb-neo` | experimental wrapper for the v2 C API |

Versioning follows upstream: the crate version encodes the engine version as
`1.MAJOR_MINOR_PATCH.x`, so `1.20000.x` corresponds to the **2.0.0** engine
(`haybarn-neo` uses `0.20000.x`). The 1.5 line (`1.10506.x`) lives on the
`haybarn` branch.

## Quickstart

Add the `haybarn` crate with the `bundled` feature, which compiles the Haybarn
engine from the vendored sources (carrying the Haybarn extension trust root, see
[Extension trust](#extension-trust)). Without `bundled`, the build downloads a
prebuilt Haybarn library instead (see [Build modes](#build-modes)):

```shell
cargo new my-app && cd my-app
cargo add haybarn -F bundled
```

```rust
use haybarn::{params, Connection, Result};

struct Bale {
    id: i32,
    label: String,
}

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch(
        "CREATE TABLE bales (id INTEGER PRIMARY KEY, label TEXT);
         INSERT INTO bales VALUES (1, 'north field'), (2, 'south field');",
    )?;

    let bales = conn
        .prepare("FROM bales ORDER BY id")?
        .query_map([], |row| {
            Ok(Bale { id: row.get(0)?, label: row.get(1)? })
        })?
        .collect::<Result<Vec<_>>>()?;

    for b in bales {
        println!("{}: {}", b.id, b.label);
    }
    Ok(())
}
```

## Migrating from `duckdb-rs`

Because `haybarn` is ABI-compatible, existing `duckdb-rs` code can switch with a
single `Cargo.toml` line and **no source changes** — Cargo's package-rename keeps
the import name as `duckdb`:

```toml
[dependencies]
duckdb = { package = "haybarn", version = "1.20000", features = ["bundled"] }
```

Every `use duckdb::...` keeps working. Fresh projects can instead depend on
`haybarn` directly and write `use haybarn::...`.

> **Loadable-extension authors** (the experimental `loadable-extension` feature)
> must import the crate as `haybarn`, because `haybarn-loadable-macros` emits
> `::haybarn::ffi` paths. The `package = "haybarn"` rename trick does not cover
> that case.

## Extension trust

The `bundled` build compiles a source tree vendored from the
[Haybarn engine](https://github.com/Query-farm-haybarn/haybarn), and the
downloaded libraries are Haybarn's own release builds. Both embed the
**single Haybarn RSA trust root** for both core and community extensions and the
`haybarn-extensions.query.farm` repositories. DuckDB-signed extensions will not
load, by design.

## Build modes

- **`bundled`** compiles the engine from `crates/libduckdb-sys/duckdb.tar.gz`
  with the `cc` crate. `json` and `parquet` can be linked in statically.
- **Linked (default without `bundled`)** downloads `libhaybarn-<platform>.zip`
  from the Haybarn GitHub release pinned in `crates/libduckdb-sys/.duckdb-release`
  (osx-arm64, linux-amd64, linux-arm64, windows-amd64) and links the shared
  `libhaybarn`. Haybarn 2.0 dropped macOS x86_64; releases up to rc5 name the
  macOS archive `osx-universal`, which is tried when `osx-arm64` is missing. The archive is cached under `target/duckdb-download/`.
  `DUCKDB_DOWNLOAD_LIB=0` skips the download; `DUCKDB_LIB_DIR` points at a
  directory holding `libhaybarn` (or, with `DUCKDB_STATIC=1`,
  `libhaybarn_static`) instead. pkg-config and vcpkg are probed for `haybarn`.
- **`bundled-cmake`** (experimental, git checkouts only) builds the engine
  submodule with CMake and links `libhaybarn_static`.

The `DUCKDB_*` environment variables keep their upstream names.

## License & trademark

Licensed under [MIT](LICENSE), preserved verbatim from upstream `duckdb-rs`.
See [NOTICE](NOTICE) for the list of modifications.

Haybarn is **independent of and not endorsed by the DuckDB Foundation**.
DuckDB is a trademark of the DuckDB Foundation
(<https://duckdb.org/trademark_guidelines>). "Haybarn, powered by DuckDB."

The original `duckdb-rs` project lives at
<https://github.com/duckdb/duckdb-rs>.
