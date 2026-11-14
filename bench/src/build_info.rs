//! Build, compiler flag, lockfile, container/VM environment, and RNG seed provenance (`#121–128`).
use crate::convenience::Seeded;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildProvenance {
    pub pkg_version: String,
    pub profile: String,
    pub opt_level: String,
    pub target_arch: String,
    pub target_os: String,
    pub target_env: String,
    pub pointer_width: usize,
    pub rustflags: Option<String>,
    pub cargo_lock_sha256: Option<String>,
    pub container_or_vm: Option<String>,
    pub seed_env: Option<u64>,
}

impl BuildProvenance {
    /// Capture deterministic build & execution environment provenance metadata.
    pub fn capture(workspace_root: Option<&Path>) -> Self {
        let profile = if cfg!(debug_assertions) {
            "debug".to_string()
        } else {
            "release".to_string()
        };
        let opt_level = option_env!("OPT_LEVEL")
            .unwrap_or(if cfg!(debug_assertions) { "0" } else { "3" })
            .to_string();
        let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_else(|_| {
            #[cfg(target_env = "msvc")]
            {
                "msvc".into()
            }
            #[cfg(target_env = "gnu")]
            {
                "gnu".into()
            }
            #[cfg(not(any(target_env = "msvc", target_env = "gnu")))]
            {
                "unknown".into()
            }
        });
        let rustflags = std::env::var("RUSTFLAGS")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .or_else(|| {
                std::env::var("CARGO_ENCODED_RUSTFLAGS")
                    .ok()
                    .filter(|v| !v.trim().is_empty())
            });
        let cargo_lock_sha256 = workspace_root
            .map(|r| r.join("Cargo.lock"))
            .filter(|p| p.is_file())
            .and_then(|p| std::fs::read(p).ok())
            .map(|bytes| {
                let digest = Sha256::digest(&bytes);
                let mut hex = String::with_capacity(64);
                for b in digest.as_slice() {
                    use std::fmt::Write;
                    let _ = write!(hex, "{b:02x}");
                }
                hex
            });
        let container_or_vm = detect_container_or_vm();
        let seed_env = std::env::var("AIRBUG_BENCH_SEED")
            .ok()
            .and_then(|raw| Seeded::parse_seed_str(&raw));

        Self {
            pkg_version: env!("CARGO_PKG_VERSION").into(),
            profile,
            opt_level,
            target_arch: std::env::consts::ARCH.into(),
            target_os: std::env::consts::OS.into(),
            target_env,
            pointer_width: usize::BITS as usize,
            rustflags,
            cargo_lock_sha256,
            container_or_vm,
            seed_env,
        }
    }
}

fn detect_container_or_vm() -> Option<String> {
    if Path::new("/.dockerenv").exists() {
        return Some("docker".into());
    }
    if Path::new("/run/.containerenv").exists() {
        return Some("podman".into());
    }
    if std::env::var_os("KUBERNETES_SERVICE_HOST").is_some() {
        return Some("kubernetes".into());
    }
    if std::env::var_os("CI").is_some() {
        return Some("ci-runner".into());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_lock_hash_and_provenance() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.lock"), b"# lockfile fixture\n").unwrap();
        let p = BuildProvenance::capture(Some(dir.path()));
        assert!(!p.pkg_version.is_empty());
        assert_eq!(p.cargo_lock_sha256.as_ref().map(String::len), Some(64));
        assert!(p.pointer_width == 32 || p.pointer_width == 64);
    }
}
