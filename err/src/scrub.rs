//! Configurable PII & secret scrubbing engine for outgoing error events.

use crate::event::{Event, User};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Action applied when a [`ScrubRule`] matches a value, field, or substring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrubAction {
    /// Replace the matched value or substring with the given placeholder.
    Redact(String),
    /// Replace the matched value or substring with `[sha256:<16 hex chars>]`.
    Hash,
    /// Remove the matched map/header/field key or omit the matched substring.
    Drop,
}

impl ScrubAction {
    /// Convenience constructor for [`ScrubAction::Redact`] with `"[REDACTED]"`.
    pub fn redacted() -> Self {
        Self::Redact("[REDACTED]".into())
    }

    pub(crate) fn apply_str(&self, raw: &str) -> String {
        match self {
            Self::Redact(placeholder) => placeholder.clone(),
            Self::Hash => format!("[sha256:{}]", short_sha256(raw)),
            Self::Drop => String::new(),
        }
    }
}

/// Target selector for a [`ScrubRule`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrubTarget {
    /// Email addresses (`local@domain.tld`).
    Email,
    /// Bearer/Basic/Token credentials, JWTs (`eyJ...`), and common API key prefixes.
    SecretTokens,
    /// Luhn-valid payment card numbers (`13..=19` digits).
    CreditCard,
    /// IPv4 and IPv6 addresses.
    IpAddress,
    /// US-style `XXX-XX-XXXX` identifiers.
    Ssn,
    /// Case-insensitive key names matched on JSON objects, `tags`, `extra`,
    /// `contexts`, breadcrumb `data`, and inline `key=value` / `key: value` pairs.
    SensitiveKeys(Vec<String>),
    /// Literal substring (or `*` wildcard pattern) inside strings.
    Literal(String),
    /// Specific event field or dot-path (`"message"`, `"exception.value"`,
    /// `"user.email"`, `"tags.secret"`, `"extra.payload.token"`, `"contexts.*"`, `"breadcrumbs"`).
    Field(String),
}

/// Single scrubbing rule pairing a [`ScrubTarget`] with a [`ScrubAction`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrubRule {
    pub target: ScrubTarget,
    pub action: ScrubAction,
}

/// Rule-based PII scrubber applied to outgoing [`Event`] payloads.
#[derive(Debug, Clone, Default)]
pub struct Scrubber {
    pub rules: Vec<ScrubRule>,
    pub remove_user_pii: bool,
    pub scrub_urls_query: bool,
}

impl Scrubber {
    /// Create an empty scrubber with no built-in rules.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a scrubber preloaded with standard PII and credential rules:
    /// emails, secret tokens, credit cards, IP addresses, SSNs, default
    /// sensitive keys, URL query/userinfo scrubbing, and user PII redaction.
    pub fn default_pii() -> Self {
        Self {
            rules: vec![
                ScrubRule {
                    target: ScrubTarget::SensitiveKeys(default_sensitive_keys()),
                    action: ScrubAction::redacted(),
                },
                ScrubRule {
                    target: ScrubTarget::SecretTokens,
                    action: ScrubAction::redacted(),
                },
                ScrubRule {
                    target: ScrubTarget::CreditCard,
                    action: ScrubAction::redacted(),
                },
                ScrubRule {
                    target: ScrubTarget::Email,
                    action: ScrubAction::redacted(),
                },
                ScrubRule {
                    target: ScrubTarget::Ssn,
                    action: ScrubAction::redacted(),
                },
                ScrubRule {
                    target: ScrubTarget::IpAddress,
                    action: ScrubAction::redacted(),
                },
            ],
            remove_user_pii: true,
            scrub_urls_query: true,
        }
    }

