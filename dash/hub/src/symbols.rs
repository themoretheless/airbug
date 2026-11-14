//! Debug-image symbol store and server-side address symbolication
//! supporting JSON symbol tables and Breakpad `.sym` files.

use crate::error::{HubError, Result};
use airbug_err::{DebugImage, Event};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolLine {
    pub offset: u64,
    pub size: u64,
    pub file: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolEntry {
    pub offset: u64,
    #[serde(default)]
    pub size: u64,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<SymbolLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DebugImageSymbolMap {
    pub debug_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    #[serde(default)]
    pub symbols: Vec<SymbolEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DebugImageSummary {
    pub debug_id: String,
    pub code_id: Option<String>,
    pub debug_file: Option<String>,
    pub arch: Option<String>,
    pub symbol_count: usize,
}

pub struct SymbolStore {
    dir: PathBuf,
    maps: RwLock<BTreeMap<String, DebugImageSymbolMap>>,
}

impl SymbolStore {
    /// Load any persisted `*.json` / `*.sym` debug symbol maps under `dir`.
    pub fn open(dir: PathBuf) -> Result<Self> {
        let mut maps = BTreeMap::new();
        if dir.is_dir()
            && let Ok(entries) = std::fs::read_dir(&dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json")
                    && let Ok(raw) = std::fs::read(&path)
                    && let Ok(mut map) = serde_json::from_slice::<DebugImageSymbolMap>(&raw)
                {
                    map.symbols.sort_by_key(|s| s.offset);
                    maps.insert(normalize_id(&map.debug_id), map);
                }
            }
        }
        Ok(Self {
            dir,
            maps: RwLock::new(maps),
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn list(&self) -> Vec<DebugImageSummary> {
        let guard = self.maps.read().unwrap_or_else(|e| e.into_inner());
        guard
            .values()
            .map(|m| DebugImageSummary {
                debug_id: m.debug_id.clone(),
                code_id: m.code_id.clone(),
                debug_file: m.debug_file.clone(),
                arch: m.arch.clone(),
                symbol_count: m.symbols.len(),
            })
            .collect()
    }

    /// Store or overwrite a JSON debug symbol map on disk and in memory.
    pub fn upload_json(&self, mut map: DebugImageSymbolMap) -> Result<DebugImageSummary> {
        if map.debug_id.trim().is_empty() {
            return Err(HubError::msg("debug_id must not be empty"));
        }
        map.symbols.sort_by_key(|s| s.offset);
        for sym in &mut map.symbols {
            sym.lines.sort_by_key(|l| l.offset);
        }
        std::fs::create_dir_all(&self.dir)?;
        let safe = sanitize_filename(&map.debug_id);
        let path = self.dir.join(format!("{safe}.json"));
        let json = serde_json::to_vec_pretty(&map)?;
        std::fs::write(&path, json)?;

        let summary = DebugImageSummary {
            debug_id: map.debug_id.clone(),
            code_id: map.code_id.clone(),
            debug_file: map.debug_file.clone(),
            arch: map.arch.clone(),
            symbol_count: map.symbols.len(),
        };
        let mut guard = self.maps.write().unwrap_or_else(|e| e.into_inner());
        guard.insert(normalize_id(&map.debug_id), map);
        Ok(summary)
    }

    /// Parse Breakpad `.sym` text and persist the resulting symbol table.
    pub fn upload_breakpad_sym(
        &self,
        debug_id_override: Option<&str>,
        text: &str,
    ) -> Result<DebugImageSummary> {
        let map = parse_breakpad_sym(debug_id_override, text)?;
        self.upload_json(map)
    }

    /// Resolve `instruction_addr` fields on frames using attached `debug_images`.
    /// Returns the number of frames updated with a resolved symbol.
    pub fn symbolicate_event(&self, event: &mut Event) -> usize {
        let Some(ex) = &mut event.exception else {
            return 0;
        };
        let Some(st) = &mut ex.stacktrace else {
            return 0;
        };
        let guard = self.maps.read().unwrap_or_else(|e| e.into_inner());
        if guard.is_empty() {
            return 0;
        }

        let mut resolved = 0usize;
        for frame in &mut st.frames {
            let Some(addr_str) = frame.instruction_addr.as_deref() else {
                continue;
            };
            let Some(ip) = parse_hex_or_dec(addr_str) else {
                continue;
            };

            let needs_symbol = frame.function.is_empty()
                || frame.function == "<unknown>"
                || frame.function == "<unsymbolicated>";
            if !needs_symbol {
                continue;
            }

            if let Some((img_base, map)) = find_matching_map(ip, &event.debug_images, &guard) {
                let rel = ip.saturating_sub(img_base);
                if let Some((sym, line_match)) = lookup_symbol(map, rel) {
                    frame.function = sym.name.clone();
                    if let Some((file, lineno)) = line_match {
                        frame.filename = Some(file);
                        frame.lineno = Some(lineno);
                    } else {
                        if frame.filename.is_none() {
                            frame.filename = sym.file.clone();
                        }
                        if frame.lineno.is_none() {
                            frame.lineno = sym.line;
                        }
                    }
                    frame.symbol_addr = Some(format!("{:#x}", img_base.saturating_add(sym.offset)));
                    frame.in_app = !frame.function.starts_with("std::")
                        && !frame.function.starts_with("core::");
                    resolved += 1;
                }
            }
        }
        resolved
    }
}

fn find_matching_map<'a>(
    ip: u64,
    images: &[DebugImage],
    maps: &'a BTreeMap<String, DebugImageSymbolMap>,
) -> Option<(u64, &'a DebugImageSymbolMap)> {
    // First try matching an image whose [image_addr, image_addr + image_size) contains `ip`.
    for img in images {
        let Some(base) = img.image_addr.as_deref().and_then(parse_hex_or_dec) else {
            continue;
        };
        let in_range = match img.image_size {
            Some(sz) if sz > 0 => ip >= base && ip < base.saturating_add(sz),
            _ => ip >= base,
        };
        if !in_range {
            continue;
        }
        if let Some(map) = match_image_in_store(img, maps) {
            return Some((base, map));
        }
    }

    // Fallback: match by debug_id / code_id / debug_file without address range check.
    for img in images {
        let base = img
            .image_addr
            .as_deref()
            .and_then(parse_hex_or_dec)
            .unwrap_or(0);
        if ip >= base
            && let Some(map) = match_image_in_store(img, maps)
        {
            return Some((base, map));
        }
    }

    // Single uploaded symbol table fallback when caller passed raw relative offsets.
    if images.is_empty() && maps.len() == 1 {
        let map = maps.values().next()?;
        return Some((0, map));
    }

    None
}

fn match_image_in_store<'a>(
    img: &DebugImage,
    maps: &'a BTreeMap<String, DebugImageSymbolMap>,
) -> Option<&'a DebugImageSymbolMap> {
    if let Some(did) = &img.debug_id
        && let Some(m) = maps.get(&normalize_id(did))
    {
        return Some(m);
    }
    if let Some(cid) = &img.code_id {
        let needle = normalize_id(cid);
        if let Some(m) = maps.values().find(|m| {
            m.code_id
                .as_deref()
                .is_some_and(|c| normalize_id(c) == needle)
        }) {
            return Some(m);
        }
    }
    if let Some(file) = &img.debug_file {
        let base_name = Path::new(file)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(file.as_str());
        if let Some(m) = maps.values().find(|m| {
            m.debug_file.as_deref().is_some_and(|df| {
                Path::new(df)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(df)
                    == base_name
            })
        }) {
            return Some(m);
        }
    }
    None
}

fn lookup_symbol(
    map: &DebugImageSymbolMap,
    offset: u64,
) -> Option<(&SymbolEntry, Option<(String, u32)>)> {
    if map.symbols.is_empty() {
        return None;
    }
    let idx = map.symbols.partition_point(|s| s.offset <= offset);
    let cand = &map.symbols[idx.checked_sub(1)?];
    if cand.size > 0 && offset >= cand.offset.saturating_add(cand.size) {
        return None;
    }
    let line_match = if cand.lines.is_empty() {
        None
    } else {
        let lidx = cand.lines.partition_point(|l| l.offset <= offset);
        lidx.checked_sub(1)
            .map(|i| &cand.lines[i])
            .filter(|l| l.size == 0 || offset < l.offset.saturating_add(l.size))
            .map(|l| (l.file.clone(), l.line))
    };
    Some((cand, line_match))
}

pub fn parse_breakpad_sym(
    debug_id_override: Option<&str>,
    text: &str,
) -> Result<DebugImageSymbolMap> {
    let mut debug_id = debug_id_override
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let mut debug_file = None;
    let mut arch = None;
    let mut files: BTreeMap<u32, String> = BTreeMap::new();
    let mut symbols: Vec<SymbolEntry> = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("MODULE ") {
            let parts: Vec<&str> = rest.splitn(4, ' ').collect();
            if parts.len() >= 4 {
                arch = Some(parts[1].trim().to_string());
                if debug_id.is_none() {
                    debug_id = Some(parts[2].trim().to_string());
                }
                debug_file = Some(parts[3].trim().to_string());
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("FILE ") {
            if let Some((num_str, path)) = rest.split_once(' ')
                && let Ok(num) = num_str.trim().parse::<u32>()
            {
                files.insert(num, path.trim().to_string());
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("FUNC ") {
            let mut parts = rest.split_whitespace();
            let first = parts.next().unwrap_or("");
            let offset_hex = if first == "m" {
                parts.next().unwrap_or("")
            } else {
                first
            };
            let size_hex = parts.next().unwrap_or("0");
            let _param_size = parts.next().unwrap_or("0");
            let name = parts.collect::<Vec<_>>().join(" ");
            if let (Ok(offset), Ok(size)) = (
                u64::from_str_radix(offset_hex.trim_start_matches("0x"), 16),
                u64::from_str_radix(size_hex.trim_start_matches("0x"), 16),
            ) {
                symbols.push(SymbolEntry {
                    offset,
                    size,
                    name: if name.is_empty() {
                        "<unknown>".into()
                    } else {
                        name
                    },
                    file: None,
                    line: None,
                    lines: Vec::new(),
                });
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("PUBLIC ") {
            let mut parts = rest.split_whitespace();
            let first = parts.next().unwrap_or("");
            let offset_hex = if first == "m" {
                parts.next().unwrap_or("")
            } else {
                first
            };
            let _param_size = parts.next().unwrap_or("0");
            let name = parts.collect::<Vec<_>>().join(" ");
            if let Ok(offset) = u64::from_str_radix(offset_hex.trim_start_matches("0x"), 16)
                && !name.is_empty()
            {
                symbols.push(SymbolEntry {
                    offset,
                    size: 0,
                    name,
                    file: None,
                    line: None,
                    lines: Vec::new(),
                });
            }
            continue;
        }
        // Line record attached to preceding FUNC: "<addr_hex> <size_hex> <line_dec> <file_id>"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 4
            && let (Ok(offset), Ok(size), Ok(lineno), Ok(file_id)) = (
                u64::from_str_radix(parts[0], 16),
                u64::from_str_radix(parts[1], 16),
                parts[2].parse::<u32>(),
                parts[3].parse::<u32>(),
            )
            && let Some(sym) = symbols.last_mut()
            && let Some(file) = files.get(&file_id)
        {
            if sym.file.is_none() {
                sym.file = Some(file.clone());
                sym.line = Some(lineno);
            }
            sym.lines.push(SymbolLine {
                offset,
                size,
                file: file.clone(),
                line: lineno,
            });
        }
    }

    let debug_id =
        debug_id.ok_or_else(|| HubError::msg("Breakpad .sym missing MODULE debug_id"))?;
    Ok(DebugImageSymbolMap {
        debug_id,
        code_id: None,
        debug_file,
        arch,
        symbols,
    })
}

pub fn parse_hex_or_dec(raw: &str) -> Option<u64> {
    let s = raw.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u64>().ok()
    }
}

fn normalize_id(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

fn sanitize_filename(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use airbug_err::{EVENT_SCHEMA_VERSION, Exception, Frame, Severity, Stacktrace};

    #[test]
    fn breakpad_upload_persists_reloads_and_symbolicates_frames() {
        let dir = std::env::temp_dir().join(format!("airbug-sym-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        {
            let store = SymbolStore::open(dir.clone()).unwrap();
            let breakpad = "\
MODULE Linux x86_64 ABC123DEF456 billing.bin
FILE 0 src/billing.rs
FUNC 1000 100 0 billing::process_payment
1040 10 42 0
";
            let summary = store.upload_breakpad_sym(None, breakpad).unwrap();
            assert_eq!(summary.debug_id, "ABC123DEF456");
            assert_eq!(summary.symbol_count, 1);
            assert_eq!(store.list().len(), 1);
        }

        let reopened = SymbolStore::open(dir.clone()).unwrap();
        let mut event = Event {
            schema_version: EVENT_SCHEMA_VERSION,
            event_id: "evt-1".into(),
            timestamp: "2026-11-14T12:00:00Z".into(),
            level: Severity::Error,
            message: None,
            release: None,
            environment: None,
            service: None,
            fingerprint: vec![],
            exception: Some(Exception {
                ty: "panic".into(),
                value: "boom".into(),
                stacktrace: Some(Stacktrace {
                    frames: vec![Frame {
                        function: "<unsymbolicated>".into(),
                        filename: None,
                        lineno: None,
                        colno: None,
                        in_app: false,
                        instruction_addr: Some("0x401040".into()),
                        symbol_addr: None,
                    }],
                }),
            }),
            breadcrumbs: vec![],
            tags: BTreeMap::new(),
            user: None,
            extra: BTreeMap::new(),
            contexts: BTreeMap::new(),
            trace_id: None,
            span_id: None,
            debug_images: vec![DebugImage {
                debug_id: Some("ABC123DEF456".into()),
                code_id: None,
                debug_file: Some("billing.bin".into()),
                image_addr: Some("0x400000".into()),
                image_size: Some(0x10000),
                arch: Some("x86_64".into()),
            }],
        };

        let resolved = reopened.symbolicate_event(&mut event);
        assert_eq!(resolved, 1);
        let frame = &event.exception.unwrap().stacktrace.unwrap().frames[0];
        assert_eq!(frame.function, "billing::process_payment");
        assert_eq!(frame.filename.as_deref(), Some("src/billing.rs"));
        assert_eq!(frame.lineno, Some(42));
        assert_eq!(frame.symbol_addr.as_deref(), Some("0x401000"));
        assert!(frame.in_app);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
