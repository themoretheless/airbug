//! Issue store + ingest for airbug-err & Sentry events (SQLite).
//!
//! Includes auto-regression detection (`Resolved` / `ResolvedInNextRelease` -> `Unresolved`),
//! safe Source Context resolution (`±5` lines strictly within `paths.root`), git blame
//! `SuspectCommit`, tag distributions, 24-bucket activity sparklines, and Release Health sessions.
use crate::error::{HubError, Result};
use airbug_err::{EVENT_SCHEMA_VERSION, Event, SessionUpdate};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Supported ingest schema major (matches [`EVENT_SCHEMA_VERSION`]).
pub const SUPPORTED_EVENT_SCHEMA: u32 = EVENT_SCHEMA_VERSION;
/// Max occurrences retained per issue in SQLite.
pub const EVENT_STORE_CAP: i64 = 100;
/// Max occurrences returned on GET detail.
pub const EVENT_RETURN_CAP: i64 = 20;
const SCHEMA_VERSION: i64 = 3;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Unresolved,
    Resolved,
    ResolvedInNextRelease,
    Ignored,
}

impl IssueStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unresolved => "unresolved",
            Self::Resolved => "resolved",
            Self::ResolvedInNextRelease => "resolved_in_next_release",
            Self::Ignored => "ignored",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "resolved" => Self::Resolved,
            "resolved_in_next_release" => Self::ResolvedInNextRelease,
            "ignored" => Self::Ignored,
            _ => Self::Unresolved,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuspectCommit {
    pub commit: String,
    pub short: String,
    pub author: String,
    pub email: String,
    pub subject: String,
    pub timestamp: i64,
    pub file: String,
    pub lineno: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TagValueBucket {
    pub value: String,
    pub count: u64,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityBucket {
    pub index: usize,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReleaseHealthSummary {
    pub release: String,
    pub environment: Option<String>,
    pub total_sessions: u64,
    pub exited_sessions: u64,
    pub crashed_sessions: u64,
    pub abnormal_sessions: u64,
    pub crash_free_rate: f64,
    pub total_errors: u64,
    pub issue_count: u64,
    pub regression_count: u64,
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
    #[serde(default)]
    pub is_regression: bool,
    #[serde(default)]
    pub regression_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regressed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_in_release: Option<String>,
    pub last_event: Value,
    /// Recent occurrences (newest last), capped for API responses.
    #[serde(default)]
    pub events: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suspect_commit: Option<SuspectCommit>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tag_distributions: BTreeMap<String, Vec<TagValueBucket>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub activity_buckets: Vec<ActivityBucket>,
}

#[derive(Debug, Serialize)]
pub struct IngestResponse {
    pub ok: bool,
    pub issue_id: String,
    pub is_new: bool,
    pub is_regression: bool,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct IssuesList {
    pub path: String,
    pub available: bool,
    pub issues: Vec<IssueSummary>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
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
    #[serde(default)]
    pub is_regression: bool,
    #[serde(default)]
    pub regression_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
}

/// Port for issue persistence (DIP).
pub trait IssueStore: Send + Sync {
    fn ingest(&self, event: Event, webhook: Option<&str>) -> Result<IngestResponse>;
    fn list(&self) -> IssuesList;
    fn list_filtered(&self, run_id: Option<&str>) -> IssuesList {
        let mut list = self.list();
        let Some(rid) = run_id else {
            return list;
        };
        let _ = rid;
        list.note = format!("{} (run filter requires store support)", list.note);
        list
    }
    fn find_by_trace_id(&self, trace_id: &str) -> Vec<IssueSummary> {
        let _ = trace_id;
        Vec::new()
    }
    fn get(&self, id: &str) -> Option<Issue>;
    fn set_status(&self, id: &str, status: IssueStatus) -> Result<Issue>;
    fn record_session(&self, session: SessionUpdate) -> Result<()> {
        let _ = session;
        Ok(())
    }
    fn list_releases(&self) -> Vec<ReleaseHealthSummary> {
        Vec::new()
    }
}

/// SQLite-backed [`IssueStore`].
pub struct SqliteIssueStore {
    path: PathBuf,
    root: PathBuf,
    conn: Mutex<Connection>,
}

impl SqliteIssueStore {
    #[allow(dead_code)]
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        // Derive workspace root if path ends with `dash/hub/data/issues.sqlite`
        let root = path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        Self::open_with_root(path, root)
    }

    pub fn open_with_root(path: impl Into<PathBuf>, root: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let root = root.into();
        let conn = open(&path)?;
        Ok(Self {
            path,
            root,
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
                    "No issues yet. Point airbug-err at POST /api/v1/errors or Sentry SDK at DSN http://airbug@127.0.0.1:8790/1.".into()
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

    fn list_filtered(&self, run_id: Option<&str>) -> IssuesList {
        let path_s = self.path.display().to_string();
        match self.with_conn(|conn| list_unlocked_filtered(conn, run_id)) {
            Ok(issues) => IssuesList {
                path: path_s,
                available: true,
                note: if issues.is_empty() {
                    if run_id.is_some() {
                        "No issues for this airbug.run_id.".into()
                    } else {
                        "No issues yet. Point airbug-err at POST /api/v1/errors.".into()
                    }
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
        let mut issue = self.with_conn(|conn| Ok(get_unlocked(conn, id)?)).ok()?;
        enrich_issue_with_root(&self.root, &mut issue);
        Some(issue)
    }

    fn find_by_trace_id(&self, trace_id: &str) -> Vec<IssueSummary> {
        let wanted = trace_id.trim();
        if wanted.is_empty() {
            return Vec::new();
        }
        self.with_conn(|conn| find_by_trace_id_unlocked(conn, wanted))
            .unwrap_or_default()
    }

    fn set_status(&self, id: &str, status: IssueStatus) -> Result<Issue> {
        let mut issue = self.with_conn(|conn| {
            let current_rel: Option<String> = conn
                .query_row(
                    "SELECT release FROM issues WHERE id = ?1",
                    params![id],
                    |row| row.get(0),
                )
                .optional()?
                .flatten();
            let resolved_rel = if status == IssueStatus::ResolvedInNextRelease {
                current_rel
            } else {
                None
            };
            let clear_reg = matches!(
                status,
                IssueStatus::Resolved | IssueStatus::ResolvedInNextRelease | IssueStatus::Ignored
            );
            let n = if clear_reg {
                conn.execute(
                    "UPDATE issues SET status = ?1, is_regression = 0, resolved_in_release = ?2 WHERE id = ?3",
                    params![status.as_str(), resolved_rel, id],
                )?
            } else {
                conn.execute(
                    "UPDATE issues SET status = ?1, resolved_in_release = ?2 WHERE id = ?3",
                    params![status.as_str(), resolved_rel, id],
                )?
            };
            if n == 0 {
                return Err(HubError::msg(format!("issue not found: {id}")));
            }
            Ok(get_unlocked(conn, id)?)
        })?;
        enrich_issue_with_root(&self.root, &mut issue);
        Ok(issue)
    }

    fn record_session(&self, session: SessionUpdate) -> Result<()> {
        self.with_conn(|conn| record_session_unlocked(conn, &session))
    }

    fn list_releases(&self) -> Vec<ReleaseHealthSummary> {
        self.with_conn(list_releases_unlocked).unwrap_or_default()
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
    conn.busy_timeout(Duration::from_secs(5))?;
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
             last_event TEXT NOT NULL,
             is_regression INTEGER NOT NULL DEFAULT 0,
             regression_count INTEGER NOT NULL DEFAULT 0,
             regressed_at TEXT,
             resolved_in_release TEXT
         );
         CREATE INDEX IF NOT EXISTS idx_issues_last_seen ON issues(last_seen DESC);
         INSERT OR IGNORE INTO meta(key, value) VALUES('next_id', 0);
         INSERT OR IGNORE INTO meta(key, value) VALUES('schema_version', 1);",
    )?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let version: i64 = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1);
    if version < 2 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS issue_events (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 issue_id TEXT NOT NULL REFERENCES issues(id),
                 event_id TEXT NOT NULL,
                 ts TEXT NOT NULL,
                 payload TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_issue_events_issue_ts
                 ON issue_events(issue_id, id DESC);",
        )?;
        let mut stmt = conn.prepare(
            "SELECT id, last_seen, last_event FROM issues
             WHERE id NOT IN (SELECT DISTINCT issue_id FROM issue_events)",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        for row in rows {
            let (issue_id, ts, payload) = row?;
            let event_id = serde_json::from_str::<Value>(&payload)
                .ok()
                .and_then(|v| v.get("event_id")?.as_str().map(str::to_string))
                .unwrap_or_else(|| "backfill".into());
            conn.execute(
                "INSERT INTO issue_events(issue_id, event_id, ts, payload) VALUES(?1,?2,?3,?4)",
                params![issue_id, event_id, ts, payload],
            )?;
        }
    }
    if version < 3 {
        for alter in [
            "ALTER TABLE issues ADD COLUMN is_regression INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE issues ADD COLUMN regression_count INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE issues ADD COLUMN regressed_at TEXT",
            "ALTER TABLE issues ADD COLUMN resolved_in_release TEXT",
        ] {
            let _ = conn.execute(alter, []);
        }
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS release_sessions (
                 session_id TEXT PRIMARY KEY,
                 distinct_id TEXT,
                 release TEXT NOT NULL,
                 environment TEXT,
                 service TEXT,
                 status TEXT NOT NULL,
                 errors INTEGER NOT NULL DEFAULT 0,
                 started_at TEXT NOT NULL,
                 duration_ms INTEGER,
                 updated_at TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_release_sessions_rel ON release_sessions(release);",
        )?;
        conn.execute(
            "INSERT INTO meta(key, value) VALUES('schema_version', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![SCHEMA_VERSION],
        )?;
    }
    Ok(())
}

fn fingerprint_key(event: &Event) -> String {
    if !event.fingerprint.is_empty() {
        return event.fingerprint.join("\u{1f}");
    }
    event.event_id.clone()
}

pub(crate) fn now_iso() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let millis = dur.subsec_millis();
    let (y, mo, d, h, mi, s) = civil_from_days(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{millis:03}Z")
}

fn civil_from_days(secs: u64) -> (i32, u32, u32, u32, u32, u32) {
    let days = (secs / 86_400) as i64;
    let tod = (secs % 86_400) as u32;
    let h = tod / 3600;
    let mi = (tod % 3600) / 60;
    let s = tod % 60;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = (yoe as i64 + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let mo = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = if mo <= 2 { y + 1 } else { y };
    (y, mo, d, h, mi, s)
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
        is_regression: row.get::<_, i64>(12).unwrap_or(0) != 0,
        regression_count: row.get::<_, i64>(13).unwrap_or(0) as u64,
        regressed_at: row.get(14).ok().flatten(),
        resolved_in_release: row.get(15).ok().flatten(),
        events: Vec::new(),
        suspect_commit: None,
        tag_distributions: BTreeMap::new(),
        activity_buckets: Vec::new(),
    })
}

fn load_events(conn: &Connection, issue_id: &str) -> rusqlite::Result<Vec<Value>> {
    let mut stmt = conn.prepare(
        "SELECT payload FROM issue_events WHERE issue_id = ?1
         ORDER BY id DESC LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![issue_id, EVENT_RETURN_CAP], |row| {
        let raw: String = row.get(0)?;
        Ok(serde_json::from_str(&raw).unwrap_or(Value::Null))
    })?;
    let mut events: Vec<Value> = rows.filter_map(|r| r.ok()).collect();
    events.reverse(); // newest last
    Ok(events)
}

fn load_all_stored_events(conn: &Connection, issue_id: &str) -> rusqlite::Result<Vec<Value>> {
    let mut stmt = conn.prepare(
        "SELECT payload FROM issue_events WHERE issue_id = ?1
         ORDER BY id ASC",
    )?;
    let rows = stmt.query_map(params![issue_id], |row| {
        let raw: String = row.get(0)?;
        Ok(serde_json::from_str(&raw).unwrap_or(Value::Null))
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

fn record_occurrence(
    conn: &Connection,
    issue_id: &str,
    event: &Event,
    ts: &str,
    payload: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO issue_events(issue_id, event_id, ts, payload) VALUES(?1,?2,?3,?4)",
        params![issue_id, event.event_id, ts, payload],
    )?;
    conn.execute(
        "DELETE FROM issue_events WHERE issue_id = ?1 AND id NOT IN (
             SELECT id FROM issue_events WHERE issue_id = ?1 ORDER BY id DESC LIMIT ?2
         )",
        params![issue_id, EVENT_STORE_CAP],
    )?;
    Ok(())
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

    type ExistingIssueRow = (
        String,
        i64,
        String,
        i64,
        i64,
        Option<String>,
        Option<String>,
    );
    let existing: Option<ExistingIssueRow> = conn
        .query_row(
            "SELECT id, count, status, is_regression, regression_count, regressed_at, resolved_in_release
             FROM issues WHERE fingerprint = ?1",
            params![fp],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .optional()?;

    if let Some((id, count, status, was_reg, reg_count, prev_reg_at, resolved_in_rel)) = existing {
        let new_count = count + 1;
        let regressed_now = match status.as_str() {
            "resolved" => true,
            "resolved_in_next_release" => match (&release, &resolved_in_rel) {
                (Some(incoming_rel), Some(res_rel)) => incoming_rel != res_rel,
                (Some(_), None) => true,
                _ => false,
            },
            _ => false,
        };
        let new_status = if regressed_now {
            "unresolved"
        } else {
            status.as_str()
        };
        let is_regression = if regressed_now { 1 } else { was_reg };
        let regression_count = if regressed_now {
            reg_count + 1
        } else {
            reg_count
        };
        let regressed_at = if regressed_now {
            Some(ts.clone())
        } else {
            prev_reg_at
        };

        conn.execute(
            "UPDATE issues SET count=?1, last_seen=?2, title=?3, level=?4,
                 release=COALESCE(?5, release), environment=COALESCE(?6, environment),
                 service=COALESCE(?7, service), last_event=?8, status=?9,
                 is_regression=?10, regression_count=?11, regressed_at=?12
                 WHERE id=?13",
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
                is_regression,
                regression_count,
                regressed_at,
                id
            ],
        )?;
        record_occurrence(conn, &id, &event, &ts, &event_json)?;

        if regressed_now
            && let Some(url) = webhook
            && let Ok(issue) = get_unlocked(conn, &id)
        {
            fire_webhook(url, &id, &issue);
        }

        return Ok(IngestResponse {
            ok: true,
            issue_id: id,
            is_new: false,
            is_regression: is_regression != 0,
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
             release, environment, service, last_event, is_regression, regression_count)
             VALUES(?1,?2,?3,?4,'unresolved',1,?5,?5,?6,?7,?8,?9,0,0)",
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
    record_occurrence(conn, &id, &event, &ts, &event_json)?;

    if let Some(url) = webhook
        && let Ok(issue) = get_unlocked(conn, &id)
    {
        fire_webhook(url, &id, &issue);
    }

    Ok(IngestResponse {
        ok: true,
        issue_id: id,
        is_new: true,
        is_regression: false,
        count: 1,
    })
}

fn get_unlocked(conn: &Connection, id: &str) -> rusqlite::Result<Issue> {
    let mut issue = conn.query_row(
        "SELECT id, fingerprint, title, level, status, count, first_seen, last_seen,
                release, environment, service, last_event,
                is_regression, regression_count, regressed_at, resolved_in_release
         FROM issues WHERE id = ?1",
        params![id],
        map_issue_row,
    )?;
    issue.events = load_events(conn, id)?;
    let all_events = load_all_stored_events(conn, id).unwrap_or_else(|_| issue.events.clone());
    issue.tag_distributions = compute_tag_distributions(&all_events);
    issue.activity_buckets = compute_activity_buckets(&all_events, 24);
    Ok(issue)
}

fn enrich_issue_with_root(root: &Path, issue: &mut Issue) {
    resolve_source_context_on_event_json(root, &mut issue.last_event);
    for ev in &mut issue.events {
        resolve_source_context_on_event_json(root, ev);
    }
    issue.suspect_commit = resolve_suspect_commit(root, &issue.last_event);
}

/// Safely resolve `pre_context` (5 lines), `context_line`, `post_context` (5 lines) and `rel_path`
/// for stack frames whose file resides strictly within `root`.
pub fn resolve_source_context_on_event_json(root: &Path, event: &mut Value) {
    let Ok(canon_root) = root.canonicalize() else {
        return;
    };
    let Some(frames) = event
        .pointer_mut("/exception/stacktrace/frames")
        .and_then(Value::as_array_mut)
    else {
        return;
    };

    for frame in frames {
        let Some(filename) = frame.get("filename").and_then(Value::as_str) else {
            continue;
        };
        let Some(lineno) = frame.get("lineno").and_then(Value::as_u64) else {
            continue;
        };
        if lineno == 0 {
            continue;
        }
        let Some((canon_file, rel_path)) = safe_resolve_inside_root(&canon_root, filename) else {
            continue;
        };
        let Ok(content) = fs::read_to_string(&canon_file) else {
            continue;
        };
        let lines: Vec<&str> = content.lines().collect();
        let idx = (lineno as usize).saturating_sub(1);
        if idx >= lines.len() {
            continue;
        }
        let start = idx.saturating_sub(5);
        let end = (idx + 6).min(lines.len());
        let pre: Vec<String> = lines[start..idx].iter().map(|s| (*s).to_string()).collect();
        let ctx_line = lines[idx].to_string();
        let post: Vec<String> = lines[idx + 1..end]
            .iter()
            .map(|s| (*s).to_string())
            .collect();

        if let Some(obj) = frame.as_object_mut() {
            obj.insert("rel_path".into(), Value::String(rel_path));
            obj.insert("start_lineno".into(), json!(start + 1));
            obj.insert("pre_context".into(), json!(pre));
            obj.insert("context_line".into(), Value::String(ctx_line));
            obj.insert("post_context".into(), json!(post));
            obj.insert("in_app".into(), Value::Bool(true));
        }
    }
}

fn safe_resolve_inside_root(canon_root: &Path, raw_file: &str) -> Option<(PathBuf, String)> {
    let raw_trimmed = raw_file.trim();
    if raw_trimmed.is_empty() {
        return None;
    }
    let p = Path::new(raw_trimmed);
    let candidate = if p.is_absolute() {
        p.to_path_buf()
    } else {
        canon_root.join(p)
    };
    let canon = candidate.canonicalize().ok()?;
    if !canon.starts_with(canon_root) || !canon.is_file() {
        return None;
    }
    let rel = canon
        .strip_prefix(canon_root)
        .ok()?
        .display()
        .to_string()
        .replace('\\', "/");
    Some((canon, rel))
}

/// Resolve git blame / log suspect commit for the top in-app frame inside `root`.
pub fn resolve_suspect_commit(root: &Path, event: &Value) -> Option<SuspectCommit> {
    let canon_root = root.canonicalize().ok()?;
    let frames = event
        .pointer("/exception/stacktrace/frames")
        .and_then(Value::as_array)?;

    // Prefer first frame that resolved inside `root`
    let mut target: Option<(String, u32)> = None;
    for frame in frames {
        let Some(filename) = frame.get("filename").and_then(Value::as_str) else {
            continue;
        };
        let lineno = frame.get("lineno").and_then(Value::as_u64).unwrap_or(1) as u32;
        if let Some((_, rel)) = safe_resolve_inside_root(&canon_root, filename) {
            target = Some((rel, lineno.max(1)));
            break;
        }
    }
    let (rel_path, lineno) = target?;

    let range_arg = format!("-L{lineno},{lineno}:{rel_path}");
    let out = Command::new("git")
        .arg("-C")
        .arg(&canon_root)
        .args([
            "log",
            "-1",
            "--no-patch",
            "--format=%H|%h|%an|%ae|%s|%ct",
            &range_arg,
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success() && !o.stdout.is_empty())
        .or_else(|| {
            Command::new("git")
                .arg("-C")
                .arg(&canon_root)
                .args([
                    "log",
                    "-1",
                    "--format=%H|%h|%an|%ae|%s|%ct",
                    "--",
                    &rel_path,
                ])
                .stdin(Stdio::null())
                .stderr(Stdio::null())
                .output()
                .ok()
                .filter(|o| o.status.success() && !o.stdout.is_empty())
        })?;

    let first_line = String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();
    let mut parts = first_line.splitn(6, '|');
    let commit = parts.next()?.to_string();
    let short = parts.next()?.to_string();
    let author = parts.next()?.to_string();
    let email = parts.next()?.to_string();
    let subject = parts.next()?.to_string();
    let timestamp = parts.next()?.parse::<i64>().unwrap_or(0);

    Some(SuspectCommit {
        commit,
        short,
        author,
        email,
        subject,
        timestamp,
        file: rel_path,
        lineno,
    })
}

fn compute_tag_distributions(events: &[Value]) -> BTreeMap<String, Vec<TagValueBucket>> {
    let total = events.len().max(1) as f64;
    let mut counts: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();

    for ev in events {
        for field in ["environment", "release", "service", "level"] {
            if let Some(val) = ev.get(field).and_then(Value::as_str)
                && !val.is_empty()
            {
                *counts
                    .entry(field.to_string())
                    .or_default()
                    .entry(val.to_string())
                    .or_insert(0) += 1;
            }
        }
        if let Some(user) = ev.get("user").and_then(Value::as_object) {
            let u_label = user
                .get("email")
                .or_else(|| user.get("username"))
                .or_else(|| user.get("id"))
                .and_then(Value::as_str);
            if let Some(u) = u_label
                && !u.is_empty()
            {
                *counts
                    .entry("user".into())
                    .or_default()
                    .entry(u.to_string())
                    .or_insert(0) += 1;
            }
        }
        if let Some(tags) = ev.get("tags").and_then(Value::as_object) {
            for (k, v) in tags {
                if let Some(vs) = v.as_str()
                    && !vs.is_empty()
                {
                    *counts
                        .entry(format!("tag:{k}"))
                        .or_default()
                        .entry(vs.to_string())
                        .or_insert(0) += 1;
                }
            }
        }
    }

    counts
        .into_iter()
        .map(|(key, val_map)| {
            let mut list: Vec<TagValueBucket> = val_map
                .into_iter()
                .map(|(value, count)| TagValueBucket {
                    value,
                    count,
                    percent: ((count as f64 / total) * 1000.0).round() / 10.0,
                })
                .collect();
            list.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.value.cmp(&b.value)));
            (key, list)
        })
        .collect()
}

fn compute_activity_buckets(events: &[Value], num_buckets: usize) -> Vec<ActivityBucket> {
    let n = num_buckets.max(1);
    let mut counts = vec![0u64; n];
    if events.is_empty() {
        return (0..n)
            .map(|index| ActivityBucket { index, count: 0 })
            .collect();
    }
    let total = events.len();
    for (idx, _) in events.iter().enumerate() {
        let b = (idx * n / total).min(n - 1);
        counts[b] += 1;
    }
    counts
        .into_iter()
        .enumerate()
        .map(|(index, count)| ActivityBucket { index, count })
        .collect()
}

fn record_session_unlocked(conn: &Connection, session: &SessionUpdate) -> Result<()> {
    let now = now_iso();
    let started = if session.started_at.trim().is_empty() {
        now.clone()
    } else {
        session.started_at.clone()
    };
    let status = session.status.as_str();
    let dur = session.duration_ms.map(|d| d as i64);
    conn.execute(
        "INSERT INTO release_sessions(
             session_id, distinct_id, release, environment, service, status, errors, started_at, duration_ms, updated_at
         ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(session_id) DO UPDATE SET
             distinct_id = COALESCE(excluded.distinct_id, release_sessions.distinct_id),
             release = excluded.release,
             environment = COALESCE(excluded.environment, release_sessions.environment),
             service = COALESCE(excluded.service, release_sessions.service),
             status = excluded.status,
             errors = MAX(release_sessions.errors, excluded.errors),
             duration_ms = COALESCE(excluded.duration_ms, release_sessions.duration_ms),
             updated_at = excluded.updated_at",
        params![
            session.session_id,
            session.distinct_id,
            session.release,
            session.environment,
            session.service,
            status,
            session.errors as i64,
            started,
            dur,
            now
        ],
    )?;
    Ok(())
}

fn list_releases_unlocked(conn: &Connection) -> Result<Vec<ReleaseHealthSummary>> {
    #[derive(Default)]
    struct RelAgg {
        environment: Option<String>,
        total_sessions: u64,
        exited_sessions: u64,
        crashed_sessions: u64,
        abnormal_sessions: u64,
        session_errors: u64,
        issue_count: u64,
        issue_events: u64,
        regression_count: u64,
    }

    let mut by_release: BTreeMap<String, RelAgg> = BTreeMap::new();

    if let Ok(mut stmt) = conn.prepare(
        "SELECT release, environment, status, errors FROM release_sessions ORDER BY updated_at DESC",
    ) && let Ok(rows) = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i64>(3)?,
        ))
    }) {
        for row in rows.flatten() {
            let (rel, env, status, errs) = row;
            let agg = by_release.entry(rel).or_default();
            if agg.environment.is_none() && env.is_some() {
                agg.environment = env;
            }
            agg.total_sessions += 1;
            agg.session_errors += errs.max(0) as u64;
            match status.as_str() {
                "crashed" => agg.crashed_sessions += 1,
                "abnormal" => agg.abnormal_sessions += 1,
                "exited" => agg.exited_sessions += 1,
                _ => {}
            }
        }
    }

    if let Ok(mut stmt) = conn.prepare(
        "SELECT release, environment, count, is_regression FROM issues WHERE release IS NOT NULL",
    ) && let Ok(rows) = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, i64>(3)?,
        ))
    }) {
        for row in rows.flatten() {
            let (rel, env, cnt, is_reg) = row;
            if rel.trim().is_empty() {
                continue;
            }
            let agg = by_release.entry(rel).or_default();
            if agg.environment.is_none() && env.is_some() {
                agg.environment = env;
            }
            agg.issue_count += 1;
            agg.issue_events += cnt.max(0) as u64;
            if is_reg != 0 {
                agg.regression_count += 1;
            }
        }
    }

    let mut out: Vec<ReleaseHealthSummary> = by_release
        .into_iter()
        .map(|(release, a)| {
            let bad = a.crashed_sessions + a.abnormal_sessions;
            let crash_free_rate = if a.total_sessions > 0 {
                (((a.total_sessions.saturating_sub(bad) as f64) / (a.total_sessions as f64))
                    * 1000.0)
                    .round()
                    / 10.0
            } else {
                100.0
            };
            ReleaseHealthSummary {
                release,
                environment: a.environment,
                total_sessions: a.total_sessions,
                exited_sessions: a.exited_sessions,
                crashed_sessions: a.crashed_sessions,
                abnormal_sessions: a.abnormal_sessions,
                crash_free_rate,
                total_errors: a.session_errors.max(a.issue_events),
                issue_count: a.issue_count,
                regression_count: a.regression_count,
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.total_sessions
            .cmp(&a.total_sessions)
            .then_with(|| b.release.cmp(&a.release))
    });
    Ok(out)
}