    /// Append a rule.
    pub fn add_rule(mut self, rule: ScrubRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Append a target + action pair.
    pub fn rule(mut self, target: ScrubTarget, action: ScrubAction) -> Self {
        self.rules.push(ScrubRule { target, action });
        self
    }

    /// Redact a field or dot-path (`"message"`, `"tags.token"`, `"user.email"`, etc.).
    pub fn redact_field(self, path: impl Into<String>) -> Self {
        self.rule(ScrubTarget::Field(path.into()), ScrubAction::redacted())
    }

    /// Replace a field or dot-path value with a deterministic SHA-256 digest.
    pub fn hash_field(self, path: impl Into<String>) -> Self {
        self.rule(ScrubTarget::Field(path.into()), ScrubAction::Hash)
    }

    /// Drop a field or map key at the given dot-path.
    pub fn drop_field(self, path: impl Into<String>) -> Self {
        self.rule(ScrubTarget::Field(path.into()), ScrubAction::Drop)
    }

    /// Redact a literal substring or `*` wildcard pattern across event text.
    pub fn redact_literal(self, pattern: impl Into<String>) -> Self {
        self.rule(
            ScrubTarget::Literal(pattern.into()),
            ScrubAction::redacted(),
        )
    }

    /// Toggle scrubbing of URL userinfo (`user:pass@`) and query values (`?k=v`).
    pub fn scrub_urls(mut self, enabled: bool) -> Self {
        self.scrub_urls_query = enabled;
        self
    }

    /// Toggle automatic redaction of `event.user.email` and `event.user.username`.
    pub fn scrub_user(mut self, enabled: bool) -> Self {
        self.remove_user_pii = enabled;
        self
    }

    /// Whether no rules and no automatic flags are active.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty() && !self.remove_user_pii && !self.scrub_urls_query
    }

    /// Scrub a standalone string with URL and string-level rules.
    pub fn scrub_str(&self, input: &str) -> String {
        let mut out = if self.scrub_urls_query {
            scrub_urls_in_str(input)
        } else {
            input.to_string()
        };
        for rule in &self.rules {
            out = match &rule.target {
                ScrubTarget::Email => replace_emails(&out, &rule.action),
                ScrubTarget::SecretTokens => replace_secret_tokens(&out, &rule.action),
                ScrubTarget::CreditCard => replace_credit_cards(&out, &rule.action),
                ScrubTarget::IpAddress => replace_ips(&out, &rule.action),
                ScrubTarget::Ssn => replace_ssns(&out, &rule.action),
                ScrubTarget::SensitiveKeys(keys) => replace_inline_kv(&out, keys, &rule.action),
                ScrubTarget::Literal(pat) => replace_literal(&out, pat, &rule.action),
                ScrubTarget::Field(_) => out,
            };
        }
        out
    }

    /// Recursively scrub a [`serde_json::Value`].
    pub fn scrub_value(&self, val: &mut serde_json::Value) {
        self.scrub_value_with_path(val, "");
    }

