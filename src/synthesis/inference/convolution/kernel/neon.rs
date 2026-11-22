//! ARM64 Advanced SIMD convolution with independent eight-channel lanes.
//! Multiplication and addition remain separate to preserve the native
//! non-fused accumulation order across taps, inputs and output channels.

use std::arch::aarch64::{
    float32x4_t, vaddq_f32, vdupq_n_f32, vld1q_f32, vmulq_f32, vst1q_f32,
};

#[derive(Clone, Copy)]
pub(super) struct Output {
    low: float32x4_t,
    high: float32x4_t,
}

impl Output {
    #[inline(always)]
    pub(super) fn zero() -> Self {
        unsafe {
            Self {
                low: vdupq_n_f32(0.0),
                high: vdupq_n_f32(0.0),
            }
        }
    }

    #[inline(always)]
    pub(super) fn load(values: &[f32]) -> Self {
        debug_assert!(values.len() <= 8);
        let mut padded = [0.0; 8];
        let source = if values.len() == 8 {
            values
        } else {
            padded[..values.len()].copy_from_slice(values);
            &padded
        };
        // SAFETY: source supplies eight readable f32 values. ARM64 vld1q
        // supports unaligned addresses; partial blocks use stack padding.
        unsafe {
            Self {
                low: vld1q_f32(source.as_ptr()),
                high: vld1q_f32(source.as_ptr().add(4)),
            }
        }
    }

    #[inline(always)]
    pub(super) fn store(self, values: &mut [f32]) {
        debug_assert!(values.len() <= 8);
        let mut padded = [0.0; 8];
        let target = if values.len() == 8 {
            &mut *values
        } else {
            &mut padded
        };
        unsafe {
            vst1q_f32(target.as_mut_ptr(), self.low);
            vst1q_f32(target.as_mut_ptr().add(4), self.high);
        }
        if values.len() != 8 {
            values.copy_from_slice(&padded[..values.len()]);
        }
    }

    #[inline(always)]
    pub(super) fn add(&mut self, value: Self) {
        unsafe {
            self.low = vaddq_f32(self.low, value.low);
            self.high = vaddq_f32(self.high, value.high);
        }
    }
}

#[inline(always)]
pub(super) fn accumulate_one(weights: &[f32], source: &[f32], sum: &mut Output) {
    debug_assert_eq!(weights.len() / 8, source.len());
    for (weights, &sample) in weights.as_chunks::<8>().0.iter().zip(source) {
        unsafe {
            let low = vld1q_f32(weights.as_ptr());
            let high = vld1q_f32(weights.as_ptr().add(4));
            let input = vdupq_n_f32(sample);
            sum.low = vaddq_f32(sum.low, vmulq_f32(low, input));
            sum.high = vaddq_f32(sum.high, vmulq_f32(high, input));
        }
    }
}

#[inline(always)]
pub(super) fn accumulate_four(
    weights: &[f32],
    source: [&[f32]; 4],
    sums: &mut [Output; 4],
) {
    debug_assert!(source.iter().all(|row| row.len() == weights.len() / 8));
    let rows = weights
        .as_chunks::<8>()
        .0
        .iter()
        .zip(source[0])
        .zip(source[1])
        .zip(source[2])
        .zip(source[3]);
    for ((((weights, &a), &b), &c), &d) in rows {
        unsafe {
            let low = vld1q_f32(weights.as_ptr());
            let high = vld1q_f32(weights.as_ptr().add(4));
            for (sum, sample) in sums.iter_mut().zip([a, b, c, d]) {
                let input = vdupq_n_f32(sample);
                sum.low = vaddq_f32(sum.low, vmulq_f32(low, input));
                sum.high = vaddq_f32(sum.high, vmulq_f32(high, input));
            }
        }
    }
}