fn list_unlocked(conn: &Connection) -> Result<Vec<IssueSummary>> {
    list_unlocked_filtered(conn, None)
}

fn payload_has_run_id(raw: &str, rid: &str) -> bool {
    let Ok(v) = serde_json::from_str::<Value>(raw) else {
        return false;
    };
    v.pointer("/tags/airbug.run_id")
        .and_then(|x| x.as_str())
        .or_else(|| {
            v.get("tags")
                .and_then(|t| t.get("airbug.run_id"))
                .and_then(|x| x.as_str())
        })
        == Some(rid)
}

fn issue_events_match_run_id(conn: &Connection, issue_id: &str, rid: &str) -> bool {
    let Ok(mut stmt) =
        conn.prepare("SELECT payload FROM issue_events WHERE issue_id = ?1 ORDER BY id DESC")
    else {
        return false;
    };
    let Ok(rows) = stmt.query_map(params![issue_id], |row| row.get::<_, String>(0)) else {
        return false;
    };
    for row in rows.flatten() {
        if payload_has_run_id(&row, rid) {
            return true;
        }
    }
    false
}

fn list_unlocked_filtered(conn: &Connection, run_id: Option<&str>) -> Result<Vec<IssueSummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, level, status, count, first_seen, last_seen, release, environment, service,
                last_event, is_regression, regression_count
         FROM issues ORDER BY last_seen DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let last_raw: String = row.get(10)?;
        let (trace_id, span_id) = serde_json::from_str::<Value>(&last_raw)
            .ok()
            .map(|v| crate::event_model::extract_trace_context_from_event_json(&v))
            .unwrap_or((None, None));
        Ok((
            IssueSummary {
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
                is_regression: row.get::<_, i64>(11).unwrap_or(0) != 0,
                regression_count: row.get::<_, i64>(12).unwrap_or(0) as u64,
                trace_id,
                span_id,
            },
            last_raw,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (summary, last_raw) = row?;
        if let Some(rid) = run_id {
            let matched = payload_has_run_id(&last_raw, rid)
                || issue_events_match_run_id(conn, &summary.id, rid);
            if !matched {
                continue;
            }
        }
        out.push(summary);
    }
    Ok(out)
}

