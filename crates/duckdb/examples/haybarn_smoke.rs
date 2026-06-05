// Smoke proof: the rebranded `haybarn` crate opens the bundled engine and queries.
// Also asserts the engine reports a RELEASE version (no "-dev"), so extensions
// resolve under the `v1.5.3` directory rather than the git source id.
use haybarn::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    let (version, source_id): (String, String) =
        conn.query_row("SELECT library_version, source_id FROM pragma_version()", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?;
    let n: i64 = conn.query_row("SELECT count(*) FROM range(10)", [], |r| r.get(0))?;
    println!("haybarn ok: version()={version}  source_id={source_id}  range(10).count={n}");
    assert_eq!(n, 10);
    assert!(
        !version.contains("-dev"),
        "engine must report a release version so extensions load from the version directory, got {version}",
    );
    Ok(())
}
