//! The two native PCM components are retained independently for phase matching.
//! Playback sums the primary and secondary components after publication/cropping.

use crate::synthesis::{
    audio::{ChannelLayout, RenderedAudio},
    score::SAMPLE_RATE,
    stages::vocoder::VocoderOutput,
    voice::{Result, VoiceError},
};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct PcmComponents {
    pub start_seconds: f64,
    pub primary: Arc<[f32]>,
    pub secondary: Arc<[f32]>,
}

impl PcmComponents {
    pub fn new(start_seconds: f64, output: VocoderOutput) -> Result<Self> {
        if !start_seconds.is_finite()
            || output.primary.values.len() != output.secondary.values.len()
            || output
                .primary
                .values
                .iter()
                .chain(&output.secondary.values)
                .any(|v| !v.is_finite())
        {
            return Err(VoiceError::new("Invalid vocoder component timeline"));
        }
        Ok(Self {
            start_seconds,
            primary: output.primary.values.into(),
            secondary: output.secondary.values.into(),
        })
    }

    pub fn publish(&self, begin_seconds: f64, end_seconds: Option<f64>) -> Result<RenderedAudio> {
        let rate = f64::from(SAMPLE_RATE);
        let first = ((begin_seconds.max(0.0) - self.start_seconds) * rate)
            .ceil()
            .clamp(0.0, self.primary.len() as f64) as usize;
        let end = end_seconds.map_or(self.primary.len(), |end| {
            ((end - self.start_seconds) * rate)
                .floor()
                .clamp(0.0, self.primary.len() as f64) as usize
        });
        if first >= end || self.primary.len() != self.secondary.len() {
            return Err(VoiceError::new(
                "Rendered components have no visible samples",
            ));
        }
        let samples: Vec<_> = self.primary[first..end]
            .iter()
            .zip(&self.secondary[first..end])
            .map(|(&a, &b)| a + b)
            .collect();
        if samples.iter().any(|value| !value.is_finite()) {
            return Err(VoiceError::new("Vocoder produced non-finite mono samples"));
        }
        RenderedAudio::new(
            SAMPLE_RATE,
            self.start_seconds + first as f64 / rate,
            ChannelLayout::Mono,
            samples,
        )
        .map_err(|error| VoiceError::new(error.to_string()))
    }

    /// Extract a single native vocoder component using the *exact* visible
    /// sample window of the published full output. Keeping these boundaries
    /// identical is required for phase-coherent stem summation.
    pub fn publish_component(
        &self,
        visible: &RenderedAudio,
        secondary: bool,
    ) -> Result<RenderedAudio> {
        if visible.sample_rate() != SAMPLE_RATE
            || visible.layout() != ChannelLayout::Mono
            || self.primary.len() != self.secondary.len()
        {
            return Err(VoiceError::new("Vocoder stem reference is incompatible"));
        }
        let rate = f64::from(SAMPLE_RATE);
        let offset = ((visible.start_seconds() - self.start_seconds) * rate).round();
        if !offset.is_finite() || offset < 0.0 || offset > self.primary.len() as f64 {
            return Err(VoiceError::new("Vocoder stem start is outside the original PCM"));
        }
        let first = offset as usize;
        let end = first.checked_add(visible.frames())
            .filter(|&end| end <= self.primary.len())
            .ok_or_else(|| VoiceError::new("Vocoder stem has mismatched sample length"))?;
        let source = if secondary { &self.secondary } else { &self.primary };
        RenderedAudio::new(
            SAMPLE_RATE,
            visible.start_seconds(),
            ChannelLayout::Mono,
            source[first..end].to_vec(),
        ).map_err(|error| VoiceError::new(error.to_string()))
    }

    pub fn memory_bytes(&self) -> usize {
        std::mem::size_of_val(self.primary.as_ref())
            .saturating_add(std::mem::size_of_val(self.secondary.as_ref()))
    }
}