fn find_matching_trace_in_payload(raw: &str, wanted: &str) -> Option<(String, Option<String>)> {
    let v = serde_json::from_str::<Value>(raw).ok()?;
    let (tid, sid) = crate::event_model::extract_trace_context_from_event_json(&v);
    let tid = tid?;
    tid.eq_ignore_ascii_case(wanted).then_some((tid, sid))
}

fn find_by_trace_id_unlocked(conn: &Connection, wanted: &str) -> Result<Vec<IssueSummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, level, status, count, first_seen, last_seen, release, environment, service,
                last_event, is_regression, regression_count
         FROM issues ORDER BY last_seen DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let last_raw: String = row.get(10)?;
        Ok((
            IssueSummary {
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
                is_regression: row.get::<_, i64>(11).unwrap_or(0) != 0,
                regression_count: row.get::<_, i64>(12).unwrap_or(0) as u64,
                trace_id: None,
                span_id: None,
            },
            last_raw,
        ))
    })?;

    let mut out = Vec::new();
    for row in rows {
        let (mut summary, last_raw) = row?;
        let mut matched = find_matching_trace_in_payload(&last_raw, wanted);
        if matched.is_none()
            && let Ok(mut ev_stmt) = conn
                .prepare("SELECT payload FROM issue_events WHERE issue_id = ?1 ORDER BY id DESC")
            && let Ok(ev_rows) = ev_stmt.query_map(params![summary.id], |r| r.get::<_, String>(0))
        {
            for payload in ev_rows.flatten() {
                if let Some(found) = find_matching_trace_in_payload(&payload, wanted) {
                    matched = Some(found);
                    break;
                }
            }
        }
        if let Some((tid, sid)) = matched {
            summary.trace_id = Some(tid);
            summary.span_id = sid;
            out.push(summary);
        }
    }
    Ok(out)
}