    fn scrub_value_with_path(&self, val: &mut serde_json::Value, key_hint: &str) {
        if let Some(action) = self.sensitive_key_action(key_hint) {
            match action {
                ScrubAction::Drop => {
                    *val = serde_json::Value::Null;
                    return;
                }
                ScrubAction::Redact(placeholder) => {
                    redact_json_leaves(val, placeholder);
                    return;
                }
                ScrubAction::Hash => {
                    hash_json_leaves(val);
                    return;
                }
            }
        }
        match val {
            serde_json::Value::String(s) => {
                *s = self.scrub_str(s);
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.scrub_value_with_path(item, key_hint);
                }
            }
            serde_json::Value::Object(map) => {
                let mut to_remove = Vec::new();
                for (k, v) in map.iter_mut() {
                    if let Some(action) = self.sensitive_key_action(k)
                        && matches!(action, ScrubAction::Drop)
                    {
                        to_remove.push(k.clone());
                        continue;
                    }
                    self.scrub_value_with_path(v, k);
                }
                for k in to_remove {
                    map.remove(&k);
                }
            }
            _ => {}
        }
    }

    fn sensitive_key_action(&self, key: &str) -> Option<&ScrubAction> {
        if key.is_empty() {
            return None;
        }
        let lower = key.to_ascii_lowercase();
        for rule in &self.rules {
            if let ScrubTarget::SensitiveKeys(patterns) = &rule.target
                && patterns
                    .iter()
                    .any(|p| !p.is_empty() && lower.contains(&p.to_ascii_lowercase()))
            {
                return Some(&rule.action);
            }
        }
        None
    }

    /// Scrub an [`Event`] in place before transport/fingerprinting.
    pub fn scrub_event(&self, event: &mut Event) {
        if self.is_empty() {
            return;
        }

        // First apply explicit Field rules so targeted Drops/Hashes take precedence cleanly.
        for rule in &self.rules {
            if let ScrubTarget::Field(path) = &rule.target {
                apply_field_rule(event, path, &rule.action);
            }
        }

        if let Some(msg) = &mut event.message {
            *msg = self.scrub_str(msg);
        }

        if let Some(ex) = &mut event.exception {
            ex.value = self.scrub_str(&ex.value);
            if let Some(st) = &mut ex.stacktrace {
                for frame in &mut st.frames {
                    frame.function = self.scrub_str(&frame.function);
                    if let Some(file) = &mut frame.filename {
                        *file = self.scrub_str(file);
                    }
                }
            }
        }

        for crumb in &mut event.breadcrumbs {
            crumb.message = self.scrub_str(&crumb.message);
            self.scrub_string_keyed_value_map(&mut crumb.data);
        }

        self.scrub_string_map(&mut event.tags);
        self.scrub_string_keyed_value_map(&mut event.extra);
        self.scrub_string_keyed_value_map(&mut event.contexts);

        if let Some(user) = &mut event.user {
            self.scrub_user_struct(user);
        }
    }

    fn scrub_user_struct(&self, user: &mut User) {
        if self.remove_user_pii {
            if user.email.is_some() {
                user.email = Some("[REDACTED]".into());
            }
            if user.username.is_some() {
                user.username = Some("[REDACTED]".into());
            }
        }
        if let Some(id) = &mut user.id {
            *id = self.scrub_str(id);
        }
        if let Some(email) = &mut user.email
            && email != "[REDACTED]"
        {
            *email = self.scrub_str(email);
        }
        if let Some(username) = &mut user.username
            && username != "[REDACTED]"
        {
            *username = self.scrub_str(username);
        }
    }

    fn scrub_string_map(&self, map: &mut BTreeMap<String, String>) {
        let mut to_remove = Vec::new();
        for (k, v) in map.iter_mut() {
            if let Some(action) = self.sensitive_key_action(k) {
                match action {
                    ScrubAction::Drop => {
                        to_remove.push(k.clone());
                    }
                    other => {
                        *v = other.apply_str(v);
                    }
                }
            } else {
                *v = self.scrub_str(v);
            }
        }
        for k in to_remove {
            map.remove(&k);
        }
    }

    fn scrub_string_keyed_value_map(&self, map: &mut BTreeMap<String, serde_json::Value>) {
        let mut to_remove = Vec::new();
        for (k, v) in map.iter_mut() {
            if let Some(action) = self.sensitive_key_action(k)
                && matches!(action, ScrubAction::Drop)
            {
                to_remove.push(k.clone());
                continue;
            }
            self.scrub_value_with_path(v, k);
        }
        for k in to_remove {
            map.remove(&k);
        }
    }
}

