use aes_gcm::{AeadInPlace, Aes256Gcm, Key, KeyInit, Nonce};
use chrono;
use hex::FromHex;
use rand::RngCore;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

const ENCRYPTION_KEY: &str = "7e4f2a3c5b8d9e1f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f";

#[derive(Debug, serde::Serialize)]
pub struct BizAccountRow {
    pub fakeid: String,
    pub nickname: String,
    pub alias: String,
    pub round_head_img: String,
    pub signature: String,
    pub article_count: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BizArticleRow {
    pub aid: String,
    pub title: String,
    pub link: String,
    pub digest: String,
    pub cover: String,
    pub author: String,
    pub publish_time: i64,
}

pub fn encrypt_data(data: &str) -> Result<String, String> {
    let key_bytes =
        <[u8; 32]>::from_hex(ENCRYPTION_KEY).map_err(|e| format!("Invalid key: {}", e))?;
    let key = Key::<Aes256Gcm>::from(key_bytes);
    let cipher = Aes256Gcm::new(&key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let mut buffer = data.as_bytes().to_vec();
    cipher
        .encrypt_in_place(nonce, b"", &mut buffer)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&buffer);
    Ok(hex::encode(&result))
}

pub fn decrypt_data(encrypted_data: &str) -> Result<String, String> {
    let data_bytes = hex::decode(encrypted_data).map_err(|e| format!("Invalid hex: {}", e))?;

    if data_bytes.len() < 12 {
        return Err("Invalid encrypted data".to_string());
    }

    let (nonce_bytes, ciphertext) = data_bytes.split_at(12);
    let key_bytes =
        <[u8; 32]>::from_hex(ENCRYPTION_KEY).map_err(|e| format!("Invalid key: {}", e))?;
    let key = Key::<Aes256Gcm>::from(key_bytes);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let mut buffer = ciphertext.to_vec();
    cipher
        .decrypt_in_place(nonce, b"", &mut buffer)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(buffer).map_err(|e| format!("Invalid UTF-8: {}", e))
}

pub struct Database {
    pub conn: Mutex<Connection>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduledTask {
    pub id: Option<String>,
    pub name: String,
    pub schedule_type: String,
    pub publish_time: String,
    pub topic_source: String,
    #[serde(default)]
    pub hot_source: String,
    pub topic_category_id: String,
    pub topic_pack_id: Option<String>,
    pub blogger_id: String,
    pub word_count: i32,
    pub account_id: String,
    pub author_name: Option<String>,
    pub cover_media_id: String,
    pub publish_type: String,
    pub enabled: bool,
    pub status: String,
    pub week_day: Option<i32>,
    pub month_day: Option<i32>,
    pub blogger_name: String,
    pub blogger_category_id: String,
    pub account_name: String,
    pub topic_category_name: String,
    pub topic_pack_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn default_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub content: String,
    pub cover_media_id: Option<String>,
    pub author_name: Option<String>,
    pub category: Option<String>,
    pub tags: Option<String>,
    pub status: String,
    #[serde(default = "default_timestamp")]
    pub created_at: String,
    #[serde(default = "default_timestamp")]
    pub updated_at: String,
    #[serde(default)]
    pub quality_report: Option<String>,
}

/// 质检库：每一次体检报告的存档（含来源与人设归因）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QualityReportRow {
    pub id: String,
    pub article_id: Option<String>,
    pub article_title: String,
    /// radar | workshop | task | recheck
    pub source: String,
    pub persona_name: Option<String>,
    pub score: i64,
    pub report: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopicCategory {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VirtualBlogger {
    pub id: String,
    pub name: String,
    pub persona: String,
    pub topic_category: String,
    pub topic_category_id: String,
    pub topic_category_name: String,
    pub writing_style: String,
    pub language_style: String,
    pub article_structure: String,
    pub layout_format: String,
    pub tone_keywords: String,
    pub content_bottom_line: String,
    pub default_word_count: i32,
    #[serde(default)]
    pub is_builtin: bool,
    #[serde(default)]
    pub preset_key: String,
    /// 体检教训清单（每行一条，最多 5 条），生成时注入 prompt 帮助人设自我修正
    #[serde(default)]
    pub lessons: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WechatAccount {
    pub id: String,
    pub name: String,
    pub app_id: String,
    pub app_secret: String,
    pub status: String,
    #[serde(default)]
    pub default_cover_media_id: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WechatAccessToken {
    pub account_id: String,
    pub access_token: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopicPack {
    pub id: Option<String>,
    pub name: String,
    pub category_id: String,
    pub category_name: String,
    pub description: Option<String>,
    pub topics: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIApiKey {
    pub id: String,
    pub api_key: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenerationHistory {
    pub id: String,
    pub topic: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Topic {
    pub id: Option<String>,
    pub pack_id: String,
    pub content: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublishHistory {
    pub id: String,
    pub article_id: String,
    pub article_title: String,
    pub account_id: String,
    pub publish_time: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskExecutionLog {
    pub id: String,
    pub task_id: String,
    pub task_name: String,
    pub execution_time: String,
    pub status: String,
    pub publish_status: Option<String>,
    pub topic: Option<String>,
    pub article_title: Option<String>,
    pub duration: Option<i32>,
    pub result: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIConfig {
    pub id: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: i32,
}

impl Database {
    pub fn new(path: &str) -> std::result::Result<Self, String> {
        const MAX_RETRY: u32 = 3;
        const RETRY_DELAY_MS: u64 = 1000;

        let mut last_error: Option<String> = None;

        for attempt in 1..=MAX_RETRY {
            match Connection::open(path) {
                Ok(conn) => {
                    // WAL + busy_timeout：允许调度器线程与命令层并发读写不互斥报错
                    if let Err(e) = conn.execute_batch(
                        "PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000; PRAGMA synchronous=NORMAL;",
                    ) {
                        log::warn!("启用 WAL 失败（回退默认 journal mode）: {}", e);
                    }
                    let db = Database {
                        conn: Mutex::new(conn),
                    };
                    match db.run_migrations() {
                        Ok(_) => {
                            return Ok(db);
                        }
                        Err(e) => {
                            last_error = Some(format!("Failed to run migrations: {}", e));
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(format!("Failed to open database: {}", e));
                }
            }

            if attempt < MAX_RETRY {
                log::warn!(
                    "Database connection attempt {} failed, retrying in {}ms...",
                    attempt,
                    RETRY_DELAY_MS
                );
                thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
            }
        }

        Err(last_error.unwrap_or_else(|| "Unknown database error".to_string()))
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(include_str!("../migrations/001_initial_schema.sql"))?;
        log::info!("Migration 001 executed successfully");

        if !Self::column_exists(&conn, "scheduled_tasks", "created_at") {
            log::info!("Adding created_at and updated_at columns to scheduled_tasks...");
            conn.execute("ALTER TABLE scheduled_tasks ADD COLUMN created_at TEXT NOT NULL DEFAULT '2025-01-01T00:00:00Z'", [])?;
            conn.execute("ALTER TABLE scheduled_tasks ADD COLUMN updated_at TEXT NOT NULL DEFAULT '2025-01-01T00:00:00Z'", [])?;
            log::info!("Migration 002 executed successfully");
        } else {
            log::info!("Migration 002 skipped (columns already exist)");
        }

        if !Self::column_exists(&conn, "task_execution_logs", "topic") {
            log::info!(
                "Adding topic, article_title, and duration columns to task_execution_logs..."
            );
            conn.execute_batch(include_str!("../migrations/003_add_log_fields.sql"))?;
            log::info!("Migration 003 executed successfully");
        } else {
            log::info!("Migration 003 skipped (columns already exist)");
        }

        if !Self::column_exists(&conn, "task_execution_logs", "publish_status") {
            log::info!("Adding publish_status column to task_execution_logs...");
            conn.execute(
                "ALTER TABLE task_execution_logs ADD COLUMN publish_status TEXT",
                [],
            )?;
            log::info!("Migration 004 executed successfully");
        } else {
            log::info!("Migration 004 skipped (columns already exist)");
        }

        if !Self::column_exists(&conn, "scheduled_tasks", "publish_type") {
            log::info!("Adding publish_type column to scheduled_tasks...");
            conn.execute(
                "ALTER TABLE scheduled_tasks ADD COLUMN publish_type TEXT NOT NULL DEFAULT 'draft'",
                [],
            )?;
            log::info!("Migration 005 executed successfully");
        } else {
            log::info!("Migration 005 skipped (columns already exist)");
        }

        // Migration 007: 人设教训清单
        if !Self::column_exists(&conn, "virtual_bloggers", "lessons") {
            conn.execute(
                "ALTER TABLE virtual_bloggers ADD COLUMN lessons TEXT NOT NULL DEFAULT ''",
                [],
            )?;
            log::info!("Migration 007 executed successfully");
        }

        // Migration 009: 定时任务热点话题源
        if !Self::column_exists(&conn, "scheduled_tasks", "hot_source") {
            conn.execute(
                "ALTER TABLE scheduled_tasks ADD COLUMN hot_source TEXT NOT NULL DEFAULT ''",
                [],
            )?;
            log::info!("Migration 009 executed successfully");
        }

        // Migration 010: 公众号默认封面素材
        if !Self::column_exists(&conn, "wechat_accounts", "default_cover_media_id") {
            conn.execute("ALTER TABLE wechat_accounts ADD COLUMN default_cover_media_id TEXT NOT NULL DEFAULT ''", [])?;
            log::info!("Migration 010 executed successfully");
        }

        // Migration 008: 质检库
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS quality_reports (
                id TEXT PRIMARY KEY,
                article_id TEXT,
                article_title TEXT NOT NULL DEFAULT '',
                source TEXT NOT NULL DEFAULT '',
                persona_name TEXT,
                score INTEGER NOT NULL DEFAULT 0,
                report TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            );
            CREATE INDEX IF NOT EXISTS idx_quality_reports_persona ON quality_reports(persona_name);"
        )?;
        log::info!("Migration 008 executed successfully");

        // Migration 006: 写作人设赛道化
        if !Self::column_exists(&conn, "virtual_bloggers", "is_builtin") {
            conn.execute(
                "ALTER TABLE virtual_bloggers ADD COLUMN is_builtin INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
            conn.execute(
                "ALTER TABLE virtual_bloggers ADD COLUMN preset_key TEXT NOT NULL DEFAULT ''",
                [],
            )?;
            log::info!("Migration 006a executed successfully");
        }
        if !Self::column_exists(&conn, "articles", "quality_report") {
            conn.execute("ALTER TABLE articles ADD COLUMN quality_report TEXT", [])?;
            log::info!("Migration 006b executed successfully");
        }
        for track in crate::presets::BUILTIN_TRACKS {
            conn.execute(
                "INSERT INTO virtual_bloggers (id, name, persona, topic_category, topic_category_id,
                    topic_category_name, writing_style, language_style, article_structure,
                    layout_format, tone_keywords, content_bottom_line, default_word_count, is_builtin, preset_key)
                 VALUES (?1, ?2, ?3, '', '', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1, ?12)
                 ON CONFLICT(id) DO UPDATE SET name=excluded.name, topic_category_name=excluded.topic_category_name",
                rusqlite::params![
                    format!("builtin-{}", track.preset_key),
                    track.name,
                    track.persona,
                    track.name,
                    track.writing_style,
                    track.language_style,
                    track.article_structure,
                    track.layout_format,
                    track.tone_keywords,
                    track.content_bottom_line,
                    track.default_word_count,
                    track.preset_key,
                ],
            )?;
        }
        log::info!("Migration 006c: builtin tracks seeded");

        log::info!("All migrations executed successfully");
        Ok(())
    }

    fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
        let mut stmt = match conn.prepare(&format!("PRAGMA table_info({})", table)) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let mut rows = match stmt.query([]) {
            Ok(r) => r,
            Err(_) => return false,
        };
        loop {
            match rows.next() {
                Ok(Some(row)) => {
                    if let Ok(name) = row.get::<_, String>(1) {
                        if name == column {
                            return true;
                        }
                    }
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }
        false
    }

    // ============ Scheduled Tasks ============
    pub fn get_all_tasks(&self) -> Result<Vec<ScheduledTask>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, schedule_type, publish_time, topic_source, topic_category_id,
                    topic_pack_id, blogger_id, word_count, account_id, author_name, cover_media_id,
                    enabled, status, week_day, month_day, blogger_name, blogger_category_id,
                    account_name, topic_category_name, topic_pack_name, created_at, updated_at, publish_type,
                    hot_source
             FROM scheduled_tasks"
        )?;

        let tasks = stmt
            .query_map([], |row| {
                Ok(ScheduledTask {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    schedule_type: row.get(2)?,
                    publish_time: row.get(3)?,
                    topic_source: row.get(4)?,
                    topic_category_id: row.get(5)?,
                    topic_pack_id: row.get(6)?,
                    blogger_id: row.get(7)?,
                    word_count: row.get(8)?,
                    account_id: row.get(9)?,
                    author_name: row.get(10)?,
                    cover_media_id: row.get(11)?,
                    enabled: row.get::<_, i32>(12)? != 0,
                    status: row.get(13)?,
                    week_day: row.get(14)?,
                    month_day: row.get(15)?,
                    blogger_name: row.get(16)?,
                    blogger_category_id: row.get(17)?,
                    account_name: row.get(18)?,
                    topic_category_name: row.get(19)?,
                    topic_pack_name: row.get(20)?,
                    created_at: row.get(21)?,
                    updated_at: row.get(22)?,
                    publish_type: row.get(23)?,
                    hot_source: row.get::<_, Option<String>>(24)?.unwrap_or_default(),
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(tasks)
    }

    pub fn add_task(&self, task: &ScheduledTask) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let task_id = task
            .id
            .clone()
            .unwrap_or_else(|| format!("{}", chrono::Utc::now().timestamp_millis()));
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        conn.execute(
            "INSERT INTO scheduled_tasks (id, name, schedule_type, publish_time, topic_source,
                topic_category_id, topic_pack_id, blogger_id, word_count, account_id, author_name,
                cover_media_id, enabled, status, week_day, month_day, blogger_name,
                blogger_category_id, account_name, topic_category_name, topic_pack_name,
                created_at, updated_at, publish_type, hot_source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)",
            params![
                task_id,
                task.name,
                task.schedule_type,
                task.publish_time,
                task.topic_source,
                task.topic_category_id,
                task.topic_pack_id,
                task.blogger_id,
                task.word_count,
                task.account_id,
                task.author_name,
                task.cover_media_id,
                task.enabled as i32,
                task.status,
                task.week_day,
                task.month_day,
                task.blogger_name,
                task.blogger_category_id,
                task.account_name,
                task.topic_category_name,
                task.topic_pack_name,
                now,
                now,
                task.publish_type,
                task.hot_source,
            ],
        )?;
        Ok(())
    }

    pub fn update_task(&self, task: &ScheduledTask) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        conn.execute(
            "UPDATE scheduled_tasks SET name=?2, schedule_type=?3, publish_time=?4, topic_source=?5,
                topic_category_id=?6, topic_pack_id=?7, blogger_id=?8, word_count=?9, account_id=?10,
                author_name=?11, cover_media_id=?12, enabled=?13, status=?14, week_day=?15, month_day=?16,
                blogger_name=?17, blogger_category_id=?18, account_name=?19, topic_category_name=?20,
                topic_pack_name=?21, publish_type=?22, hot_source=?23,
                updated_at=?24 WHERE id=?1",
            params![
                task.id,
                task.name,
                task.schedule_type,
                task.publish_time,
                task.topic_source,
                task.topic_category_id,
                task.topic_pack_id,
                task.blogger_id,
                task.word_count,
                task.account_id,
                task.author_name,
                task.cover_media_id,
                task.enabled as i32,
                task.status,
                task.week_day,
                task.month_day,
                task.blogger_name,
                task.blogger_category_id,
                task.account_name,
                task.topic_category_name,
                task.topic_pack_name,
                task.publish_type,
                task.hot_source,
                now,
            ],
        )?;
        Ok(())
    }

    pub fn delete_task(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM scheduled_tasks WHERE id=?1", [id])?;
        Ok(())
    }

    pub fn get_task(&self, id: &str) -> Result<Option<ScheduledTask>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, schedule_type, publish_time, topic_source, topic_category_id,
                    topic_pack_id, blogger_id, word_count, account_id, author_name, cover_media_id,
                    enabled, status, week_day, month_day, blogger_name, blogger_category_id,
                    account_name, topic_category_name, topic_pack_name, created_at, updated_at, publish_type,
                    hot_source
             FROM scheduled_tasks WHERE id=?1"
        )?;

        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(ScheduledTask {
                id: row.get(0)?,
                name: row.get(1)?,
                schedule_type: row.get(2)?,
                publish_time: row.get(3)?,
                topic_source: row.get(4)?,
                topic_category_id: row.get(5)?,
                topic_pack_id: row.get(6)?,
                blogger_id: row.get(7)?,
                word_count: row.get(8)?,
                account_id: row.get(9)?,
                author_name: row.get(10)?,
                cover_media_id: row.get(11)?,
                enabled: row.get::<_, i32>(12)? != 0,
                status: row.get(13)?,
                week_day: row.get(14)?,
                month_day: row.get(15)?,
                blogger_name: row.get(16)?,
                blogger_category_id: row.get(17)?,
                account_name: row.get(18)?,
                topic_category_name: row.get(19)?,
                topic_pack_name: row.get(20)?,
                created_at: row.get(21)?,
                updated_at: row.get(22)?,
                publish_type: row.get(23)?,
                hot_source: row.get::<_, Option<String>>(24)?.unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    // ============ Articles ============
    pub fn get_all_articles(&self) -> Result<Vec<Article>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, content, cover_media_id, author_name, category, tags, status, created_at, updated_at, quality_report
             FROM articles ORDER BY created_at DESC"
        )?;

        let articles = stmt
            .query_map([], |row| {
                Ok(Article {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    cover_media_id: row.get(3)?,
                    author_name: row.get(4)?,
                    category: row.get(5)?,
                    tags: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    quality_report: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(articles)
    }

    pub fn get_article(&self, id: &str) -> Result<Option<Article>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, content, cover_media_id, author_name, category, tags, status, created_at, updated_at, quality_report
             FROM articles WHERE id=?1"
        )?;

        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Article {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                cover_media_id: row.get(3)?,
                author_name: row.get(4)?,
                category: row.get(5)?,
                tags: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                quality_report: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn add_article(&self, article: &Article) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO articles (id, title, content, cover_media_id, author_name, category, tags, status, created_at, updated_at, quality_report)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                article.id,
                article.title,
                article.content,
                article.cover_media_id,
                article.author_name,
                article.category,
                article.tags,
                article.status,
                article.created_at,
                article.updated_at,
                article.quality_report,
            ],
        )?;
        Ok(())
    }

    pub fn update_article(&self, article: &Article) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE articles SET title=?2, content=?3, cover_media_id=?4, author_name=?5, category=?6,
                tags=?7, status=?8, updated_at=?9, quality_report=?10 WHERE id=?1",
            params![
                article.id,
                article.title,
                article.content,
                article.cover_media_id,
                article.author_name,
                article.category,
                article.tags,
                article.status,
                article.updated_at,
                article.quality_report,
            ],
        )?;
        Ok(())
    }

    pub fn delete_article(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM articles WHERE id=?1", [id])?;
        Ok(())
    }

    pub fn save_articles_batch(&self, articles: &[Article]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        for article in articles {
            tx.execute(
                "INSERT OR REPLACE INTO articles (id, title, content, cover_media_id, author_name, category, tags, status, created_at, updated_at, quality_report)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    article.id,
                    article.title,
                    article.content,
                    article.cover_media_id,
                    article.author_name,
                    article.category,
                    article.tags,
                    article.status,
                    article.created_at,
                    article.updated_at,
                    article.quality_report,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    // ============ Topic Categories ============
    pub fn get_all_topic_categories(&self) -> Result<Vec<TopicCategory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, name, description, created_at FROM topic_categories")?;
        let categories = stmt
            .query_map([], |row| {
                Ok(TopicCategory {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(categories)
    }

    pub fn add_topic_category(&self, category: &TopicCategory) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO topic_categories (id, name, description, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![category.id, category.name, category.description, category.created_at],
        )?;
        Ok(())
    }

    pub fn update_topic_category(&self, category: &TopicCategory) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE topic_categories SET name=?2, description=?3 WHERE id=?1",
            params![category.id, category.name, category.description],
        )?;
        Ok(())
    }

    pub fn delete_topic_category(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM topic_categories WHERE id=?1", [id])?;
        Ok(())
    }

    // ============ Virtual Bloggers ============
    pub fn get_all_bloggers(&self) -> Result<Vec<VirtualBlogger>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, persona, topic_category, topic_category_id, topic_category_name,
                    writing_style, language_style, article_structure, layout_format,
                    tone_keywords, content_bottom_line, default_word_count, is_builtin, preset_key, lessons FROM virtual_bloggers"
        )?;
        let bloggers = stmt
            .query_map([], |row| {
                Ok(VirtualBlogger {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    persona: row.get(2)?,
                    topic_category: row.get(3)?,
                    topic_category_id: row.get(4)?,
                    topic_category_name: row.get(5)?,
                    writing_style: row.get(6)?,
                    language_style: row.get(7)?,
                    article_structure: row.get(8)?,
                    layout_format: row.get(9)?,
                    tone_keywords: row.get(10)?,
                    content_bottom_line: row.get(11)?,
                    default_word_count: row.get(12)?,
                    is_builtin: row.get::<_, Option<i64>>(13)?.unwrap_or(0) != 0,
                    preset_key: row.get::<_, Option<String>>(14)?.unwrap_or_default(),
                    lessons: row.get::<_, Option<String>>(15)?.unwrap_or_default(),
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(bloggers)
    }

    pub fn add_blogger(&self, blogger: &VirtualBlogger) -> Result<()> {
        log::debug!("[DB] add_blogger called with: {:?}", blogger);
        let conn = self.conn.lock().unwrap();
        let result = conn.execute(
            "INSERT INTO virtual_bloggers (id, name, persona, topic_category, topic_category_id,
                topic_category_name, writing_style, language_style, article_structure,
                layout_format, tone_keywords, content_bottom_line, default_word_count, is_builtin, preset_key, lessons)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                blogger.id,
                blogger.name,
                blogger.persona,
                blogger.topic_category,
                blogger.topic_category_id,
                blogger.topic_category_name,
                blogger.writing_style,
                blogger.language_style,
                blogger.article_structure,
                blogger.layout_format,
                blogger.tone_keywords,
                blogger.content_bottom_line,
                blogger.default_word_count,
                blogger.is_builtin as i64,
                blogger.preset_key,
                blogger.lessons,
            ],
        );
        result.map(|_| ())
    }

    pub fn update_blogger(&self, blogger: &VirtualBlogger) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE virtual_bloggers SET name=?2, persona=?3, topic_category=?4, topic_category_id=?5,
                topic_category_name=?6, writing_style=?7, language_style=?8, article_structure=?9,
                layout_format=?10, tone_keywords=?11, content_bottom_line=?12, default_word_count=?13,
                is_builtin=?14, preset_key=?15, lessons=?16 WHERE id=?1",
            params![
                blogger.id,
                blogger.name,
                blogger.persona,
                blogger.topic_category,
                blogger.topic_category_id,
                blogger.topic_category_name,
                blogger.writing_style,
                blogger.language_style,
                blogger.article_structure,
                blogger.layout_format,
                blogger.tone_keywords,
                blogger.content_bottom_line,
                blogger.default_word_count,
                blogger.is_builtin as i64,
                blogger.preset_key,
                blogger.lessons,
            ],
        )?;
        Ok(())
    }

    pub fn delete_blogger(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM virtual_bloggers WHERE id=?1", [id])?;
        Ok(())
    }

    pub fn get_blogger(&self, id: &str) -> Result<Option<VirtualBlogger>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, persona, topic_category, topic_category_id, topic_category_name,
                    writing_style, language_style, article_structure, layout_format,
                    tone_keywords, content_bottom_line, default_word_count, is_builtin, preset_key, lessons
             FROM virtual_bloggers WHERE id=?1"
        )?;

        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(VirtualBlogger {
                id: row.get(0)?,
                name: row.get(1)?,
                persona: row.get(2)?,
                topic_category: row.get(3)?,
                topic_category_id: row.get(4)?,
                topic_category_name: row.get(5)?,
                writing_style: row.get(6)?,
                language_style: row.get(7)?,
                article_structure: row.get(8)?,
                layout_format: row.get(9)?,
                tone_keywords: row.get(10)?,
                content_bottom_line: row.get(11)?,
                default_word_count: row.get(12)?,
                is_builtin: row.get::<_, Option<i64>>(13)?.unwrap_or(0) != 0,
                preset_key: row.get::<_, Option<String>>(14)?.unwrap_or_default(),
                lessons: row.get::<_, Option<String>>(15)?.unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    // ============ Quality Reports（质检库） ============
    pub fn add_quality_report(&self, r: &QualityReportRow) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO quality_reports (id, article_id, article_title, source, persona_name, score, report, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![r.id, r.article_id, r.article_title, r.source, r.persona_name, r.score, r.report, r.created_at],
        )?;
        Ok(())
    }

    pub fn record_quality_report(
        &self,
        article_id: Option<&str>,
        article_title: &str,
        source: &str,
        persona_name: Option<&str>,
        score: i64,
        report: &str,
    ) {
        let row = QualityReportRow {
            id: format!("{}", chrono::Utc::now().timestamp_millis()),
            article_id: article_id.map(|s| s.to_string()),
            article_title: article_title.to_string(),
            source: source.to_string(),
            persona_name: persona_name.map(|s| s.to_string()),
            score,
            report: report.to_string(),
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        if let Err(e) = self.add_quality_report(&row) {
            log::warn!("[质检库] 报告入库失败: {}", e);
        }
    }

    pub fn get_quality_reports(
        &self,
        persona_name: Option<&str>,
        limit: i64,
    ) -> Result<Vec<QualityReportRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = if persona_name.is_some() {
            conn.prepare(
                "SELECT id, article_id, article_title, source, persona_name, score, report, created_at
                 FROM quality_reports WHERE persona_name = ?1 ORDER BY created_at DESC LIMIT ?2"
            )?
        } else {
            conn.prepare(
                "SELECT id, article_id, article_title, source, persona_name, score, report, created_at
                 FROM quality_reports ORDER BY created_at DESC LIMIT ?2"
            )?
        };
        let name_arg = persona_name.map(|s| s.to_string());
        let rows = stmt.query_map(rusqlite::params![name_arg, limit], |row| {
            Ok(QualityReportRow {
                id: row.get(0)?,
                article_id: row.get(1)?,
                article_title: row.get(2)?,
                source: row.get(3)?,
                persona_name: row.get(4)?,
                score: row.get(5)?,
                report: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        rows.collect()
    }

    pub fn delete_quality_report(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM quality_reports WHERE id = ?1", [id])?;
        Ok(())
    }

    // ============ Wechat Accounts ============
    pub fn get_all_accounts(&self) -> Result<Vec<WechatAccount>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, app_id, app_secret, status, default_cover_media_id, created_at FROM wechat_accounts"
        )?;
        let accounts = stmt
            .query_map([], |row| {
                Ok(WechatAccount {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    app_id: row.get(2)?,
                    app_secret: row.get(3)?,
                    status: row.get(4)?,
                    default_cover_media_id: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(accounts)
    }

    pub fn add_account(&self, account: &WechatAccount) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO wechat_accounts (id, name, app_id, app_secret, status, default_cover_media_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                account.id,
                account.name,
                account.app_id,
                account.app_secret,
                account.status,
                account.default_cover_media_id,
                account.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_account(&self, account: &WechatAccount) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE wechat_accounts SET name=?2, app_id=?3, app_secret=?4, status=?5,
                default_cover_media_id=?6 WHERE id=?1",
            params![
                account.id,
                account.name,
                account.app_id,
                account.app_secret,
                account.status,
                account.default_cover_media_id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_account(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM wechat_accounts WHERE id=?1", [id])?;
        Ok(())
    }

    pub fn save_wechat_access_token(&self, token: &WechatAccessToken) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO wechat_access_tokens (account_id, access_token, expires_at)
             VALUES (?1, ?2, ?3)",
            params![token.account_id, token.access_token, token.expires_at],
        )?;
        Ok(())
    }

    pub fn get_wechat_access_token(&self, account_id: &str) -> Result<Option<WechatAccessToken>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT account_id, access_token, expires_at FROM wechat_access_tokens WHERE account_id=?1"
        )?;
        let mut rows = stmt.query([account_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(WechatAccessToken {
                account_id: row.get(0)?,
                access_token: row.get(1)?,
                expires_at: row.get(2)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn delete_wechat_access_token(&self, account_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM wechat_access_tokens WHERE account_id=?1",
            [account_id],
        )?;
        Ok(())
    }

    // ============ Topic Packs ============
    pub fn get_all_topic_packs(&self) -> Result<Vec<TopicPack>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, category_id, category_name, description, topics, created_at
             FROM topic_packs",
        )?;
        let packs = stmt
            .query_map([], |row| {
                Ok(TopicPack {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    category_id: row.get(2)?,
                    category_name: row.get(3)?,
                    description: row.get(4)?,
                    topics: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(packs)
    }

    pub fn add_topic_pack(&self, pack: &TopicPack) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let pack_id = pack
            .id
            .clone()
            .unwrap_or_else(|| format!("{}", chrono::Utc::now().timestamp_millis()));
        conn.execute(
            "INSERT INTO topic_packs (id, name, category_id, category_name, description, topics, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                pack_id,
                pack.name,
                pack.category_id,
                pack.category_name,
                pack.description,
                pack.topics,
                pack.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_topic_pack(&self, pack: &TopicPack) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let pack_id = pack.id.as_ref().ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("TopicPack id is required for update".into())
        })?;
        conn.execute(
            "UPDATE topic_packs SET name=?2, category_id=?3, category_name=?4, description=?5,
                topics=?6 WHERE id=?1",
            params![
                pack_id,
                pack.name,
                pack.category_id,
                pack.category_name,
                pack.description,
                pack.topics,
            ],
        )?;
        Ok(())
    }

    pub fn delete_topic_pack(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM topic_packs WHERE id=?1", [id])?;
        Ok(())
    }

    // ============ AI API Keys ============
    pub fn get_ai_api_key(&self) -> Result<Option<AIApiKey>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, api_key, created_at FROM ai_api_keys LIMIT 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let encrypted_key: String = row.get(1)?;
            let decrypted_key = decrypt_data(&encrypted_key).map_err(|e| {
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                    Some(e),
                )
            })?;
            Ok(Some(AIApiKey {
                id: row.get(0)?,
                api_key: decrypted_key,
                created_at: row.get(2)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn save_ai_api_key(&self, api_key: &AIApiKey) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        log::debug!(
            "[DEBUG] save_ai_api_key: encrypting key of length {}",
            api_key.api_key.len()
        );
        let encrypted_key = encrypt_data(&api_key.api_key).map_err(|e| {
            log::error!("[DEBUG] encrypt_data error: {}", e);
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                Some(e),
            )
        })?;
        log::debug!(
            "[DEBUG] save_ai_api_key: encrypted key length {}",
            encrypted_key.len()
        );
        conn.execute("DELETE FROM ai_api_keys", [])?;
        let result = conn.execute(
            "INSERT INTO ai_api_keys (id, api_key, created_at) VALUES (?1, ?2, ?3)",
            params![api_key.id, encrypted_key, api_key.created_at],
        )?;
        log::debug!("[DEBUG] save_ai_api_key: insert result = {:?}", result);
        Ok(())
    }

    // ============ AI Config ============
    pub fn get_ai_config(&self) -> Result<Option<AIConfig>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, base_url, model, temperature, max_tokens FROM ai_config LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(AIConfig {
                id: row.get(0)?,
                base_url: row.get(1)?,
                model: row.get(2)?,
                temperature: row.get(3)?,
                max_tokens: row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn save_ai_config(&self, config: &AIConfig) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM ai_config", [])?;
        conn.execute(
            "INSERT INTO ai_config (id, base_url, model, temperature, max_tokens) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![config.id, config.base_url, config.model, config.temperature, config.max_tokens],
        )?;
        Ok(())
    }

    // ============ Task Execution Logs ============
    pub fn get_task_logs(&self, task_id: Option<&str>) -> Result<Vec<TaskExecutionLog>> {
        let conn = self.conn.lock().unwrap();
        let sql = match task_id {
            Some(_) => "SELECT id, task_id, task_name, execution_time, status, publish_status, topic, article_title, duration, result, error_message
                       FROM task_execution_logs WHERE task_id=?1 ORDER BY execution_time DESC",
            None => "SELECT id, task_id, task_name, execution_time, status, publish_status, topic, article_title, duration, result, error_message
                   FROM task_execution_logs ORDER BY execution_time DESC",
        };

        let mut stmt = conn.prepare(sql)?;
        let logs = if let Some(tid) = task_id {
            stmt.query_map([tid], |row| {
                Ok(TaskExecutionLog {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    task_name: row.get(2)?,
                    execution_time: row.get(3)?,
                    status: row.get(4)?,
                    publish_status: row.get(5)?,
                    topic: row.get(6)?,
                    article_title: row.get(7)?,
                    duration: row.get(8)?,
                    result: row.get(9)?,
                    error_message: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(TaskExecutionLog {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    task_name: row.get(2)?,
                    execution_time: row.get(3)?,
                    status: row.get(4)?,
                    publish_status: row.get(5)?,
                    topic: row.get(6)?,
                    article_title: row.get(7)?,
                    duration: row.get(8)?,
                    result: row.get(9)?,
                    error_message: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?
        };
        Ok(logs)
    }

    pub fn add_task_log(&self, log: &TaskExecutionLog) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO task_execution_logs (id, task_id, task_name, execution_time, status, publish_status, topic, article_title, duration, result, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                log.id,
                log.task_id,
                log.task_name,
                log.execution_time,
                log.status,
                log.publish_status,
                log.topic,
                log.article_title,
                log.duration,
                log.result,
                log.error_message,
            ],
        )?;
        Ok(())
    }

    pub fn clear_task_logs(&self, task_id: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        match task_id {
            Some(tid) => {
                conn.execute("DELETE FROM task_execution_logs WHERE task_id=?1", [tid])?;
            }
            None => {
                conn.execute("DELETE FROM task_execution_logs", [])?;
            }
        }
        Ok(())
    }

    // ============ Generation History ============
    pub fn get_all_generation_history(&self) -> Result<Vec<GenerationHistory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, topic, content, created_at FROM generation_history ORDER BY created_at DESC")?;
        let history = stmt
            .query_map([], |row| {
                Ok(GenerationHistory {
                    id: row.get(0)?,
                    topic: row.get(1)?,
                    content: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(history)
    }

    pub fn add_generation_history(&self, history: &GenerationHistory) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO generation_history (id, topic, content, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                history.id,
                history.topic,
                history.content,
                history.created_at
            ],
        )?;
        Ok(())
    }

    pub fn clear_generation_history(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM generation_history", [])?;
        Ok(())
    }

    // ============ Topics ============
    pub fn get_topics_by_pack(&self, pack_id: Option<&str>) -> Result<Vec<Topic>> {
        let conn = self.conn.lock().unwrap();
        let sql = match pack_id {
            Some(_) => {
                "SELECT id, pack_id, content, status, created_at FROM topics WHERE pack_id=?1"
            }
            None => "SELECT id, pack_id, content, status, created_at FROM topics",
        };
        let mut stmt = conn.prepare(sql)?;
        let topics = if let Some(pid) = pack_id {
            stmt.query_map([pid], |row| {
                Ok(Topic {
                    id: Some(row.get(0)?),
                    pack_id: row.get(1)?,
                    content: row.get(2)?,
                    status: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(Topic {
                    id: Some(row.get(0)?),
                    pack_id: row.get(1)?,
                    content: row.get(2)?,
                    status: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?
        };
        Ok(topics)
    }

    pub fn add_topic(&self, topic: &Topic) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let topic_id = topic
            .id
            .clone()
            .unwrap_or_else(|| format!("{}", chrono::Utc::now().timestamp_millis()));
        conn.execute(
            "INSERT INTO topics (id, pack_id, content, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                topic_id,
                topic.pack_id,
                topic.content,
                topic.status,
                topic.created_at
            ],
        )?;
        Ok(())
    }

    pub fn add_topic_pack_with_topics(&self, pack: &TopicPack, topics: &[Topic]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let pack_id = pack
            .id
            .clone()
            .unwrap_or_else(|| format!("{}", chrono::Utc::now().timestamp_millis()));

        tx.execute(
            "INSERT INTO topic_packs (id, name, category_id, category_name, description, topics, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                pack_id,
                pack.name,
                pack.category_id,
                pack.category_name,
                pack.description,
                pack.topics,
                pack.created_at,
            ],
        )?;

        for topic in topics {
            let topic_id = topic
                .id
                .clone()
                .unwrap_or_else(|| format!("{}", chrono::Utc::now().timestamp_millis()));
            tx.execute(
                "INSERT INTO topics (id, pack_id, content, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    topic_id,
                    pack_id,
                    topic.content,
                    topic.status,
                    topic.created_at
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn update_topic(&self, topic: &Topic) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let topic_id = topic.id.as_ref().ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("Topic id is required for update".into())
        })?;
        conn.execute(
            "UPDATE topics SET pack_id=?2, content=?3, status=?4 WHERE id=?1",
            params![topic_id, topic.pack_id, topic.content, topic.status],
        )?;
        Ok(())
    }

    pub fn delete_topic(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM topics WHERE id=?1", [id])?;
        Ok(())
    }

    pub fn delete_topic_pack_with_topics(&self, pack_id: &str) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM topics WHERE pack_id=?1", [pack_id])?;
        tx.execute("DELETE FROM topic_packs WHERE id=?1", [pack_id])?;

        tx.commit()?;
        Ok(())
    }

    // ============ Publish History ============
    pub fn get_all_publish_history(&self) -> Result<Vec<PublishHistory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, article_id, article_title, account_id, publish_time, status FROM publish_history ORDER BY publish_time DESC")?;
        let history = stmt
            .query_map([], |row| {
                Ok(PublishHistory {
                    id: row.get(0)?,
                    article_id: row.get(1)?,
                    article_title: row.get(2)?,
                    account_id: row.get(3)?,
                    publish_time: row.get(4)?,
                    status: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(history)
    }

    pub fn add_publish_history(&self, history: &PublishHistory) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO publish_history (id, article_id, article_title, account_id, publish_time, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![history.id, history.article_id, history.article_title, history.account_id, history.publish_time, history.status],
        )?;
        Ok(())
    }

    pub fn clear_publish_history(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM publish_history", [])?;
        Ok(())
    }

    // ============ 爆文工坊：博主文章库 ============

    pub fn ensure_hot_workshop_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS biz_accounts (
                fakeid TEXT PRIMARY KEY,
                nickname TEXT NOT NULL,
                alias TEXT DEFAULT '',
                round_head_img TEXT DEFAULT '',
                signature TEXT DEFAULT '',
                added_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS biz_articles (
                aid TEXT PRIMARY KEY,
                fakeid TEXT NOT NULL,
                title TEXT NOT NULL,
                link TEXT NOT NULL,
                digest TEXT DEFAULT '',
                cover TEXT DEFAULT '',
                author TEXT DEFAULT '',
                publish_time INTEGER NOT NULL,
                fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_biz_articles_fakeid ON biz_articles(fakeid, publish_time DESC);
            CREATE TABLE IF NOT EXISTS mp_credential (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                cookie_enc TEXT NOT NULL,
                token TEXT NOT NULL,
                saved_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;
        Ok(())
    }

    pub fn save_mp_credential(&self, cookie: &str, token: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let cookie_enc = encrypt_data(cookie).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e,
            )))
        })?;
        conn.execute(
            "INSERT INTO mp_credential (id, cookie_enc, token, saved_at) VALUES (1, ?1, ?2, datetime('now'))
             ON CONFLICT(id) DO UPDATE SET cookie_enc = ?1, token = ?2, saved_at = datetime('now')",
            params![cookie_enc, token],
        )?;
        Ok(())
    }

    /// 返回 (解密后的 cookie, token, saved_at)
    pub fn get_mp_credential(
        &self,
    ) -> std::result::Result<Option<(String, String, String)>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT cookie_enc, token, saved_at FROM mp_credential WHERE id = 1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let cookie_enc: String = row.get(0).map_err(|e| e.to_string())?;
            let token: String = row.get(1).map_err(|e| e.to_string())?;
            let saved_at: String = row.get(2).map_err(|e| e.to_string())?;
            let cookie = decrypt_data(&cookie_enc)?;
            return Ok(Some((cookie, token, saved_at)));
        }
        Ok(None)
    }

    pub fn upsert_biz_account(
        &self,
        fakeid: &str,
        nickname: &str,
        alias: &str,
        round_head_img: &str,
        signature: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO biz_accounts (fakeid, nickname, alias, round_head_img, signature)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(fakeid) DO UPDATE SET nickname = ?2, alias = ?3, round_head_img = ?4, signature = ?5",
            params![fakeid, nickname, alias, round_head_img, signature],
        )?;
        Ok(())
    }

    pub fn delete_biz_account(&self, fakeid: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM biz_accounts WHERE fakeid = ?1",
            params![fakeid],
        )?;
        Ok(())
    }

    pub fn list_biz_accounts(&self) -> Result<Vec<BizAccountRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT a.fakeid, a.nickname, a.alias, a.round_head_img, a.signature,
                    (SELECT COUNT(*) FROM biz_articles b WHERE b.fakeid = a.fakeid) AS article_count
             FROM biz_accounts a ORDER BY a.added_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(BizAccountRow {
                fakeid: row.get(0)?,
                nickname: row.get(1)?,
                alias: row.get(2)?,
                round_head_img: row.get(3)?,
                signature: row.get(4)?,
                article_count: row.get(5)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// 批量入库（忽略重复），返回新插入条数
    pub fn upsert_biz_articles(&self, fakeid: &str, articles: &[BizArticleRow]) -> Result<usize> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let mut inserted = 0usize;
        {
            let mut stmt = tx.prepare(
                "INSERT OR IGNORE INTO biz_articles (aid, fakeid, title, link, digest, cover, author, publish_time)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )?;
            for a in articles {
                inserted += stmt.execute(params![
                    a.aid,
                    fakeid,
                    a.title,
                    a.link,
                    a.digest,
                    a.cover,
                    a.author,
                    a.publish_time
                ])?;
            }
        }
        tx.commit()?;
        Ok(inserted)
    }

    pub fn list_biz_articles(
        &self,
        fakeid: &str,
        keyword: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<BizArticleRow>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", keyword);
        let mut stmt = conn.prepare(
            "SELECT aid, title, link, digest, cover, publish_time FROM biz_articles
             WHERE fakeid = ?1 AND (?2 = '' OR title LIKE ?2)
             ORDER BY publish_time DESC LIMIT ?3 OFFSET ?4",
        )?;
        let rows = stmt.query_map(params![fakeid, pattern, limit, offset], |row| {
            Ok(BizArticleRow {
                aid: row.get(0)?,
                title: row.get(1)?,
                link: row.get(2)?,
                digest: row.get(3)?,
                cover: row.get(4)?,
                author: String::new(),
                publish_time: row.get(5)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn count_biz_articles(&self, fakeid: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM biz_articles WHERE fakeid = ?1",
            params![fakeid],
            |row| row.get(0),
        )
    }
}

#[cfg(test)]
mod database_test;