fn fire_webhook(url: &str, issue_id: &str, issue: &Issue) {
    let url = url.to_string();
    let prefix = if issue.is_regression {
        "Regressed airbug issue"
    } else {
        "New airbug issue"
    };
    let body = serde_json::json!({
        "text": format!("{prefix} {issue_id}: {}", issue.title),
        "is_regression": issue.is_regression,
        "issue": issue,
    });
    std::thread::spawn(move || {
        if let Err(e) = post_json_webhook(&url, &body) {
            tracing::warn!(error = %e, "issues webhook failed");
        }
    });
}

fn webhook_client() -> Result<&'static reqwest::blocking::Client> {
    static CLIENT: OnceLock<std::result::Result<reqwest::blocking::Client, String>> =
        OnceLock::new();
    let res = CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| e.to_string())
    });
    res.as_ref().map_err(|e| HubError::msg(e.clone()))
}

fn post_json_webhook(url: &str, body: &Value) -> Result<()> {
    validate_webhook_url(url)?;
    let client = webhook_client()?;
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
    let authority = rest.split(&['/', '?', '#'][..]).next().unwrap_or(rest);
    if authority.contains('@') {
        return Err(HubError::msg("webhook URL must not contain userinfo"));
    }
    let host = if let Some(after_open) = authority.strip_prefix('[') {
        let Some((ipv6, remainder)) = after_open.split_once(']') else {
            return Err(HubError::msg("invalid bracketed IPv6 host in webhook URL"));
        };
        if ipv6.is_empty() {
            return Err(HubError::msg("webhook URL missing host"));
        }
        if !remainder.is_empty() {
            let Some(port) = remainder.strip_prefix(':') else {
                return Err(HubError::msg("invalid authority in webhook URL"));
            };
            if port.parse::<u16>().is_err() {
                return Err(HubError::msg("invalid webhook port"));
            }
        }
        &authority[..=1 + ipv6.len()]
    } else {
        match authority.rsplit_once(':') {
            Some((h, p)) => {
                if h.contains(':') {
                    return Err(HubError::msg(
                        "IPv6 webhook host must be bracketed (e.g. [::1])",
                    ));
                }
                if p.parse::<u16>().is_err() {
                    return Err(HubError::msg("invalid webhook port"));
                }
                h
            }
            None => authority,
        }
    };
    if host.is_empty() {
        return Err(HubError::msg("webhook URL missing host"));
    }
    let loopback = host == "127.0.0.1" || host.eq_ignore_ascii_case("localhost") || host == "[::1]";
    if !loopback {
        tracing::warn!(%host, "webhook host is not loopback; local-toolkit threat model assumes localhost");
    }
    Ok(())
}

