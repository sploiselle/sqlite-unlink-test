# SQLite Unlink Test

Demonstrates how SQLite logs propagate when database files are unlinked while
connections are open.

## Build & Run

```bash
cargo run
```

## Expected Output

```
1. Creating database and opening connection...

2. Enabling WAL mode...

3. Creating table and inserting data...

4. Verifying files exist...
  test.db exists
  test.db-wal exists
  test.db-shm exists

5. UNLINKING DATABASE FILE...
  test.db has been unlinked!

6. Attempting write after unlink...
  Write succeeded after unlink

7. Demonstrating logs are still connected...
WARN [sqlite] error_code=1: near "INVALID": syntax error in "INVALID SQL STATEMENT"
Expected error: near "INVALID": syntax error in INVALID SQL STATEMENT at offset 0

8. Attempting read after unlink...
  Read succeeded, count: 2

9. Creating another table after unlink...
  Table creation succeeded after unlink

10. Closing connection...
WARN [sqlite] error_code=28: file unlinked while open: /Users/seanloiselle/github/getditto/sqlite-unlink-test/test.db

11. Cleaning up remaining files...

Done
```

## Key Finding

SQLite logs `error_code=28: file unlinked while open` when closing a connection
to a database file that was deleted while the connection was active; it does not
log when the file itself has been unlinked.