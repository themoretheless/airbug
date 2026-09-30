//! Raw and overhead-adjusted intervals travel together until sample emission.
use crate::{Result, error};
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Measured {
    pub raw_ns: u128,
    pub adjusted_ns: u128,
}
impl Measured {
    pub fn raw(raw_ns: u128) -> Self {
        Self {
            raw_ns,
            adjusted_ns: raw_ns,
        }
    }
    pub fn add(self, other: Self) -> Result<Self> {
        Ok(Self {
            raw_ns: self
                .raw_ns
                .checked_add(other.raw_ns)
                .ok_or_else(|| error("raw interval sum overflow"))?,
            adjusted_ns: self
                .adjusted_ns
                .checked_add(other.adjusted_ns)
                .ok_or_else(|| error("adjusted interval sum overflow"))?,
        })
    }
    pub fn max(self, other: Self) -> Self {
        Self {
            raw_ns: self.raw_ns.max(other.raw_ns),
            adjusted_ns: self.adjusted_ns.max(other.adjusted_ns),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Correction {
    pub loop_batch_ns: u128,
    pub loop_iterations: u64,
    pub tally_batch_ns: [u128; 4],
    pub tally_iterations: u64,
}
impl Correction {
    pub fn apply(
        self,
        raw_ns: u128,
        operations: u64,
        counts: Option<&crate::alloc::ThreadStats>,
    ) -> Result<Measured> {
        if self.loop_iterations == 0 || self.tally_iterations == 0 {
            return Err(error("invalid overhead denominators"));
        }
        let loop_numerator = self
            .loop_batch_ns
            .checked_mul(operations as u128)
            .ok_or_else(|| error("loop overhead overflow"))?;
        let mut tally = 0u128;
        if let Some(counts) = counts {
            if counts.overflowed
                || counts.grow_operations.checked_add(counts.shrink_operations)
                    != Some(counts.reallocations)
            {
                return Err(error("invalid overhead allocation counters"));
            }
            for (cost, count) in self.tally_batch_ns.into_iter().zip([
                counts.allocations,
                counts.deallocations,
                counts.grow_operations,
                counts.shrink_operations,
            ]) {
                let term = cost
                    .checked_mul(count)
                    .ok_or_else(|| error("tally overhead overflow"))?;
                tally = tally
                    .checked_add(term)
                    .ok_or_else(|| error("tally overhead sum overflow"))?;
            }
        }
        let a = self.loop_iterations as u128;
        let b = self.tally_iterations as u128;
        // Denominators are u64: their product and cross-multiplied
        // remainders fit u128. Compare against the gap to avoid overflow
        // when adding two remainders, and round only after combining costs.
        let left = (loop_numerator % a) * b;
        let right = (tally % b) * a;
        let carry = u128::from(left >= a * b - right);
        let cost = (loop_numerator / a)
            .checked_add(tally / b)
            .and_then(|n| n.checked_add(carry))
            .ok_or_else(|| error("overhead sum overflow"))?;
        Ok(Measured {
            raw_ns,
            adjusted_ns: raw_ns.saturating_sub(cost),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn correction_rounds_combined_fractions_and_preserves_raw() {
        let c = Correction {
            loop_batch_ns: 2,
            loop_iterations: 3,
            tally_batch_ns: [1, 0, 0, 0],
            tally_iterations: 2,
        };
        let stats = crate::alloc::ThreadStats {
            allocations: 1,
            ..Default::default()
        };
        let measured = c.apply(10, 1, Some(&stats)).unwrap();
        assert_eq!(measured.raw_ns, 10);
        assert_eq!(measured.adjusted_ns, 9);
        assert_eq!(c.apply(1, 100, None).unwrap().adjusted_ns, 0);
        let overflow = Correction {
            loop_batch_ns: u128::MAX,
            ..c
        };
        assert!(overflow.apply(10, 2, None).is_err());
        assert!(Measured::raw(u128::MAX).add(Measured::raw(1)).is_err());
    }
    #[test]
    fn worker_maxima_and_wave_clamps_precede_sample_sum() {
        let c = Correction {
            loop_batch_ns: 0,
            loop_iterations: 1,
            tally_batch_ns: [1, 0, 0, 0],
            tally_iterations: 1,
        };
        let slow = c
            .apply(
                100,
                1,
                Some(&crate::alloc::ThreadStats {
                    allocations: 100,
                    ..Default::default()
                }),
            )
            .unwrap();
        let fast = c.apply(90, 1, None).unwrap();
        let wave = slow.max(fast);
        assert_eq!((wave.raw_ns, wave.adjusted_ns), (100, 90));
        let c = Correction {
            loop_batch_ns: 30,
            ..c
        };
        let sum = c
            .apply(10, 1, None)
            .unwrap()
            .add(c.apply(100, 1, None).unwrap())
            .unwrap();
        assert_eq!((sum.raw_ns, sum.adjusted_ns), (110, 70));
        assert_ne!(sum.adjusted_ns, c.apply(110, 2, None).unwrap().adjusted_ns);
    }
}
