//! Flow-controlled progress streaming, JSON Lines logs, and compact binary observation codec (`#205–208`).
use crate::{Result, error};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, mpsc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressFrame {
    pub case: String,
    pub sample: u64,
    pub total_samples: u64,
    pub ns_per_op: f64,
}

/// `#205` Flow-controlled bounded channel that never blocks producers; slow consumers drop oldest-excess frames.
pub struct BoundedProgressStream<T> {
    tx: mpsc::SyncSender<T>,
    dropped: Arc<AtomicU64>,
}

impl<T> Clone for BoundedProgressStream<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            dropped: Arc::clone(&self.dropped),
        }
    }
}

impl<T> BoundedProgressStream<T> {
    pub fn channel(capacity: usize) -> (Self, mpsc::Receiver<T>) {
        let (tx, rx) = mpsc::sync_channel(capacity.max(1));
        (
            Self {
                tx,
                dropped: Arc::new(AtomicU64::new(0)),
            },
            rx,
        )
    }

    pub fn try_publish(&self, value: T) -> bool {
        match self.tx.try_send(value) {
            Ok(()) => true,
            Err(mpsc::TrySendError::Full(_)) | Err(mpsc::TrySendError::Disconnected(_)) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }

    pub fn dropped_count(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

/// `#206` Append-only JSON Lines progress log.
pub struct JsonlProgressLog {
    file: std::fs::File,
}

impl JsonlProgressLog {
    pub fn open_append(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self { file })
    }

    pub fn append(&mut self, frame: &ProgressFrame) -> Result<()> {
        let line = serde_json::to_string(frame)?;
        writeln!(self.file, "{line}")?;
        self.file.flush()?;
        Ok(())
    }

    pub fn read_all(path: &Path) -> Result<Vec<ProgressFrame>> {
        let text = std::fs::read_to_string(path)?;
        let mut frames = Vec::new();
        for line in text.lines() {
            let t = line.trim();
            if !t.is_empty() {
                frames.push(serde_json::from_str(t)?);
            }
        }
        Ok(frames)
    }
}

/// `#207–208` Compact string-interned varint observation record for binary transport/storage.
#[derive(Debug, Clone, PartialEq)]
pub struct CompactObservation {
    pub case: String,
    pub metric: String,
    pub variant: String,
    pub process: u32,
    pub sequence: u64,
    pub operations: u64,
    pub value: f64,
}

const MAGIC: &[u8; 4] = b"ABOC";

fn write_varint(mut v: u64, out: &mut Vec<u8>) {
    loop {
        let mut byte = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if v == 0 {
            break;
        }
    }
}

fn read_varint(bytes: &[u8], pos: &mut usize) -> Result<u64> {
    let mut out = 0u64;
    let mut shift = 0u32;
    loop {
        let b = *bytes
            .get(*pos)
            .ok_or_else(|| error("unexpected EOF in compact observation varint"))?;
        *pos += 1;
        out |= u64::from(b & 0x7f)
            .checked_shl(shift)
            .ok_or_else(|| error("varint shift overflow"))?;
        if (b & 0x80) == 0 {
            return Ok(out);
        }
        shift += 7;
        if shift >= 64 {
            return Err(error("varint exceeds u64"));
        }
    }
}

/// `#208` Encode observations with deduplicated string table and varint integers.
pub fn encode_compact_observations(items: &[CompactObservation]) -> Vec<u8> {
    fn intern<'a>(
        dict: &mut Vec<&'a str>,
        map: &mut std::collections::BTreeMap<&'a str, usize>,
        s: &'a str,
    ) -> u64 {
        if let Some(&idx) = map.get(s) {
            idx as u64
        } else {
            let idx = dict.len();
            dict.push(s);
            map.insert(s, idx);
            idx as u64
        }
    }

    let mut dict: Vec<&str> = Vec::new();
    let mut map = std::collections::BTreeMap::<&str, usize>::new();

    let mut records = Vec::new();
    write_varint(items.len() as u64, &mut records);
    for item in items {
        write_varint(intern(&mut dict, &mut map, &item.case), &mut records);
        write_varint(intern(&mut dict, &mut map, &item.metric), &mut records);
        write_varint(intern(&mut dict, &mut map, &item.variant), &mut records);
        write_varint(u64::from(item.process), &mut records);
        write_varint(item.sequence, &mut records);
        write_varint(item.operations, &mut records);
        records.extend_from_slice(&item.value.to_le_bytes());
    }

    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    write_varint(dict.len() as u64, &mut out);
    for s in dict {
        write_varint(s.len() as u64, &mut out);
        out.extend_from_slice(s.as_bytes());
    }
    out.extend_from_slice(&records);
    out
}

/// Decode bytes produced by `encode_compact_observations`.
pub fn decode_compact_observations(bytes: &[u8]) -> Result<Vec<CompactObservation>> {
    if bytes.get(0..4) != Some(MAGIC) {
        return Err(error("invalid compact observation magic header"));
    }
    let mut pos = 4usize;
    let dict_len = read_varint(bytes, &mut pos)? as usize;
    let mut dict = Vec::with_capacity(dict_len.min(1024));
    for _ in 0..dict_len {
        let len = read_varint(bytes, &mut pos)? as usize;
        let end = pos
            .checked_add(len)
            .ok_or_else(|| error("string length overflow"))?;
        let slice = bytes
            .get(pos..end)
            .ok_or_else(|| error("truncated string table"))?;
        dict.push(std::str::from_utf8(slice)?.to_string());
        pos = end;
    }

    let count = read_varint(bytes, &mut pos)? as usize;
    let mut out = Vec::with_capacity(count.min(4096));
    let lookup = |idx: u64| -> Result<String> {
        dict.get(idx as usize)
            .cloned()
            .ok_or_else(|| error("compact string index out of range"))
    };
    for _ in 0..count {
        let case = lookup(read_varint(bytes, &mut pos)?)?;
        let metric = lookup(read_varint(bytes, &mut pos)?)?;
        let variant = lookup(read_varint(bytes, &mut pos)?)?;
        let process = u32::try_from(read_varint(bytes, &mut pos)?)
            .map_err(|_| error("process index exceeds u32"))?;
        let sequence = read_varint(bytes, &mut pos)?;
        let operations = read_varint(bytes, &mut pos)?;
        let end = pos + 8;
        let fbytes: [u8; 8] = bytes
            .get(pos..end)
            .ok_or_else(|| error("truncated float payload"))?
            .try_into()?;
        pos = end;
        out.push(CompactObservation {
            case,
            metric,
            variant,
            process,
            sequence,
            operations,
            value: f64::from_le_bytes(fbytes),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_stream_jsonl_and_compact_codec_roundtrip() {
        let (stream, rx) = BoundedProgressStream::channel(2);
        assert!(stream.try_publish(1));
        assert!(stream.try_publish(2));
        assert!(!stream.try_publish(3));
        assert_eq!(stream.dropped_count(), 1);
        assert_eq!(rx.recv().unwrap(), 1);
        assert_eq!(rx.recv().unwrap(), 2);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.jsonl");
        let frame = ProgressFrame {
            case: "sort/u64".into(),
            sample: 3,
            total_samples: 10,
            ns_per_op: 42.5,
        };
        {
            let mut log = JsonlProgressLog::open_append(&path).unwrap();
            log.append(&frame).unwrap();
        }
        assert_eq!(JsonlProgressLog::read_all(&path).unwrap(), vec![frame]);

        let rows = vec![
            CompactObservation {
                case: "case/a".into(),
                metric: "wall".into(),
                variant: "candidate".into(),
                process: 0,
                sequence: 1,
                operations: 1000,
                value: 12345.0,
            },
            CompactObservation {
                case: "case/a".into(),
                metric: "wall".into(),
                variant: "candidate".into(),
                process: 1,
                sequence: 2,
                operations: 1000,
                value: 12456.0,
            },
        ];
        let encoded = encode_compact_observations(&rows);
        let decoded = decode_compact_observations(&encoded).unwrap();
        assert_eq!(decoded, rows);
    }
}
