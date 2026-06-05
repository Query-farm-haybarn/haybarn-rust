// Smoke proof: the rebranded `haybarn` crate opens the bundled engine and queries.
use haybarn::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    let v: String = conn.query_row("SELECT version()", [], |r| r.get(0))?;
    let n: i64 = conn.query_row("SELECT count(*) FROM range(10)", [], |r| r.get(0))?;
    println!("haybarn ok: engine version()={v}  range(10).count={n}");
    assert_eq!(n, 10);
    Ok(())
}
