//! Dependency-free RGBA8 golden comparison. Color space and renderer identity belong in case contracts.
use crate::{error, Result};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
#[derive(Clone, Debug)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}
#[derive(Debug, Serialize)]
pub struct Difference {
    pub changed_pixels: usize,
    pub total_pixels: usize,
    pub max_channel_delta: u8,
    pub mean_channel_delta: f64,
    pub accepted: bool,
}
impl RgbaImage {
    pub fn validate(&self) -> Result<()> {
        let size = u64::from(self.width) * u64::from(self.height) * 4;
        if size == 0 || size > 256 * 1024 * 1024 || size != self.pixels.len() as u64 {
            return Err(error("invalid RGBA dimensions/length (limit 256 MiB)"));
        }
        Ok(())
    }
    /// A changed pixel has at least one RGBA channel beyond channel_tolerance.
    pub fn compare(
        &self,
        expected: &Self,
        channel_tolerance: u8,
        max_changed_percent: f64,
    ) -> Result<Difference> {
        self.validate()?;
        expected.validate()?;
        if !max_changed_percent.is_finite() || !(0.0..=100.0).contains(&max_changed_percent) {
            return Err(error("pixel percentage must be 0..100"));
        }
        if (self.width, self.height) != (expected.width, expected.height) {
            return Err(error("golden dimensions differ"));
        }
        let mut changed = 0;
        let mut maximum = 0;
        let mut sum = 0u64;
        for (a, b) in self
            .pixels
            .chunks_exact(4)
            .zip(expected.pixels.chunks_exact(4))
        {
            let mut bad = false;
            for (a, b) in a.iter().zip(b) {
                let d = a.abs_diff(*b);
                maximum = maximum.max(d);
                sum += u64::from(d);
                bad |= d > channel_tolerance;
            }
            changed += usize::from(bad);
        }
        let total = self.pixels.len() / 4;
        Ok(Difference {
            changed_pixels: changed,
            total_pixels: total,
            max_channel_delta: maximum,
            mean_channel_delta: sum as f64 / self.pixels.len() as f64,
            accepted: changed as f64 * 100.0 / total as f64 <= max_changed_percent,
        })
    }
    /// Versioned raw format: RBIMG001 + width/height little endian + tightly packed RGBA8.
    pub fn save_new(&self, path: impl AsRef<Path>) -> Result<()> {
        self.validate()?;
        let mut f = OpenOptions::new().write(true).create_new(true).open(path)?;
        f.write_all(b"RBIMG001")?;
        f.write_all(&self.width.to_le_bytes())?;
        f.write_all(&self.height.to_le_bytes())?;
        f.write_all(&self.pixels)?;
        f.sync_all()?;
        Ok(())
    }
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if fs::metadata(path)?.len() > 256 * 1024 * 1024 + 16 {
            return Err(error("golden exceeds size limit"));
        }
        let b = fs::read(path)?;
        if b.len() < 16 || &b[..8] != b"RBIMG001" {
            return Err(error("invalid golden header"));
        }
        let image = Self {
            width: u32::from_le_bytes(b[8..12].try_into()?),
            height: u32::from_le_bytes(b[12..16].try_into()?),
            pixels: b[16..].into(),
        };
        image.validate()?;
        Ok(image)
    }
}