fn default_sensitive_keys() -> Vec<String> {
    [
        "password",
        "passwd",
        "secret",
        "token",
        "api_key",
        "apikey",
        "authorization",
        "cookie",
        "set-cookie",
        "private_key",
        "access_token",
        "refresh_token",
        "client_secret",
        "session_id",
        "x-api-key",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn short_sha256(raw: &str) -> String {
    let digest = Sha256::digest(raw.as_bytes());
    let mut hex = String::with_capacity(16);
    for b in &digest[..8] {
        use std::fmt::Write;
        let _ = write!(hex, "{b:02x}");
    }
    hex
}

fn redact_json_leaves(val: &mut serde_json::Value, placeholder: &str) {
    match val {
        serde_json::Value::Null => {}
        serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => {
            *val = serde_json::Value::String(placeholder.to_string());
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                redact_json_leaves(item, placeholder);
            }
        }
        serde_json::Value::Object(map) => {
            for (_, v) in map.iter_mut() {
                redact_json_leaves(v, placeholder);
            }
        }
    }
}

fn hash_json_leaves(val: &mut serde_json::Value) {
    match val {
        serde_json::Value::Null => {}
        serde_json::Value::Bool(b) => {
            *val = serde_json::Value::String(format!("[sha256:{}]", short_sha256(&b.to_string())));
        }
        serde_json::Value::Number(n) => {
            *val = serde_json::Value::String(format!("[sha256:{}]", short_sha256(&n.to_string())));
        }
        serde_json::Value::String(s) => {
            *val = serde_json::Value::String(format!("[sha256:{}]", short_sha256(s)));
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                hash_json_leaves(item);
            }
        }
        serde_json::Value::Object(map) => {
            for (_, v) in map.iter_mut() {
                hash_json_leaves(v);
            }
        }
    }
}

fn apply_field_rule(event: &mut Event, path: &str, action: &ScrubAction) {
    match path {
        "message" => apply_opt_string(&mut event.message, action),
        "exception.value" => {
            if let Some(ex) = &mut event.exception {
                ex.value = action.apply_str(&ex.value);
            }
        }
        "user" => {
            if matches!(action, ScrubAction::Drop) {
                event.user = None;
            } else if let Some(u) = &mut event.user {
                apply_opt_string(&mut u.id, action);
                apply_opt_string(&mut u.email, action);
                apply_opt_string(&mut u.username, action);
            }
        }
        "user.id" => {
            if let Some(u) = &mut event.user {
                apply_opt_string(&mut u.id, action);
            }
        }
        "user.email" => {
            if let Some(u) = &mut event.user {
                apply_opt_string(&mut u.email, action);
            }
        }
        "user.username" => {
            if let Some(u) = &mut event.user {
                apply_opt_string(&mut u.username, action);
            }
        }
        "breadcrumbs" => {
            if matches!(action, ScrubAction::Drop) {
                event.breadcrumbs.clear();
            } else {
                for b in &mut event.breadcrumbs {
                    b.message = action.apply_str(&b.message);
                }
            }
        }
        p if p == "tags" || p == "tags.*" => {
            if matches!(action, ScrubAction::Drop) {
                event.tags.clear();
            } else {
                for v in event.tags.values_mut() {
                    *v = action.apply_str(v);
                }
            }
        }
        p if p.starts_with("tags.") => {
            let key = &p["tags.".len()..];
            if matches!(action, ScrubAction::Drop) {
                event.tags.remove(key);
            } else if let Some(v) = event.tags.get_mut(key) {
                *v = action.apply_str(v);
            }
        }
        p if p == "extra" || p == "extra.*" => {
            apply_value_map_all(&mut event.extra, action);
        }
        p if p.starts_with("extra.") => {
            let sub = &p["extra.".len()..];
            apply_value_map_path(&mut event.extra, sub, action);
        }
        p if p == "contexts" || p == "contexts.*" => {
            apply_value_map_all(&mut event.contexts, action);
        }
        p if p.starts_with("contexts.") => {
            let sub = &p["contexts.".len()..];
            apply_value_map_path(&mut event.contexts, sub, action);
        }
        _ => {}
    }
}

fn apply_opt_string(opt: &mut Option<String>, action: &ScrubAction) {
    match action {
        ScrubAction::Drop => *opt = None,
        other => {
            if let Some(s) = opt {
                *s = other.apply_str(s);
            }
        }
    }
}

fn apply_value_map_all(map: &mut BTreeMap<String, serde_json::Value>, action: &ScrubAction) {
    match action {
        ScrubAction::Drop => map.clear(),
        ScrubAction::Redact(placeholder) => {
            for v in map.values_mut() {
                redact_json_leaves(v, placeholder);
            }
        }
        ScrubAction::Hash => {
            for v in map.values_mut() {
                hash_json_leaves(v);
            }
        }
    }
}

