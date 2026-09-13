//! Issue store + ingest for airbug-err events (SQLite).
use crate::{config::RootPaths, error::{HubError, Result}};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    io::Write,
    net::TcpStream,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Unresolved,
    Resolved,
    Ignored,
}

impl IssueStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Unresolved => "unresolved",
            Self::Resolved => "resolved",
            Self::Ignored => "ignored",
        }
    }

    fn parse(s: &str) -> Self {
        match s {
            "resolved" => Self::Resolved,
            "ignored" => Self::Ignored,
            _ => Self::Unresolved,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub fingerprint: String,
    pub title: String,
    pub level: String,
    pub status: IssueStatus,
    pub count: u64,
    pub first_seen: String,
    pub last_seen: String,
    pub release: Option<String>,
    pub environment: Option<String>,
    pub service: Option<String>,
    pub last_event: Value,
}

#[derive(Debug, Serialize)]
pub struct IngestResponse {
    pub ok: bool,
    pub issue_id: String,
    pub is_new: bool,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct IssuesList {
    pub path: String,
    pub available: bool,
    pub issues: Vec<IssueSummary>,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct IssueSummary {
    pub id: String,
    pub title: String,
    pub level: String,
    pub status: IssueStatus,
    pub count: u64,
    pub first_seen: String,
    pub last_seen: String,
    pub release: Option<String>,
    pub environment: Option<String>,
    pub service: Option<String>,
}

struct DbState {
    path: PathBuf,
    conn: Connection,
}

static DB: Mutex<Option<DbState>> = Mutex::new(None);

pub fn db_path(root: &Path) -> PathBuf {
    RootPaths::new(root).issues_db()
}

fn with_db<T>(root: &Path, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
    let path = db_path(root);
    let mut guard = DB.lock().map_err(|e| HubError::msg(format!("lock poisoned: {e}")))?;
    let reopen = guard.as_ref().map(|s| s.path != path).unwrap_or(true);
    if reopen {
        *guard = Some(DbState {
            conn: open(&path)?,
            path,
        });
    }
    f(&guard.as_ref().expect("db just opened").conn)
}

fn open(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         CREATE TABLE IF NOT EXISTS meta(
             key TEXT PRIMARY KEY,
             value INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS issues(
             id TEXT PRIMARY KEY,
             fingerprint TEXT NOT NULL UNIQUE,
             title TEXT NOT NULL,
             level TEXT NOT NULL,
             status TEXT NOT NULL,
             count INTEGER NOT NULL,
             first_seen TEXT NOT NULL,
             last_seen TEXT NOT NULL,
             release TEXT,
             environment TEXT,
             service TEXT,
             last_event TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS idx_issues_last_seen ON issues(last_seen DESC);
         INSERT OR IGNORE INTO meta(key, value) VALUES('next_id', 0);",
    )?;
    Ok(conn)
}

fn fingerprint_key(event: &Value) -> String {
    if let Some(arr) = event.get("fingerprint").and_then(|v| v.as_array()) {
        let parts: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        if !parts.is_empty() {
            return parts.join("\u{1f}");
        }
    }
    event
        .get("event_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string()
}

fn event_title(event: &Value) -> String {
    if let Some(ex) = event.get("exception") {
        let ty = ex.get("type").and_then(|v| v.as_str()).unwrap_or("Error");
        let value = ex.get("value").and_then(|v| v.as_str()).unwrap_or("");
        if value.is_empty() {
            ty.to_string()
        } else {
            let short: String = value.chars().take(120).collect();
            format!("{ty}: {short}")
        }
    } else if let Some(msg) = event.get("message").and_then(|v| v.as_str()) {
        msg.chars().take(140).collect()
    } else {
        "untitled event".into()
    }
}

fn str_field(event: &Value, key: &str) -> Option<String> {
    event.get(key).and_then(|v| v.as_str()).map(str::to_string)
}

fn now_iso() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", dur.as_secs(), dur.subsec_millis())
}

fn map_issue_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Issue> {
    Ok(Issue {
        id: row.get(0)?,
        fingerprint: row.get(1)?,
        title: row.get(2)?,
        level: row.get(3)?,
        status: IssueStatus::parse(&row.get::<_, String>(4)?),
        count: row.get::<_, i64>(5)? as u64,
        first_seen: row.get(6)?,
        last_seen: row.get(7)?,
        release: row.get(8)?,
        environment: row.get(9)?,
        service: row.get(10)?,
        last_event: {
            let raw: String = row.get(11)?;
            serde_json::from_str(&raw).unwrap_or(Value::Null)
        },
    })
}

/// Ingest one error event JSON object.
pub fn ingest(root: &Path, event: Value, webhook: Option<&str>) -> Result<IngestResponse> {
    with_db(root, |conn| {
        let fp = fingerprint_key(&event);
        let ts = str_field(&event, "timestamp").unwrap_or_else(now_iso);
        let title = event_title(&event);
        let level = str_field(&event, "level").unwrap_or_else(|| "error".into());
        let release = str_field(&event, "release");
        let environment = str_field(&event, "environment");
        let service = str_field(&event, "service");
        let event_json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".into());

        let existing: Option<(String, i64, String)> = conn
            .query_row(
                "SELECT id, count, status FROM issues WHERE fingerprint = ?1",
                params![fp],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;

        if let Some((id, count, status)) = existing {
            let new_count = count + 1;
            let new_status = if status == "resolved" {
                "unresolved"
            } else {
                status.as_str()
            };
            conn.execute(
                "UPDATE issues SET count=?1, last_seen=?2, title=?3, level=?4,
                 release=?5, environment=?6, service=?7, last_event=?8, status=?9
                 WHERE id=?10",
                params![
                    new_count,
                    ts,
                    title,
                    level,
                    release,
                    environment,
                    service,
                    event_json,
                    new_status,
                    id
                ],
            )?;
            return Ok(IngestResponse {
                ok: true,
                issue_id: id,
                is_new: false,
                count: new_count as u64,
            });
        }

        let next_id: i64 = conn
            .query_row(
                "UPDATE meta SET value = value + 1 WHERE key = 'next_id' RETURNING value",
                [],
                |row| row.get(0),
            )
            .or_else(|_| {
                conn.execute(
                    "UPDATE meta SET value = value + 1 WHERE key = 'next_id'",
                    [],
                )?;
                conn.query_row("SELECT value FROM meta WHERE key = 'next_id'", [], |row| {
                    row.get(0)
                })
            })?;

        let id = format!("ISSUE-{next_id}");
        conn.execute(
            "INSERT INTO issues(id, fingerprint, title, level, status, count, first_seen, last_seen,
             release, environment, service, last_event)
             VALUES(?1,?2,?3,?4,'unresolved',1,?5,?5,?6,?7,?8,?9)",
            params![
                id,
                fp,
                title,
                level,
                ts,
                release,
                environment,
                service,
                event_json
            ],
        )?;

        if let Some(url) = webhook {
            let issue = get_unlocked(conn, &id).ok();
            if let Some(issue) = issue {
                fire_webhook(url, &id, &issue);
            }
        }

        Ok(IngestResponse {
            ok: true,
            issue_id: id,
            is_new: true,
            count: 1,
        })
    })
}

fn get_unlocked(conn: &Connection, id: &str) -> rusqlite::Result<Issue> {
    conn.query_row(
        "SELECT id, fingerprint, title, level, status, count, first_seen, last_seen,
                release, environment, service, last_event
         FROM issues WHERE id = ?1",
        params![id],
        map_issue_row,
    )
}

fn fire_webhook(url: &str, issue_id: &str, issue: &Issue) {
    let url = url.to_string();
    let body = serde_json::json!({
        "text": format!("New airbug issue {issue_id}: {}", issue.title),
        "issue": issue,
    });
    std::thread::spawn(move || {
        if let Err(e) = post_json(&url, &body) {
            eprintln!("issues webhook: {e}");
        }
    });
}

fn post_json(url: &str, body: &Value) -> Result<()> {
    let (host, port, path) = parse_http_url(url)?;
    let payload = serde_json::to_vec(body)?;
    let mut stream = TcpStream::connect((host.as_str(), port))
        .map_err(|e| HubError::msg(format!("connect {host}:{port}: {e}")))?;
    let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(5)));
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    stream.write_all(req.as_bytes())?;
    stream.write_all(&payload)?;
    Ok(())
}

fn parse_http_url(url: &str) -> Result<(String, u16, String)> {
    if url.starts_with("https://") {
        return Err(HubError::msg(
            "webhook HTTPS is not supported (plain TCP only); use http:// for local hooks",
        ));
    }
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| HubError::msg("webhook must be http://…"))?;
    let (authority, path) = match rest.split_once('/') {
        Some((a, p)) => (a, format!("/{p}")),
        None => (rest, "/".into()),
    };
    let (host, port) = if let Some((h, p)) = authority.rsplit_once(':') {
        (
            h.to_string(),
            p.parse::<u16>()
                .map_err(|_| HubError::msg(format!("bad webhook port: {p}")))?,
        )
    } else {
        (authority.to_string(), 80)
    };
    if host.is_empty() {
        return Err(HubError::msg("webhook URL missing host"));
    }
    Ok((host, port, path))
}

pub fn list(root: &Path) -> IssuesList {
    let path = db_path(root);
    let path_s = path.display().to_string();
    match with_db(root, |conn| {
        let mut stmt = conn.prepare(
            "SELECT id, title, level, status, count, first_seen, last_seen, release, environment, service
             FROM issues ORDER BY last_seen DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(IssueSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                level: row.get(2)?,
                status: IssueStatus::parse(&row.get::<_, String>(3)?),
                count: row.get::<_, i64>(4)? as u64,
                first_seen: row.get(5)?,
                last_seen: row.get(6)?,
                release: row.get(7)?,
                environment: row.get(8)?,
                service: row.get(9)?,
            })
        })?;
        let issues: Vec<IssueSummary> = rows.filter_map(|r| r.ok()).collect();
        Ok(issues)
    }) {
        Ok(issues) => IssuesList {
            path: path_s,
            available: true,
            note: if issues.is_empty() {
                "No issues yet. Point airbug-err at POST /api/errors.".into()
            } else {
                format!("{} issue(s)", issues.len())
            },
            issues,
        },
        Err(e) => IssuesList {
            path: path_s,
            available: false,
            issues: vec![],
            note: e.to_string(),
        },
    }
}

