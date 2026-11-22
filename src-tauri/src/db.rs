use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

/// 本地 SQLite：状态 JSON（主机 / 密钥元数据）+ 终端命令历史。
pub struct Db(Mutex<Connection>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: i64,
    pub host_id: String,
    pub host_label: String,
    pub command: String,
    pub executed_at: i64,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("打开数据库失败：{e}"))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS state (
               key TEXT PRIMARY KEY,
               value TEXT NOT NULL,
               updated_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS command_history (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               host_id TEXT NOT NULL,
               host_label TEXT NOT NULL,
               command TEXT NOT NULL,
               executed_at INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_history_executed
               ON command_history(executed_at DESC);",
        )
        .map_err(|e| format!("初始化数据库失败：{e}"))?;
        Ok(Self(Mutex::new(conn)))
    }

    fn conn(&self) -> Result<MutexGuard<'_, Connection>, String> {
        self.0.lock().map_err(|_| "数据库连接被污染".to_string())
    }

    pub fn get_state(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT value FROM state WHERE key = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![key]).map_err(|e| e.to_string())?;
        match rows.next().map_err(|e| e.to_string())? {
            Some(row) => Ok(Some(row.get(0).map_err(|e| e.to_string())?)),
            None => Ok(None),
        }
    }

    pub fn set_state(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO state(key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![key, value, now_ms()],
        )
        .map_err(|e| format!("写入数据库失败：{e}"))?;
        Ok(())
    }

    pub fn add_history(
        &self,
        host_id: &str,
        host_label: &str,
        command: &str,
        executed_at: i64,
    ) -> Result<i64, String> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO command_history(host_id, host_label, command, executed_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![host_id, host_label, command, executed_at],
        )
        .map_err(|e| format!("写入命令历史失败：{e}"))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_history(&self, query: Option<&str>, limit: u32) -> Result<Vec<HistoryEntry>, String> {
        let conn = self.conn()?;
        let limit = limit.clamp(1, 1000);
        let sql = match query {
            Some(_) => {
                "SELECT id, host_id, host_label, command, executed_at
                 FROM command_history
                 WHERE command LIKE ?1 OR host_label LIKE ?1
                 ORDER BY executed_at DESC, id DESC LIMIT ?2"
            }
            None => {
                "SELECT id, host_id, host_label, command, executed_at
                 FROM command_history
                 ORDER BY executed_at DESC, id DESC LIMIT ?1"
            }
        };
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let map_row = |row: &rusqlite::Row<'_>| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                host_id: row.get(1)?,
                host_label: row.get(2)?,
                command: row.get(3)?,
                executed_at: row.get(4)?,
            })
        };
        let rows = match query {
            Some(q) => {
                let pattern = format!("%{q}%");
                stmt.query_map(params![pattern, limit], map_row)
            }
            None => stmt.query_map(params![limit], map_row),
        }
        .map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("读取命令历史失败：{e}"))
    }

    pub fn clear_history(&self) -> Result<(), String> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM command_history", [])
            .map_err(|e| format!("清空命令历史失败：{e}"))?;
        Ok(())
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> Db {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);

        // 计数器保证并行测试不会撞同一个目录（时间戳在低分辨率时钟上可能重复）。
        let index = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("sfterm-db-{}-{index}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        Db::open(&dir.join("test.db")).unwrap()
    }

    #[test]
    fn state_roundtrip() {
        let db = temp_db();
        assert_eq!(db.get_state("hosts").unwrap(), None);
        db.set_state("hosts", r#"[{"id":"h1"}]"#).unwrap();
        assert_eq!(db.get_state("hosts").unwrap().as_deref(), Some(r#"[{"id":"h1"}]"#));
        db.set_state("hosts", "[]").unwrap();
        assert_eq!(db.get_state("hosts").unwrap().as_deref(), Some("[]"));
    }

    #[test]
    fn history_search_and_clear() {
        let db = temp_db();
        db.add_history("h1", "web-1", "ls -la", 1).unwrap();
        db.add_history("h2", "db-2", "systemctl status nginx", 2).unwrap();

        let all = db.list_history(None, 10).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].command, "systemctl status nginx");

        let matched = db.list_history(Some("nginx"), 10).unwrap();
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].host_label, "db-2");

        db.clear_history().unwrap();
        assert!(db.list_history(None, 10).unwrap().is_empty());
    }
}