fn apply_value_map_path(
    map: &mut BTreeMap<String, serde_json::Value>,
    path: &str,
    action: &ScrubAction,
) {
    let mut parts = path.splitn(2, '.');
    let head = parts.next().unwrap_or_default();
    let tail = parts.next();
    match tail {
        None => match action {
            ScrubAction::Drop => {
                map.remove(head);
            }
            ScrubAction::Redact(placeholder) => {
                if let Some(v) = map.get_mut(head) {
                    redact_json_leaves(v, placeholder);
                }
            }
            ScrubAction::Hash => {
                if let Some(v) = map.get_mut(head) {
                    hash_json_leaves(v);
                }
            }
        },
        Some(rest) => {
            if let Some(v) = map.get_mut(head) {
                apply_json_path(v, rest, action);
            }
        }
    }
}

fn apply_json_path(val: &mut serde_json::Value, path: &str, action: &ScrubAction) {
    let serde_json::Value::Object(map) = val else {
        return;
    };
    let mut parts = path.splitn(2, '.');
    let head = parts.next().unwrap_or_default();
    let tail = parts.next();
    match tail {
        None => {
            if head == "*" {
                match action {
                    ScrubAction::Drop => map.clear(),
                    ScrubAction::Redact(p) => {
                        for v in map.values_mut() {
                            redact_json_leaves(v, p);
                        }
                    }
                    ScrubAction::Hash => {
                        for v in map.values_mut() {
                            hash_json_leaves(v);
                        }
                    }
                }
            } else {
                match action {
                    ScrubAction::Drop => {
                        map.remove(head);
                    }
                    ScrubAction::Redact(p) => {
                        if let Some(v) = map.get_mut(head) {
                            redact_json_leaves(v, p);
                        }
                    }
                    ScrubAction::Hash => {
                        if let Some(v) = map.get_mut(head) {
                            hash_json_leaves(v);
                        }
                    }
                }
            }
        }
        Some(rest) => {
            if let Some(v) = map.get_mut(head) {
                apply_json_path(v, rest, action);
            }
        }
    }
}

fn scrub_urls_in_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        let scheme_len = if rest.starts_with("https://") {
            8
        } else if rest.starts_with("http://") {
            7
        } else {
            let ch = rest.chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
            continue;
        };

        let url_end_rel = rest
            .find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | ')'))
            .unwrap_or(rest.len());
        let url = &rest[..url_end_rel];
        let (scheme, after_scheme) = url.split_at(scheme_len);
        out.push_str(scheme);

        // Scrub optional userinfo@authority before query/fragment.
        let authority_end = after_scheme
            .find(['/', '?', '#'])
            .unwrap_or(after_scheme.len());
        let (authority, path_and_query) = after_scheme.split_at(authority_end);
        if let Some(at_pos) = authority.find('@') {
            out.push_str("[REDACTED]@");
            out.push_str(&authority[at_pos + 1..]);
        } else {
            out.push_str(authority);
        }

        if let Some(q_pos) = path_and_query.find('?') {
            let (path_part, q_and_frag) = path_and_query.split_at(q_pos);
            out.push_str(path_part);
            out.push('?');
            let (query, frag) = match q_and_frag[1..].find('#') {
                Some(f_pos) => (&q_and_frag[1..1 + f_pos], Some(&q_and_frag[1 + f_pos..])),
                None => (&q_and_frag[1..], None),
            };
            let mut first = true;
            for pair in query.split('&') {
                if !first {
                    out.push('&');
                }
                first = false;
                if let Some((k, _)) = pair.split_once('=') {
                    out.push_str(k);
                    out.push_str("=[REDACTED]");
                } else {
                    out.push_str("[REDACTED]");
                }
            }
            if let Some(f) = frag {
                out.push_str(f);
            }
        } else {
            out.push_str(path_and_query);
        }

        i += url_end_rel;
    }
    out
}

