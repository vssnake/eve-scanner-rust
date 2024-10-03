use rusqlite::{params, Connection, Result};

pub struct Database {
    pub conn: Connection,
}

impl Database {
    // Constructor para abrir una conexión con la base de datos
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Database { conn };
        
        db.run_migrations()?;
        Ok(db)
    }
    
    fn run_migrations(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS migrations (
                version INTEGER PRIMARY KEY,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            params![],
        )?;
        
        let current_version = self.get_current_version()?;
        let migrations = vec![
            (1, "CREATE TABLE IF NOT EXISTS process_info (
                id INTEGER PRIMARY KEY,
                memory_address TEXT NOT NULL
            )"),
        ];

        for (version, query) in migrations {
            if version > current_version {
                println!("Applying migration version {}", version);
                self.conn.execute(query, params![])?;
                self.set_migration_version(version)?;
            }
        }

        Ok(())
    }
    
    fn get_current_version(&self) -> Result<i32> {
        let mut stmt = self.conn.prepare("SELECT COALESCE(MAX(version), 0) FROM migrations")?;
        let version: i32 = stmt.query_row(params![], |row| row.get(0))?;
        Ok(version)
    }
    
    fn set_migration_version(&self, version: i32) -> Result<()> {
        self.conn.execute(
            "INSERT INTO migrations (version) VALUES (?1)",
            params![version],
        )?;
        Ok(())
    }
}
