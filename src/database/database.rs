use rusqlite::Connection;

refinery::embed_migrations!("migrations");

pub fn migrate() -> Result<(), String> {
    let mut connection = Connection::open("kakeibo.db")
        .map_err(|error| format!("Failed to open the database: {error}"))?;

    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|error| format!("Failed to enable foreign key constraints: {error}"))?;

    migrations::runner()
        .run(&mut connection)
        .map_err(|error| format!("Failed to apply migrations: {error}"))?;

    Ok(())
}
