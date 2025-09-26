use colored::*;
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

fn setup_sqlite_logging() {
    let initialized = INITIALIZED.swap(true, Ordering::SeqCst);
    if !initialized {
        unsafe {
            if let Err(error) = rusqlite::trace::config_log(Some(|code, message| match code {
                0 => {
                    // Code 0 is OK, don't include that in the log.
                    println!("{} [sqlite] {}", "INFO".green(), message);
                }
                result_code @ (283 | 539) => {
                    // These are recovery codes after abnormal shutdown
                    // - 283: SQLITE_NOTICE_RECOVER_WAL
                    // - 539: SQLITE_NOTICE_RECOVER_ROLLBACK
                    println!(
                        "{} [sqlite] code={}: sqlite successfully recovered from a previous abnormal shutdown: {}",
                        "DEBUG".cyan(),
                        result_code,
                        message
                    );
                }
                error_code => {
                    // Log any others as warnings
                    println!(
                        "{} [sqlite] error_code={}: {}",
                        "WARN".yellow(),
                        error_code,
                        message
                    );
                }
            })) {
                eprintln!("Failed to setup sqlite logging: {:?}", error);
            }
        }
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Set up SQLite logging exactly like Ditto does
    setup_sqlite_logging();

    let db_path = "test.db";
    let wal_path = "test.db-wal";
    let shm_path = "test.db-shm";

    // Clean up any existing files
    let _ = fs::remove_file(db_path);
    let _ = fs::remove_file(wal_path);
    let _ = fs::remove_file(shm_path);

    println!(
        "\n{}",
        "1. Creating database and opening connection...".bold()
    );
    let conn = Connection::open(db_path)?;

    println!("\n{}", "2. Enabling WAL mode...".bold());
    conn.pragma_update(None, "journal_mode", "WAL")?;

    println!("\n{}", "3. Creating table and inserting data...".bold());
    conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)", [])?;

    conn.execute("INSERT INTO test (value) VALUES (?1)", ["initial data"])?;

    println!("\n{}", "4. Verifying files exist...".bold());
    if Path::new(db_path).exists() {
        println!("  {} exists", db_path.green());
    }
    if Path::new(wal_path).exists() {
        println!("  {} exists", wal_path.green());
    }
    if Path::new(shm_path).exists() {
        println!("  {} exists", shm_path.green());
    }

    println!("\n{}", "5. UNLINKING DATABASE FILE...".bold().red());
    fs::remove_file(db_path)?;
    println!("  {} has been unlinked!", db_path.red());

    // Give SQLite a moment to detect the unlink
    thread::sleep(Duration::from_millis(100));

    println!("\n{}", "6. Attempting write after unlink...".bold());
    match conn.execute(
        "INSERT INTO test (value) VALUES (?1)",
        ["data after unlink"],
    ) {
        Ok(_) => println!("  Write succeeded after unlink"),
        Err(e) => println!("  Write failed: {}", e.to_string().red()),
    }

    println!(
        "\n{}",
        "7. Demonstrating logs are still connected...".bold()
    );
    // Intentionally cause an error by invalid SQL
    match conn.execute("INVALID SQL STATEMENT", []) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error: {}", e.to_string().red()),
    }

    println!("\n{}", "8. Attempting read after unlink...".bold());
    match conn
        .prepare("SELECT COUNT(*) FROM test")?
        .query_row([], |row| {
            let count: i64 = row.get(0)?;
            Ok(count)
        }) {
        Ok(count) => println!("  Read succeeded, count: {}", count),
        Err(e) => println!("  Read failed: {}", e.to_string().red()),
    }

    println!("\n{}", "9. Creating another table after unlink...".bold());
    match conn.execute("CREATE TABLE test2 (id INTEGER PRIMARY KEY)", []) {
        Ok(_) => println!("  Table creation succeeded after unlink"),
        Err(e) => println!("  Table creation failed: {}", e.to_string().red()),
    }

    println!("\n{}", "10. Closing connection...".bold());
    drop(conn);

    println!("\n{}", "11. Cleaning up remaining files...".bold());
    let _ = fs::remove_file(wal_path);
    let _ = fs::remove_file(shm_path);

    println!("\n{}", "Done".bold().green());

    Ok(())
}