pub fn status_from_action(action: &str) -> Option<IssueStatus> {
    match action {
        "resolve" => Some(IssueStatus::Resolved),
        "resolve_next_release" => Some(IssueStatus::ResolvedInNextRelease),
        "ignore" => Some(IssueStatus::Ignored),
        "reopen" => Some(IssueStatus::Unresolved),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use airbug_err::SessionStatus;
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
            trace_id: None,
            span_id: None,
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
        let detail = store.get(&r1.issue_id).unwrap();
        assert_eq!(detail.events.len(), 2);
        assert_eq!(detail.events[1]["event_id"], "b");
        assert_eq!(detail.last_event["event_id"], "b");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn occurrence_cap_keeps_lifetime_count() {
        let dir = env::temp_dir().join(format!("airbug-issues-cap-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("dash/hub/data")).unwrap();
        let store =
            SqliteIssueStore::open(crate::config::RootPaths::new(&dir).issues_db()).unwrap();
        let base = sample_event();
        let mut last_id = String::new();
        for i in 0..(EVENT_STORE_CAP as usize + 5) {
            let mut event = base.clone();
            event.event_id = format!("e{i}");
            event.timestamp = format!("t{i}");
            last_id = store.ingest(event, None).unwrap().issue_id;
        }
        let detail = store.get(&last_id).unwrap();
        assert_eq!(detail.count, EVENT_STORE_CAP as u64 + 5);
        assert!(detail.events.len() <= EVENT_RETURN_CAP as usize);
        let stored: i64 = store
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM issue_events WHERE issue_id = ?1",
                    params![last_id],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(stored, EVENT_STORE_CAP);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn set_status_and_auto_regression_tracking() {
        let dir = env::temp_dir().join(format!("airbug-issues-status-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("dash/hub/data")).unwrap();
        let store =
            SqliteIssueStore::open(crate::config::RootPaths::new(&dir).issues_db()).unwrap();
        let mut ev = sample_event();
        ev.release = Some("1.0.0".into());
        let id = store.ingest(ev.clone(), None).unwrap().issue_id;
        assert_eq!(
            store.set_status(&id, IssueStatus::Resolved).unwrap().status,
            IssueStatus::Resolved
        );
        // Resolved auto-reopens as regression on ingest!
        let mut again = ev.clone();
        again.event_id = "reopen".into();
        let resp = store.ingest(again.clone(), None).unwrap();
        assert!(resp.is_regression);
        let issue = store.get(&id).unwrap();
        assert_eq!(issue.status, IssueStatus::Unresolved);
        assert!(issue.is_regression);
        assert_eq!(issue.regression_count, 1);

        // Resolve in next release: same release stays resolved, newer release regresses
        store
            .set_status(&id, IssueStatus::ResolvedInNextRelease)
            .unwrap();
        again.event_id = "same-rel".into();
        let resp_same = store.ingest(again.clone(), None).unwrap();
        assert!(!resp_same.is_regression);
        assert_eq!(
            store.get(&id).unwrap().status,
            IssueStatus::ResolvedInNextRelease
        );

        again.event_id = "new-rel".into();
        again.release = Some("1.1.0".into());
        let resp_new = store.ingest(again.clone(), None).unwrap();
        assert!(resp_new.is_regression);
        let issue_after = store.get(&id).unwrap();
        assert_eq!(issue_after.status, IssueStatus::Unresolved);
        assert!(issue_after.is_regression);
        assert_eq!(issue_after.regression_count, 2);

        // Ignored stays ignored
        store.set_status(&id, IssueStatus::Ignored).unwrap();
        again.event_id = "ignored".into();
        store.ingest(again, None).unwrap();
        assert_eq!(store.get(&id).unwrap().status, IssueStatus::Ignored);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn source_context_resolves_inside_root_and_rejects_outside_paths() {
        let dir = env::temp_dir().join(format!("airbug-issues-srcctx-{}", std::process::id()));
        let outside = env::temp_dir().join(format!("airbug-outside-{}.rs", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("dash/hub/data")).unwrap();

        let src_content = (1..=15)
            .map(|i| format!("let line_{i} = {i};"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(dir.join("src/main.rs"), &src_content).unwrap();
        fs::write(&outside, "secret_outside_line();\n").unwrap();

        let store = SqliteIssueStore::open_with_root(dir.join("dash/hub/data/issues.sqlite"), &dir)
            .unwrap();

        let mut ev = sample_event();
        ev.exception = Some(airbug_err::Exception {
            ty: "panic".into(),
            value: "crash".into(),
            stacktrace: Some(airbug_err::Stacktrace {
                frames: vec![
                    airbug_err::Frame {
                        function: "app::crash".into(),
                        filename: Some("src/main.rs".into()),
                        lineno: Some(8),
                        colno: None,
                        in_app: true,
                    },
                    airbug_err::Frame {
                        function: "evil::escape".into(),
                        filename: Some(outside.display().to_string()),
                        lineno: Some(1),
                        colno: None,
                        in_app: true,
                    },
                    airbug_err::Frame {
                        function: "evil::rel".into(),
                        filename: Some("../../../etc/passwd".into()),
                        lineno: Some(1),
                        colno: None,
                        in_app: true,
                    },
                ],
            }),
        });
        let id = store.ingest(ev, None).unwrap().issue_id;
        let issue = store.get(&id).unwrap();
        let frames = issue.last_event["exception"]["stacktrace"]["frames"]
            .as_array()
            .unwrap();
        assert_eq!(frames[0]["context_line"], "let line_8 = 8;");
        assert_eq!(frames[0]["pre_context"].as_array().unwrap().len(), 5);
        assert_eq!(frames[0]["post_context"].as_array().unwrap().len(), 5);
        assert_eq!(frames[0]["rel_path"], "src/main.rs");
        assert!(frames[1].get("context_line").is_none());
        assert!(frames[2].get("context_line").is_none());

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_file(&outside);
    }

    #[test]
    fn release_sessions_compute_crash_free_rate() {
        let dir = env::temp_dir().join(format!("airbug-issues-sessions-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("dash/hub/data")).unwrap();
        let store =
            SqliteIssueStore::open(crate::config::RootPaths::new(&dir).issues_db()).unwrap();

        for (i, st) in [
            SessionStatus::Exited,
            SessionStatus::Exited,
            SessionStatus::Ok,
            SessionStatus::Crashed,
        ]
        .into_iter()
        .enumerate()
        {
            store
                .record_session(SessionUpdate {
                    session_id: format!("s-{i}"),
                    distinct_id: Some("u1".into()),
                    release: "2.0.0".into(),
                    environment: Some("prod".into()),
                    service: Some("api".into()),
                    status: st,
                    errors: u32::from(st == SessionStatus::Crashed),
                    started_at: "2026-01-01T00:00:00Z".into(),
                    duration_ms: Some(1200),
                })
                .unwrap();
        }
        let releases = store.list_releases();
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].release, "2.0.0");
        assert_eq!(releases[0].total_sessions, 4);
        assert_eq!(releases[0].crashed_sessions, 1);
        assert!((releases[0].crash_free_rate - 75.0).abs() < 0.01);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn webhook_rejects_https() {
        let err = validate_webhook_url("https://example.com/hook").unwrap_err();
        assert!(err.to_string().contains("HTTPS"));
        assert!(validate_webhook_url("http://127.0.0.1:8080/hook?foo=bar#baz").is_ok());
        assert!(validate_webhook_url("http://[::1]:8080/hook").is_ok());
        assert!(validate_webhook_url("http://[::1]/hook").is_ok());
        assert!(validate_webhook_url("http://user@127.0.0.1/hook").is_err());
        assert!(validate_webhook_url("http://::1/hook").is_err());
    }

    #[test]
    fn run_id_filter_matches_earlier_occurrence() {
        let dir = env::temp_dir().join(format!("airbug-issues-runid-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("dash/hub/data")).unwrap();
        let store =
            SqliteIssueStore::open(crate::config::RootPaths::new(&dir).issues_db()).unwrap();

        let mut e1 = sample_event();
        e1.event_id = "e1".into();
        e1.tags.insert("airbug.run_id".into(), "run-1".into());
        store.ingest(e1, None).unwrap();

        let mut e2 = sample_event();
        e2.event_id = "e2".into();
        e2.tags.insert("airbug.run_id".into(), "run-2".into());
        store.ingest(e2, None).unwrap();

        assert_eq!(store.list_filtered(Some("run-1")).issues.len(), 1);
        assert_eq!(store.list_filtered(Some("run-2")).issues.len(), 1);
        assert_eq!(store.list_filtered(Some("run-3")).issues.len(), 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_unsupported_schema() {
        let mut event = sample_event();
        event.schema_version = 99;
        let err = validate_schema(&event).unwrap_err();
        assert!(err.to_string().contains("schema_version"));
    }

    #[test]
    fn find_by_trace_id_matches_occurrences() {
        let dir = env::temp_dir().join(format!("airbug-issues-trace-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("dash/hub/data")).unwrap();
        let store =
            SqliteIssueStore::open(crate::config::RootPaths::new(&dir).issues_db()).unwrap();

        let mut e1 = sample_event();
        e1.event_id = "ev-trace-1".into();
        e1.trace_id = Some("5b8efff798038103d269b633813fc60c".into());
        e1.span_id = Some("eee19b7ec3c1b174".into());
        store.ingest(e1, None).unwrap();

        let mut e2 = sample_event();
        e2.event_id = "ev-trace-2".into();
        e2.tags
            .insert("trace_id".into(), "11112222333344445555666677778888".into());
        store.ingest(e2, None).unwrap();

        let found = store.find_by_trace_id("5B8EFFF798038103D269B633813FC60C");
        assert_eq!(found.len(), 1);
        assert_eq!(
            found[0].trace_id.as_deref(),
            Some("5b8efff798038103d269b633813fc60c")
        );
        assert_eq!(found[0].span_id.as_deref(), Some("eee19b7ec3c1b174"));

        let found_tag = store.find_by_trace_id("11112222333344445555666677778888");
        assert_eq!(found_tag.len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }
}
