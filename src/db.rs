use anyhow::Result;
use rusqlite::{Connection, params};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 初始化数据库表
    pub fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "
        )?;
        Ok(())
    }

    /// 确保管理员账户存在
    pub fn ensure_admin_exists(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users WHERE username = 'admin'",
            [],
            |row| row.get(0),
        )?;

        if count == 0 {
            let hash = bcrypt::hash("admin123", 10)?;
            conn.execute(
                "INSERT INTO users (username, password_hash) VALUES ('admin', ?1)",
                params![hash],
            )?;
            tracing::info!("📝 已创建默认管理员账户: admin / admin123");
        }

        Ok(())
    }

    /// 验证用户登录
    pub fn verify_user(&self, username: &str, password: &str) -> Result<Option<crate::models::User>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, created_at FROM users WHERE username = ?1"
        )?;

        let user = stmt.query_row(params![username], |row| {
            Ok(crate::models::User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                created_at: row.get(3)?,
            })
        });

        match user {
            Ok(u) => {
                if bcrypt::verify(password, &u.password_hash)? {
                    Ok(Some(u))
                } else {
                    Ok(None)
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// 修改密码
    pub fn change_password(&self, username: &str, new_password: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let hash = bcrypt::hash(new_password, 10)?;
        conn.execute(
            "UPDATE users SET password_hash = ?1 WHERE username = ?2",
            params![hash, username],
        )?;
        Ok(())
    }

    /// 获取配置项
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// 设置配置项
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}