pub fn get(root: &Path, id: &str) -> Option<Issue> {
    with_db(root, |conn| Ok(get_unlocked(conn, id)?)).ok()
}

pub fn set_status(root: &Path, id: &str, status: IssueStatus) -> Result<Issue> {
    with_db(root, |conn| {
        let n = conn.execute(
            "UPDATE issues SET status = ?1 WHERE id = ?2",
            params![status.as_str(), id],
        )?;
        if n == 0 {
            return Err(HubError::msg(format!("issue not found: {id}")));
        }
        Ok(get_unlocked(conn, id)?)
    })
}

pub fn status_from_action(action: &str) -> Option<IssueStatus> {
    match action {
        "resolve" => Some(IssueStatus::Resolved),
        "ignore" => Some(IssueStatus::Ignored),
        "reopen" => Some(IssueStatus::Unresolved),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn ingest_groups_by_fingerprint() {
        let dir = env::temp_dir().join(format!("airbug-issues-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("dash/hub/data")).unwrap();

        let event = serde_json::json!({
            "event_id": "a",
            "timestamp": "t1",
            "level": "error",
            "message": "boom",
            "fingerprint": ["fp:1"],
            "exception": { "type": "panic", "value": "boom" }
        });
        let r1 = ingest(&dir, event.clone(), None).unwrap();
        assert!(r1.is_new);
        let mut event2 = event;
        event2["event_id"] = Value::String("b".into());
        event2["timestamp"] = Value::String("t2".into());
        let r2 = ingest(&dir, event2, None).unwrap();
        assert!(!r2.is_new);
        assert_eq!(r2.count, 2);
        assert_eq!(r1.issue_id, r2.issue_id);
        let list = list(&dir);
        assert_eq!(list.issues.len(), 1);
        assert_eq!(list.issues[0].count, 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn webhook_rejects_https() {
        let err = parse_http_url("https://example.com/hook").unwrap_err();
        assert!(err.to_string().contains("HTTPS"));
    }
}
