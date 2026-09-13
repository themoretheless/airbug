//! Issue store + ingest for airbug-err events (SQLite).
use crate::error::{HubError, Result};
use airbug_err::{EVENT_SCHEMA_VERSION, Event};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Supported ingest schema major (matches [`EVENT_SCHEMA_VERSION`]).
pub const SUPPORTED_EVENT_SCHEMA: u32 = EVENT_SCHEMA_VERSION;

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

/// Port for issue persistence (DIP).
pub trait IssueStore: Send + Sync {
    fn ingest(&self, event: Event, webhook: Option<&str>) -> Result<IngestResponse>;
    fn list(&self) -> IssuesList;
    fn get(&self, id: &str) -> Option<Issue>;
    fn set_status(&self, id: &str, status: IssueStatus) -> Result<Issue>;
}

/// SQLite-backed [`IssueStore`].
pub struct SqliteIssueStore {
    path: PathBuf,
    conn: Mutex<Connection>,
}

impl SqliteIssueStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let conn = open(&path)?;
        Ok(Self {
            path,
            conn: Mutex::new(conn),
        })
    }

    fn with_conn<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let guard = self
            .conn
            .lock()
            .map_err(|e| HubError::msg(format!("lock poisoned: {e}")))?;
        f(&guard)
    }
}

impl IssueStore for SqliteIssueStore {
    fn ingest(&self, event: Event, webhook: Option<&str>) -> Result<IngestResponse> {
        validate_schema(&event)?;
        self.with_conn(|conn| ingest_unlocked(conn, event, webhook))
    }

    fn list(&self) -> IssuesList {
        let path_s = self.path.display().to_string();
        match self.with_conn(list_unlocked) {
            Ok(issues) => IssuesList {
                path: path_s,
                available: true,
                note: if issues.is_empty() {
                    "No issues yet. Point airbug-err at POST /api/v1/errors.".into()
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

    fn get(&self, id: &str) -> Option<Issue> {
        self.with_conn(|conn| Ok(get_unlocked(conn, id)?)).ok()
    }

    fn set_status(&self, id: &str, status: IssueStatus) -> Result<Issue> {
        self.with_conn(|conn| {
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
}

pub fn validate_schema(event: &Event) -> Result<()> {
    if event.schema_version == 0 || event.schema_version > SUPPORTED_EVENT_SCHEMA {
        return Err(HubError::msg(format!(
            "unsupported event schema_version {} (supported major ≤ {SUPPORTED_EVENT_SCHEMA})",
            event.schema_version
        )));
    }
    Ok(())
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

fn fingerprint_key(event: &Event) -> String {
    if !event.fingerprint.is_empty() {
        return event.fingerprint.join("\u{1f}");
    }
    event.event_id.clone()
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

fn ingest_unlocked(
    conn: &Connection,
    event: Event,
    webhook: Option<&str>,
) -> Result<IngestResponse> {
    let fp = fingerprint_key(&event);
    let ts = if event.timestamp.is_empty() {
        now_iso()
    } else {
        event.timestamp.clone()
    };
    let title = event.title();
    let level = event.level.as_str().to_string();
    let release = event.release.clone();
    let environment = event.environment.clone();
    let service = event.service.clone();
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

fn list_unlocked(conn: &Connection) -> Result<Vec<IssueSummary>> {
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
    Ok(rows.filter_map(|r| r.ok()).collect())
}

fn fire_webhook(url: &str, issue_id: &str, issue: &Issue) {
    let url = url.to_string();
    let body = serde_json::json!({
        "text": format!("New airbug issue {issue_id}: {}", issue.title),
        "issue": issue,
    });
    std::thread::spawn(move || {
        if let Err(e) = post_json_webhook(&url, &body) {
            tracing::warn!(error = %e, "issues webhook failed");
        }
    });
}

fn post_json_webhook(url: &str, body: &Value) -> Result<()> {
    validate_webhook_url(url)?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| HubError::msg(e.to_string()))?;
    let response = client
        .post(url)
        .json(body)
        .send()
        .map_err(|e| HubError::msg(e.to_string()))?;
    if !response.status().is_success() {
        return Err(HubError::msg(format!("webhook HTTP {}", response.status())));
    }
    Ok(())
}

/// Webhooks: `http://` only; prefer loopback hosts.
pub fn validate_webhook_url(url: &str) -> Result<()> {
    if url.starts_with("https://") {
        return Err(HubError::msg(
            "webhook HTTPS is not supported; use http:// for local hooks",
        ));
    }
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| HubError::msg("webhook must be http://…"))?;
    let authority = rest.split_once('/').map(|(a, _)| a).unwrap_or(rest);
    let host = authority
        .rsplit_once(':')
        .map(|(h, _)| h)
        .unwrap_or(authority);
    if host.is_empty() {
        return Err(HubError::msg("webhook URL missing host"));
    }
    let loopback = host == "127.0.0.1" || host == "localhost" || host == "[::1]" || host == "::1";
    if !loopback {
        tracing::warn!(%host, "webhook host is not loopback; local-toolkit threat model assumes localhost");
    }
    Ok(())
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

    fn sample_event() -> Event {
        Event {
            schema_version: EVENT_SCHEMA_VERSION,
            event_id: "a".into(),
            timestamp: "t1".into(),
            level: airbug_err::Severity::Error,
            message: Some("boom".into()),
            release: None,
            environment: None,
            service: None,
            fingerprint: vec!["fp:1".into()],
            exception: Some(airbug_err::Exception {
                ty: "panic".into(),
                value: "boom".into(),
                stacktrace: None,
            }),
            breadcrumbs: vec![],
            tags: Default::default(),
            user: None,
            extra: Default::default(),
            contexts: Default::default(),
        }
    }

    #[test]
    fn ingest_groups_by_fingerprint() {
        let dir = env::temp_dir().join(format!("airbug-issues-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("dash/hub/data")).unwrap();
        let store =
            SqliteIssueStore::open(crate::config::RootPaths::new(&dir).issues_db()).unwrap();

        let event = sample_event();
        let r1 = store.ingest(event.clone(), None).unwrap();
        assert!(r1.is_new);
        let mut event2 = event;
        event2.event_id = "b".into();
        event2.timestamp = "t2".into();
        let r2 = store.ingest(event2, None).unwrap();
        assert!(!r2.is_new);
        assert_eq!(r2.count, 2);
        assert_eq!(r1.issue_id, r2.issue_id);
        let list = store.list();
        assert_eq!(list.issues.len(), 1);
        assert_eq!(list.issues[0].count, 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn webhook_rejects_https() {
        let err = validate_webhook_url("https://example.com/hook").unwrap_err();
        assert!(err.to_string().contains("HTTPS"));
    }

    #[test]
    fn rejects_unsupported_schema() {
        let mut event = sample_event();
        event.schema_version = 99;
        let err = validate_schema(&event).unwrap_err();
        assert!(err.to_string().contains("schema_version"));
    }
}
