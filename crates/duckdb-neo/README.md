# haybarn-neo

### The v2 C API is in rapid development upstream. Consider `haybarn-neo` unstable.

[![Latest Version](https://img.shields.io/crates/v/haybarn-neo.svg)](https://crates.io/crates/haybarn-neo)
[![Documentation](https://img.shields.io/badge/docs.rs-haybarn--neo-orange)](https://docs.rs/haybarn-neo)
[![MIT License](https://img.shields.io/crates/l/haybarn-neo.svg)](LICENSE)

`haybarn-neo` is a safe, ergonomic Rust wrapper for the v2 C API of
[Haybarn](https://github.com/Query-farm-haybarn/haybarn), an independent derived
distribution of DuckDB published by Query Farm LLC ("Haybarn, powered by DuckDB").
It is the Haybarn build of upstream's `duckdb-neo` crate; only the crate names, the
library it links and the extension trust root differ.

You can use it to:

- Query Haybarn with type-safe bindings.
- Read and write Arrow, Parquet, JSON, and CSV formats natively.
- [Create extensions](#extensions) in Rust with various custom functions.

## Quickstart

Create a new project and add the `haybarn-neo` crate:

```shell
cargo new my-app
cd my-app
cargo add haybarn-neo
```

Update `src/main.rs` with the following code:

```rust
use haybarn_neo::{
    Parameters, Result,
    environment::{Environment, StorageLocation},
};

struct Bale {
    id: i32,
    label: String,
}

fn main() -> Result<()> {
    let environment = Environment::new()?;
    let database = environment.open(StorageLocation::InMemory)?;
    let conn = database.connect()?;

    conn.execute(
        "CREATE TABLE bales (id INTEGER PRIMARY KEY, label TEXT)",
        Parameters::None,
    )?;

    // `execute` runs one statement; `parse` splits a script into statements.
    for statement in conn.parse(
        r#"
        INSERT INTO bales (id, label) VALUES (1, 'north field');
        INSERT INTO bales (id, label) VALUES (2, 'south field');
        "#,
    )? {
        conn.execute(statement?, Parameters::None)?;
    }

    conn.execute(
        "INSERT INTO bales (id, label) VALUES (?, ?)",
        Parameters::positional(&[&3, &"east field"]),
    )?;

    // Results arrive as chunks of column vectors.
    let mut bales = Vec::new();
    let mut result = conn.query("FROM bales ORDER BY id", Parameters::None)?;
    while let Some(chunk) = result.next_chunk()? {
        let ids = chunk.get_vector_at::<i32>(0)?.reader()?;
        let labels = chunk.get_vector_at::<String>(1)?.reader()?;
        for (id, label) in ids.iter().zip(labels.iter()) {
            bales.push(Bale {
                id: *id.expect("id is a primary key"),
                label: label.unwrap_or_default().to_owned(),
            });
        }
    }

    for bale in bales {
        println!("{}) {}", bale.id, bale.label);
    }

    Ok(())
}
```

## Feature flags

### Integrations

- `chrono` - Convert `DATE`, `TIME`, and `TIMESTAMP` values to [`chrono`](https://docs.rs/chrono) types.
- `r2d2` - `haybarn_neo::r2d2::ConnectionManager`, a connection pool manager for [r2d2](https://github.com/sfackler/r2d2).
- `uuid` - Read and write `UUID` values as [`uuid::Uuid`](https://docs.rs/uuid).
- `rust_decimal` - Read and write `DECIMAL` values as [`rust_decimal::Decimal`](https://docs.rs/rust_decimal). Vectors support `DECIMAL` widths up to 28.

### Linking against a prebuilt Haybarn library

By default, `haybarn-neo` links against a prebuilt `libhaybarn`. Set `DUCKDB_LIB_DIR` to use a specific one; otherwise these features are tried in order:

- `buildtime-download` _(default)_ - Download the `libhaybarn-<platform>.zip` this crate was built for from the [Haybarn GitHub release](https://github.com/Query-farm-haybarn/haybarn/releases) pinned in `crates/libduckdb-sys/.duckdb-release`. Set `DUCKDB_DOWNLOAD_LIB=0` to skip it.
- `buildtime-pkg-config` _(default)_ - Find a `haybarn` library installed on the system with pkg-config.

### Bindings

- `buildtime-pregenerated` _(default)_ - Use the v2 C API bindings that ship with the crate.
- `buildtime-bindgen` - Generate the bindings at build time instead. Needs libclang and `duckdb_v2.h`, found through `DUCKDB_INCLUDE_DIR`. Overrides `buildtime-pregenerated` when both are enabled.

### Building the engine from source

These features compile the Haybarn engine into your binary, so the linking features above are not needed:

```toml
haybarn-neo = { version = "...", default-features = false, features = ["bundled"] }
```

- `bundled` - Compile the engine from the bundled Haybarn sources.
- `bundled-cmake` - _Experimental_. Compile the engine with its CMake build instead. Always includes the Parquet extension. Requires a git dependency on haybarn-rust, since the published crate does not include the full engine sources.

The following extensions can be statically linked through `bundled-cmake`. Each one implies `bundled-cmake`:

- `bundled-cmake-autocomplete` - SQL autocompletion.
- `bundled-cmake-httpfs` - HTTP(S) and S3 file systems (Linux and macOS only).
- `bundled-cmake-icu` - Time zones, collations and other locale-aware operations.
- `bundled-cmake-tpch` - TPC-H data and queries.
- `bundled-cmake-tpcds` - TPC-DS data and queries.

## libhaybarn-sys

`haybarn-neo` is built on [libhaybarn-sys](https://crates.io/crates/libhaybarn-sys), which provides the raw FFI bindings to the C API and finds or builds the engine library. The `haybarn` crate uses the same crate and version, with the v1 C API. `haybarn-neo` only enables the v2 bindings, exposed as `libhaybarn_sys::v2`. For any part of the C API that `haybarn-neo` does not wrap yet, add `libhaybarn-sys` with the `capi-v2` feature.

## Extensions

_Experimental._ `haybarn-neo` can also be used to write extensions, loadable with `LOAD` or statically linked
into a binary. See [extensions.md](extensions.md).

## Rust version compatibility

`haybarn-neo` shares its MSRV (minimum supported Rust version) with the `haybarn` crate. It is currently Rust 1.88.

## License & trademark

Licensed under the [MIT license](https://github.com/Query-farm-haybarn/haybarn-rust/blob/haybarn-2.0/LICENSE), preserved verbatim from upstream duckdb-rs (Copyright (c) Stichting DuckDB Foundation).

Haybarn is independent of and not endorsed by the DuckDB Foundation. DuckDB is a trademark of the DuckDB Foundation
(<https://duckdb.org/trademark_guidelines>). The original `duckdb-neo` crate lives at <https://github.com/duckdb/duckdb-rs>.