fn replace_emails(s: &str, action: &ScrubAction) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if let Some(end) = match_email_at(bytes, i) {
            out.push_str(&action.apply_str(&s[i..end]));
            i = end;
        } else {
            let ch = s[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

fn is_email_local_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'%' | b'+' | b'-')
}

fn is_email_domain_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-')
}

fn match_email_at(bytes: &[u8], start: usize) -> Option<usize> {
    if start > 0 && is_email_local_byte(bytes[start - 1]) {
        return None;
    }
    if !bytes[start].is_ascii_alphanumeric() {
        return None;
    }
    let mut p = start;
    while p < bytes.len() && is_email_local_byte(bytes[p]) {
        p += 1;
    }
    if p == start || p >= bytes.len() || bytes[p] != b'@' {
        return None;
    }
    p += 1;
    let domain_start = p;
    while p < bytes.len() && is_email_domain_byte(bytes[p]) {
        p += 1;
    }
    let domain = &bytes[domain_start..p];
    if domain.len() < 4 {
        return None;
    }
    let dot = domain.iter().rposition(|&b| b == b'.')?;
    let tld = &domain[dot + 1..];
    if tld.len() < 2 || !tld.iter().all(|b| b.is_ascii_alphabetic()) {
        return None;
    }
    if dot == 0 || domain[0] == b'-' || domain[dot - 1] == b'.' {
        return None;
    }
    Some(p)
}

fn replace_secret_tokens(s: &str, action: &ScrubAction) -> String {
    const AUTH_PREFIXES: &[&str] = &["Bearer ", "Basic ", "Token "];
    const KEY_PREFIXES: &[&str] = &[
        "sk_live_",
        "sk_test_",
        "pk_live_",
        "pk_test_",
        "rk_live_",
        "ghp_",
        "gho_",
        "ghu_",
        "ghs_",
        "github_pat_",
        "glpat-",
        "xoxb-",
        "xoxp-",
        "xoxa-",
        "AKIA",
        "ASIA",
        "AIza",
        "airbug_",
    ];

    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        let at_boundary = i == 0 || !is_token_char(s.as_bytes()[i - 1]);

        if at_boundary {
            let mut handled = false;
            for prefix in AUTH_PREFIXES {
                if rest.len() > prefix.len()
                    && rest
                        .get(..prefix.len())
                        .is_some_and(|p| p.eq_ignore_ascii_case(prefix))
                {
                    let val_start = i + prefix.len();
                    let val_end = scan_token_end(s.as_bytes(), val_start);
                    if val_end.saturating_sub(val_start) >= 6 {
                        out.push_str(&s[i..val_start]);
                        out.push_str(&action.apply_str(&s[val_start..val_end]));
                        i = val_end;
                        handled = true;
                        break;
                    }
                }
            }
            if handled {
                continue;
            }

            if rest.starts_with("eyJ") {
                let tok_end = scan_token_end(s.as_bytes(), i);
                let cand = &s[i..tok_end];
                if cand.split('.').count() == 3 && cand.len() >= 16 {
                    out.push_str(&action.apply_str(cand));
                    i = tok_end;
                    continue;
                }
            }

            for prefix in KEY_PREFIXES {
                if rest.starts_with(prefix) {
                    let tok_end = scan_token_end(s.as_bytes(), i);
                    if tok_end - i >= prefix.len() + 6 {
                        out.push_str(&action.apply_str(&s[i..tok_end]));
                        i = tok_end;
                        handled = true;
                        break;
                    }
                }
            }
            if handled {
                continue;
            }
        }

        let ch = rest.chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn is_token_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b'+' | b'/' | b'=')
}

fn scan_token_end(bytes: &[u8], start: usize) -> usize {
    let mut p = start;
    while p < bytes.len() && is_token_char(bytes[p]) {
        p += 1;
    }
    p
}

