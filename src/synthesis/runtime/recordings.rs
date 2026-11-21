use super::{Runtime, mixer};
use crate::{model::Project, synthesis::audio::PlaybackSource};
use std::sync::Arc;

impl Runtime {
    pub(super) fn prepare_recordings(&mut self, project: &Project) {
        match mixer::recordings(project, &self.instrumentals, &self.solo_tracks) {
            Ok(mix) => {
                self.recordings = mix;
                self.recording_error = None;
            }
            Err(error) => {
                self.recordings = mixer::RecordingMix::default();
                self.recording_error = Some(error);
            }
        }
    }

    pub(super) fn refresh_playback(&self) {
        let mut pending = self.recordings.pending.clone();
        let mut end = self.recordings.end;
        let plan = self.planner.plan();
        for (id, request) in &plan.requests {
            let coverage = request.coverage;
            let start = (coverage.sample_start.0.max(0) as f64 * 48_000.0
                / coverage.sample_rate as f64)
                .ceil() as usize;
            let last = (coverage.sample_end.0.max(0) as f64 * 48_000.0
                / coverage.sample_rate as f64)
                .ceil() as usize;
            end = end.max(last);
            let voice = self.routes.get(id).and_then(|key| self.voices.get(key));
            if voice.is_some_and(|v| v.live.is_some()) {
                continue;
            }
            // A missing or failed singer must not hold the entire project
            // transport in a permanent buffering state. It is reported in the
            // render panel; other singers and accompaniment remain playable.
            // Only a voice still loading/preparing can block its time span.
            if self.binding_errors.contains_key(id) {
                continue;
            }
            if let Some(voice) = voice {
                if matches!(voice.state, super::VoiceState::Failed(_))
                    || (voice.worker.is_none() && !voice.ready)
                    || (voice.error.is_some() && !voice.waiting && voice.stats.running == 0
                        && voice.stats.queued == 0 && !voice.ready)
                {
                    continue;
                }
            } else if !self.pending_voices.contains(id) || !self.library.is_scanning() {
                continue;
            }
            let output = &plan.output_controls[id];
            let mute = output
                .mixer
                .get("mute")
                .and_then(serde_json::Value::as_bool)
                == Some(true);
            if !mute
                && !output.reference_muted
                && (self.solo_tracks.is_empty() || self.solo_tracks.contains(&output.track_id))
            {
                pending.push(start..last);
            }
        }
        if (end == 0 && !self.metronome_enabled() && self.loop_range.is_none())
            || self.recording_error.is_some()
        {
            self.playback.clear();
        } else {
            let source = PlaybackSource::new(
                self.voices
                    .values()
                    .filter_map(|v| v.live.clone())
                    .collect(),
                self.recordings.layers.clone(),
                pending,
                end.max(self.metronome.end_frames),
            )
            .with_metronome(Arc::clone(&self.metronome.track));
            self.playback
                .load_source(Arc::new(source), self.desired_seconds, self.desired_playing);
        }
    }
}
