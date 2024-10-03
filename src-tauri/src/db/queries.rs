use rusqlite::{params, OptionalExtension, Result};
use crate::db::database::Database;

impl Database {
    pub fn add_process_info(&self, process_id: u32, memory_address: String) -> Result<()> {
        self.conn.execute(
            "INSERT INTO process_info (id, memory_address) VALUES (?1, ?2)",
            params![process_id, memory_address],
        )?;
        Ok(())
    }
    
    pub fn get_process_info(&self, process_id: u32) -> Result<Option<(u32, String)>> {
        let mut stmt = self.conn.prepare("SELECT id, memory_address FROM process_info WHERE id = ?1")?;
        let process_info = stmt.query_row(params![process_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).optional()?;
        Ok(process_info)
    }
    
    pub fn delete_process_info(&self, process_id: u32) -> Result<()> {
        self.conn.execute(
            "DELETE FROM process_info WHERE id = ?1",
            params![process_id],
        )?;
        Ok(())
    }
}