fn replace_credit_cards(s: &str, action: &ScrubAction) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit()
            && (i == 0 || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'-'))
            && let Some((end, digits)) = scan_card_candidate(bytes, i)
            && luhn_valid(&digits)
        {
            out.push_str(&action.apply_str(&s[i..end]));
            i = end;
            continue;
        }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn scan_card_candidate(bytes: &[u8], start: usize) -> Option<(usize, Vec<u8>)> {
    let mut p = start;
    let mut digits = Vec::with_capacity(20);
    while p < bytes.len() {
        let b = bytes[p];
        if b.is_ascii_digit() {
            digits.push(b - b'0');
            if digits.len() > 19 {
                return None;
            }
            p += 1;
        } else if matches!(b, b' ' | b'-') && p + 1 < bytes.len() && bytes[p + 1].is_ascii_digit() {
            p += 1;
        } else {
            break;
        }
    }
    if !(13..=19).contains(&digits.len()) {
        return None;
    }
    if p < bytes.len() && (bytes[p].is_ascii_alphanumeric() || bytes[p] == b'.') {
        return None;
    }
    Some((p, digits))
}

fn luhn_valid(digits: &[u8]) -> bool {
    let mut sum = 0u32;
    let mut double = false;
    for &d in digits.iter().rev() {
        let mut v = d as u32;
        if double {
            v *= 2;
            if v > 9 {
                v -= 9;
            }
        }
        sum += v;
        double = !double;
    }
    sum.is_multiple_of(10)
}

fn replace_ssns(s: &str, action: &ScrubAction) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if i + 11 <= bytes.len()
            && (i == 0 || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'-'))
            && bytes[i..i + 3].iter().all(|b| b.is_ascii_digit())
            && bytes[i + 3] == b'-'
            && bytes[i + 4..i + 6].iter().all(|b| b.is_ascii_digit())
            && bytes[i + 6] == b'-'
            && bytes[i + 7..i + 11].iter().all(|b| b.is_ascii_digit())
            && (i + 11 == bytes.len()
                || (!bytes[i + 11].is_ascii_alphanumeric() && bytes[i + 11] != b'-'))
            && valid_ssn(&s[i..i + 11])
        {
            out.push_str(&action.apply_str(&s[i..i + 11]));
            i += 11;
            continue;
        }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn valid_ssn(ssn: &str) -> bool {
    let area = &ssn[..3];
    let group = &ssn[4..6];
    let serial = &ssn[7..11];
    area != "000" && area != "666" && !area.starts_with('9') && group != "00" && serial != "0000"
}

fn replace_ips(s: &str, action: &ScrubAction) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if let Some(end) = match_ipv4_at(s, i) {
            out.push_str(&action.apply_str(&s[i..end]));
            i = end;
            continue;
        }
        if let Some(end) = match_ipv6_at(s, i) {
            out.push_str(&action.apply_str(&s[i..end]));
            i = end;
            continue;
        }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn match_ipv4_at(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    if !bytes[start].is_ascii_digit() {
        return None;
    }
    if start > 0
        && (bytes[start - 1].is_ascii_alphanumeric()
            || matches!(bytes[start - 1], b'.' | b'_' | b'-'))
    {
        return None;
    }
    let mut p = start;
    for part in 0..4 {
        let d_start = p;
        while p < bytes.len() && bytes[p].is_ascii_digit() && p - d_start < 3 {
            p += 1;
        }
        if p == d_start || p < bytes.len() && bytes[p].is_ascii_digit() {
            return None;
        }
        let octet: u8 = s[d_start..p].parse().ok()?;
        let _ = octet;
        if part < 3 {
            if p >= bytes.len() || bytes[p] != b'.' {
                return None;
            }
            p += 1;
        }
    }
    if p < bytes.len() && (bytes[p].is_ascii_alphanumeric() || matches!(bytes[p], b'.' | b'_')) {
        return None;
    }
    Some(p)
}

