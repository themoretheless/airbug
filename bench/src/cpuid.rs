//! Hardware CPUID / processor signature fingerprinting (`#119`).
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CpuSignature {
    pub arch: String,
    pub os: String,
    pub vendor: String,
    pub brand: String,
    pub family: u32,
    pub model: u32,
    pub stepping: u32,
    pub logical_cores: usize,
    pub cache_line_bytes: u32,
    pub features: Vec<String>,
    /// Stable SHA-256 hex digest over normalized processor identity & feature set.
    pub fingerprint: String,
}

impl CpuSignature {
    pub fn detect() -> Self {
        let logical_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let (vendor, brand, family, model, stepping, cache_line_bytes, features) =
            detect_platform();
        let arch = std::env::consts::ARCH.to_string();
        let os = std::env::consts::OS.to_string();

        let mut h = Sha256::new();
        h.update(arch.as_bytes());
        h.update(b"|");
        h.update(vendor.as_bytes());
        h.update(b"|");
        h.update(brand.as_bytes());
        h.update(format!("|{family}|{model}|{stepping}|{cache_line_bytes}|").as_bytes());
        for feat in &features {
            h.update(feat.as_bytes());
            h.update(b",");
        }
        let digest = h.finalize();
        let mut fingerprint = String::with_capacity(64);
        for b in digest.as_slice() {
            use std::fmt::Write;
            let _ = write!(fingerprint, "{b:02x}");
        }

        Self {
            arch,
            os,
            vendor,
            brand,
            family,
            model,
            stepping,
            logical_cores,
            cache_line_bytes,
            features,
            fingerprint,
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn detect_platform() -> (String, String, u32, u32, u32, u32, Vec<String>) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::__cpuid_count;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::__cpuid_count;

    // CPUID leaf 0 is supported on all x86/x86_64 processors running Rust std.
    let leaf0 = __cpuid_count(0, 0);
    let mut vendor_bytes = [0u8; 12];
    vendor_bytes[0..4].copy_from_slice(&leaf0.ebx.to_le_bytes());
    vendor_bytes[4..8].copy_from_slice(&leaf0.edx.to_le_bytes());
    vendor_bytes[8..12].copy_from_slice(&leaf0.ecx.to_le_bytes());
    let vendor = String::from_utf8_lossy(&vendor_bytes).trim().to_string();

    let (mut family, mut model, mut stepping, mut cache_line) = (0, 0, 0, 64);
    if leaf0.eax >= 1 {
        let leaf1 = __cpuid_count(1, 0);
        let base_family = (leaf1.eax >> 8) & 0xf;
        let ext_family = (leaf1.eax >> 20) & 0xff;
        let base_model = (leaf1.eax >> 4) & 0xf;
        let ext_model = (leaf1.eax >> 16) & 0xf;
        family = if base_family == 0xf {
            base_family + ext_family
        } else {
            base_family
        };
        model = if base_family == 6 || base_family == 0xf {
            (ext_model << 4) | base_model
        } else {
            base_model
        };
        stepping = leaf1.eax & 0xf;
        let clflush = ((leaf1.ebx >> 8) & 0xff) * 8;
        if clflush > 0 {
            cache_line = clflush;
        }
    }

    let ext0 = __cpuid_count(0x8000_0000, 0);
    let brand = if ext0.eax >= 0x8000_0004 {
        let mut buf = [0u8; 48];
        for (i, leaf) in [0x8000_0002u32, 0x8000_0003, 0x8000_0004]
            .into_iter()
            .enumerate()
        {
            let r = __cpuid_count(leaf, 0);
            let off = i * 16;
            buf[off..off + 4].copy_from_slice(&r.eax.to_le_bytes());
            buf[off + 4..off + 8].copy_from_slice(&r.ebx.to_le_bytes());
            buf[off + 8..off + 12].copy_from_slice(&r.ecx.to_le_bytes());
            buf[off + 12..off + 16].copy_from_slice(&r.edx.to_le_bytes());
        }
        String::from_utf8_lossy(&buf)
            .trim_matches(char::from(0))
            .trim()
            .to_string()
    } else {
        "unknown".into()
    };

    let mut features = Vec::new();
    for (name, present) in [
        ("sse2", std::arch::is_x86_feature_detected!("sse2")),
        ("sse4.2", std::arch::is_x86_feature_detected!("sse4.2")),
        ("aes", std::arch::is_x86_feature_detected!("aes")),
        ("avx", std::arch::is_x86_feature_detected!("avx")),
        ("avx2", std::arch::is_x86_feature_detected!("avx2")),
        ("fma", std::arch::is_x86_feature_detected!("fma")),
        ("bmi1", std::arch::is_x86_feature_detected!("bmi1")),
        ("bmi2", std::arch::is_x86_feature_detected!("bmi2")),
        ("sha", std::arch::is_x86_feature_detected!("sha")),
        ("avx512f", std::arch::is_x86_feature_detected!("avx512f")),
    ] {
        if present {
            features.push(name.into());
        }
    }

    (vendor, brand, family, model, stepping, cache_line, features)
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn detect_platform() -> (String, String, u32, u32, u32, u32, Vec<String>) {
    let mut features = Vec::new();
    #[cfg(target_arch = "aarch64")]
    {
        features.push("neon".into());
    }
    (
        std::env::consts::ARCH.into(),
        "portable".into(),
        0,
        0,
        0,
        64,
        features,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_signature_is_deterministic_and_populated() {
        let a = CpuSignature::detect();
        let b = CpuSignature::detect();
        assert_eq!(a, b);
        assert_eq!(a.fingerprint.len(), 64);
        assert!(a.logical_cores >= 1);
        assert!(a.cache_line_bytes >= 32);
        assert!(!a.vendor.is_empty());
    }
}
