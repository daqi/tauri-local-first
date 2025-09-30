use crate::{CommandHistoryRecord, util::now_ms};
use super::{HistoryStore};
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// SQLite adapter implementing HistoryStore.
// Minimal schema focused on required fields only to honor minimal footprint principle.

const CREATE_TABLE_SQL: &str = r#"CREATE TABLE IF NOT EXISTS command_history (
    signature TEXT PRIMARY KEY,
    input TEXT NOT NULL,
    intents_summary TEXT NOT NULL,
    overall_status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    plan_size INTEGER NOT NULL,
    explain_used INTEGER NOT NULL
);"#;

pub struct SQLiteHistoryStore {
    conn: Arc<Mutex<Connection>>,
    retention_ms: u64,
}

impl SQLiteHistoryStore {
    pub fn new(path: PathBuf, retention_days: u64) -> anyhow::Result<Self> {
        let should_init = !path.exists();
        let conn = Connection::open(path)?;
        if should_init {
            conn.execute_batch(CREATE_TABLE_SQL)?;
        } else {
            // ensure table exists (idempotent)
            conn.execute_batch(CREATE_TABLE_SQL)?;
        }
        Ok(Self { conn: Arc::new(Mutex::new(conn)), retention_ms: retention_days * 24 * 60 * 60 * 1000 })
    }

    fn purge_internal(conn: &Connection, cutoff: u64) -> rusqlite::Result<usize> {
        let affected = conn.execute("DELETE FROM command_history WHERE created_at < ?1", params![cutoff])?;
        Ok(affected)
    }
}

impl HistoryStore for SQLiteHistoryStore {
    fn save(&mut self, mut record: CommandHistoryRecord) {
        // allocate created_at if missing (0)
        if record.created_at == 0 {
            record.created_at = now_ms();
        }
        if let Ok(conn) = self.conn.lock() {
            let cutoff = record.created_at.saturating_sub(self.retention_ms);
            let _ = Self::purge_internal(&conn, cutoff); // ignore errors for robustness
            let intents_summary_json = serde_json::to_string(&record.intents_summary).unwrap_or("[]".into());
            let _ = conn.execute(
                "INSERT OR REPLACE INTO command_history (signature,input,intents_summary,overall_status,created_at,plan_size,explain_used) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![
                    record.signature,
                    record.input,
                    intents_summary_json,
                    record.overall_status,
                    record.created_at as i64,
                    record.plan_size as i64,
                    if record.explain_used {1} else {0}
                ]
            );
        }
    }

    fn list(&self, limit: usize, after: Option<u64>) -> Vec<CommandHistoryRecord> {
        if limit == 0 { return vec![]; }
        let conn = match self.conn.lock() { Ok(c) => c, Err(_) => return vec![] };
        let mut out = Vec::new();
        if let Some(a) = after {
            let sql = "SELECT signature,input,intents_summary,overall_status,created_at,plan_size,explain_used FROM command_history WHERE created_at > ?1 ORDER BY created_at DESC LIMIT ?2";
            if let Ok(mut stmt) = conn.prepare(sql) {
                if let Ok(rows) = stmt.query_map(params![a as i64, limit as i64], |row| {
                    let intents_summary: String = row.get(2)?;
                    let intents_summary: Vec<String> = serde_json::from_str(&intents_summary).unwrap_or_default();
                    Ok(CommandHistoryRecord {
                        signature: row.get(0)?,
                        input: row.get(1)?,
                        intents_summary,
                        overall_status: row.get(3)?,
                        created_at: row.get::<_, i64>(4)? as u64,
                        plan_size: row.get::<_, i64>(5)? as u32,
                        explain_used: row.get::<_, i64>(6)? == 1,
                    })
                }) {
                    for r in rows { if let Ok(rec) = r { out.push(rec); } }
                }
            }
        } else {
            let sql = "SELECT signature,input,intents_summary,overall_status,created_at,plan_size,explain_used FROM command_history ORDER BY created_at DESC LIMIT ?1";
            if let Ok(mut stmt) = conn.prepare(sql) {
                if let Ok(rows) = stmt.query_map(params![limit as i64], |row| {
                    let intents_summary: String = row.get(2)?;
                    let intents_summary: Vec<String> = serde_json::from_str(&intents_summary).unwrap_or_default();
                    Ok(CommandHistoryRecord {
                        signature: row.get(0)?,
                        input: row.get(1)?,
                        intents_summary,
                        overall_status: row.get(3)?,
                        created_at: row.get::<_, i64>(4)? as u64,
                        plan_size: row.get::<_, i64>(5)? as u32,
                        explain_used: row.get::<_, i64>(6)? == 1,
                    })
                }) { for r in rows { if let Ok(rec) = r { out.push(rec); } } }
            }
        }
        out
    }

    fn purge_older_than(&mut self, cutoff_ts: u64) -> usize {
        if let Ok(conn) = self.conn.lock() {
            if let Ok(count) = Self::purge_internal(&conn, cutoff_ts) { return count; }
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::HistoryStore;
    use crate::CommandHistoryRecord;
    use tempfile::tempdir;

    fn rec(sig: &str, ts: u64) -> CommandHistoryRecord {
        CommandHistoryRecord {
            signature: sig.into(),
            input: format!("input-{sig}"),
            intents_summary: vec!["a".into()],
            overall_status: "success".into(),
            created_at: ts,
            plan_size: 1,
            explain_used: false,
        }
    }

    #[test]
    fn migration_and_insert_list() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("history.db");
        let mut store = SQLiteHistoryStore::new(db_path.clone(), 30).unwrap();
    let now = now_ms();
        store.save(rec("s1", now));
        store.save(rec("s2", now + 10));
        let listed = store.list(10, None);
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].signature, "s2");
    }

    #[test]
    fn purge_on_save() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("history.db");
        let mut store = SQLiteHistoryStore::new(db_path.clone(), 30).unwrap();
    let now = now_ms();
        let old = now - (31 * 24 * 60 * 60 * 1000);
        store.save(rec("old", old));
        store.save(rec("new", now)); // triggers purge
        let listed = store.list(10, None);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].signature, "new");
    }
}