fn match_ipv6_at(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    if !(bytes[start].is_ascii_hexdigit() || bytes[start] == b':') {
        return None;
    }
    if start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b':') {
        return None;
    }
    let mut p = start;
    let mut colons = 0usize;
    let mut hex_digits = 0usize;
    while p < bytes.len() && (bytes[p].is_ascii_hexdigit() || bytes[p] == b':') {
        if bytes[p] == b':' {
            colons += 1;
        } else {
            hex_digits += 1;
        }
        p += 1;
    }
    if colons < 2 || hex_digits == 0 {
        return None;
    }
    if p < bytes.len() && (bytes[p].is_ascii_alphanumeric() || bytes[p] == b':') {
        return None;
    }
    s[start..p].parse::<std::net::Ipv6Addr>().ok()?;
    Some(p)
}

fn replace_inline_kv(s: &str, keys: &[String], action: &ScrubAction) -> String {
    let lower = s.to_ascii_lowercase();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let mut matched = None;
        let at_boundary = i == 0 || !s.as_bytes()[i - 1].is_ascii_alphanumeric();
        if at_boundary {
            for key in keys {
                let pat = key.to_ascii_lowercase();
                if pat.is_empty() || !lower[i..].starts_with(&pat) {
                    continue;
                }
                let mut after = i + pat.len();
                let bytes = s.as_bytes();
                while after < bytes.len() && (bytes[after] == b' ' || bytes[after] == b'\t') {
                    after += 1;
                }
                if after < bytes.len() && matches!(bytes[after], b'=' | b':') {
                    let mut val_start = after + 1;
                    while val_start < bytes.len()
                        && (bytes[val_start] == b' ' || bytes[val_start] == b'\t')
                    {
                        val_start += 1;
                    }
                    let val_end =
                        if val_start < bytes.len() && matches!(bytes[val_start], b'"' | b'\'') {
                            let quote = bytes[val_start];
                            let inner_start = val_start + 1;
                            let mut end = inner_start;
                            while end < bytes.len() && bytes[end] != quote {
                                end += 1;
                            }
                            if end > inner_start {
                                matched = Some((inner_start, end));
                                break;
                            }
                            continue;
                        } else {
                            let mut end = val_start;
                            while end < bytes.len()
                                && !bytes[end].is_ascii_whitespace()
                                && !matches!(bytes[end], b',' | b';' | b'&' | b')' | b']' | b'}')
                            {
                                end += 1;
                            }
                            end
                        };
                    if val_end > val_start {
                        matched = Some((val_start, val_end));
                        break;
                    }
                }
            }
        }
        if let Some((val_start, val_end)) = matched {
            out.push_str(&s[i..val_start]);
            out.push_str(&action.apply_str(&s[val_start..val_end]));
            i = val_end;
        } else {
            let ch = s[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

fn replace_literal(s: &str, pattern: &str, action: &ScrubAction) -> String {
    if pattern.is_empty() {
        return s.to_string();
    }
    if !pattern.contains('*') {
        let mut out = String::with_capacity(s.len());
        let mut rest = s;
        while let Some(pos) = rest.find(pattern) {
            out.push_str(&rest[..pos]);
            out.push_str(&action.apply_str(pattern));
            rest = &rest[pos + pattern.len()..];
        }
        out.push_str(rest);
        return out;
    }

    // Wildcard pattern inside a whitespace/punctuation-delimited segment.
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if let Some(end) = match_wildcard_at(s, i, &parts) {
            out.push_str(&action.apply_str(&s[i..end]));
            i = end.max(i + 1);
            continue;
        }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn match_wildcard_at(s: &str, start: usize, parts: &[&str]) -> Option<usize> {
    if parts.is_empty() {
        return None;
    }
    let first = parts[0];
    if !first.is_empty() && !s[start..].starts_with(first) {
        return None;
    }
    let mut cur = start + first.len();
    for (idx, part) in parts.iter().enumerate().skip(1) {
        if part.is_empty() {
            if idx + 1 == parts.len() {
                while cur < s.len() && !s.as_bytes()[cur].is_ascii_whitespace() {
                    cur += 1;
                }
            }
            continue;
        }
        let rel = s[cur..].find(part)?;
        if s[cur..cur + rel].contains(char::is_whitespace) {
            return None;
        }
        cur += rel + part.len();
    }
    Some(cur)
}
