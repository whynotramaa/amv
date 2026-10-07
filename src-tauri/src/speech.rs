//! Local speech worker using Parakeet TDT through sherpa-onnx.

use crate::audio::{AudioSource, CaptureSession};
use sherpa_onnx::{
    OfflineModelConfig, OfflineRecognizer, OfflineRecognizerConfig, OfflineTransducerModelConfig,
};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use webrtc_vad::{SampleRate, Vad, VadMode};

pub const FRAME_SAMPLES: usize = 320;
pub const MAX_OUTPUTS: usize = 32;

#[derive(Clone, Debug)]
pub struct SpeechConfig {
    pub model_path: PathBuf,
    pub selected_mic_id: Option<String>,
    pub pre_roll: Duration,
    pub silence: Duration,
    pub min_speech: Duration,
    pub max_utterance: Duration,
}

impl Default for SpeechConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::new(),
            selected_mic_id: None,
            pre_roll: Duration::from_millis(160),
            silence: Duration::from_millis(600),
            min_speech: Duration::from_millis(100),
            max_utterance: Duration::from_secs(10),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SpeechEvent {
    Partial {
        id: u64,
        source: AudioSource,
        timestamp: Duration,
        revision: u32,
        text: String,
    },
    Final {
        id: u64,
        source: AudioSource,
        timestamp: Duration,
        revision: u32,
        text: String,
    },
    Error {
        id: u64,
        source: Option<AudioSource>,
        message: String,
    },
    Flushed {
        ticket: u64,
    },
}

pub struct SpeechSession {
    events: Receiver<SpeechEvent>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl SpeechSession {
    pub fn start(config: SpeechConfig, flush: Arc<AtomicU64>) -> Result<Self, String> {
        validate_config(&config)?;
        let context = load_recognizer(&config.model_path)?;
        let capture = CaptureSession::start(config.selected_mic_id.as_deref())?;
        let (tx, events) = mpsc::sync_channel(MAX_OUTPUTS);
        let stop = Arc::new(AtomicBool::new(false));
        let stop_for_worker = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name("speech-worker".to_owned())
            .spawn(move || speech_worker(context, capture, config, tx, stop_for_worker, flush))
            .map_err(|error| error.to_string())?;
        Ok(Self {
            events,
            stop,
            worker: Some(worker),
        })
    }

    pub fn events(&self) -> &Receiver<SpeechEvent> {
        &self.events
    }

    /// Drains pending finalized events while joining, avoiding bounded-channel deadlock.
    pub fn stop(mut self) -> Result<Vec<SpeechEvent>, String> {
        self.stop.store(true, Ordering::Release);
        let worker = self.worker.take().ok_or("speech worker missing")?;
        let mut pending = Vec::new();
        while !worker.is_finished() {
            if let Ok(event) = self.events.recv_timeout(Duration::from_millis(50)) {
                pending.push(event);
            }
        }
        pending.extend(self.events.try_iter());
        if worker.join().is_err() {
            pending.push(SpeechEvent::Error {
                id: 0,
                source: None,
                message: "Local transcription stopped unexpectedly".into(),
            });
        }
        Ok(pending)
    }
}

impl Drop for SpeechSession {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let (_, empty) = mpsc::sync_channel(1);
        drop(std::mem::replace(&mut self.events, empty));
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn validate_config(config: &SpeechConfig) -> Result<(), String> {
    if config.model_path.as_os_str().is_empty() {
        return Err("model_path is required".into());
    }
    if !config.model_path.join("tokens.txt").is_file() {
        return Err("Speech model files are missing. Reinstall Harness.".into());
    }
    if config.pre_roll > Duration::from_secs(15)
        || config.silence.is_zero()
        || config.min_speech.is_zero()
        || config.max_utterance.is_zero()
        || config.max_utterance > Duration::from_secs(15)
        || config.silence > Duration::from_secs(5)
        || config.min_speech > config.max_utterance
        || config.pre_roll > config.max_utterance
    {
        return Err("invalid speech timing bounds".into());
    }
    Ok(())
}

struct Segmenter {
    vad: Vad,
    config: SpeechConfig,
    pre_roll: Vec<f32>,
    utterance: Vec<f32>,
    source: Option<AudioSource>,
    started: Option<Duration>,
    silent_for: Duration,
    voiced_for: Duration,
}

impl Segmenter {
    fn new(config: SpeechConfig) -> Self {
        Self {
            vad: Vad::new_with_rate_and_mode(SampleRate::Rate16kHz, VadMode::Aggressive),
            config,
            pre_roll: Vec::new(),
            utterance: Vec::new(),
            source: None,
            started: None,
            silent_for: Duration::ZERO,
            voiced_for: Duration::ZERO,
        }
    }

    fn push(
        &mut self,
        source: AudioSource,
        timestamp: Duration,
        samples: &[f32],
    ) -> Result<Option<Utterance>, String> {
        if samples.len() != FRAME_SAMPLES {
            return Err("VAD requires exactly 20 ms / 320 samples".into());
        }
        let mut pcm = [0i16; FRAME_SAMPLES];
        for (target, sample) in pcm.iter_mut().zip(samples) {
            *target = if !sample.is_finite() {
                0
            } else {
                (sample.clamp(-1.0, 1.0) * 32767.0) as i16
            };
        }
        let voice = self
            .vad
            .is_voice_segment(&pcm)
            .map_err(|_| "invalid VAD frame length".to_owned())?;
        let frame_time = Duration::from_millis(20);
        if self.started.is_none() {
            self.pre_roll.extend_from_slice(samples);
            let max = (self.config.pre_roll.as_millis() as usize / 20) * FRAME_SAMPLES;
            if self.pre_roll.len() > max {
                self.pre_roll.drain(..self.pre_roll.len() - max);
            }
            if voice {
                self.source = Some(source);
                self.started = Some(timestamp.saturating_sub(Duration::from_secs_f64(
                    self.pre_roll.len().saturating_sub(FRAME_SAMPLES) as f64 / 16000.0,
                )));
                if self.pre_roll.is_empty() {
                    self.pre_roll.extend_from_slice(samples);
                }
                self.utterance.append(&mut self.pre_roll);
                self.silent_for = Duration::ZERO;
                self.voiced_for = frame_time;
            }
            return Ok(None);
        }
        self.utterance.extend_from_slice(samples);
        if voice {
            self.silent_for = Duration::ZERO;
            self.voiced_for += frame_time;
        } else {
            self.silent_for += frame_time;
        }
        let too_long = self.utterance.len() >= self.config.max_utterance.as_millis() as usize * 16;
        let enough_quiet = self.silent_for >= self.config.silence;
        if too_long || enough_quiet {
            let utterance = if self.voiced_for >= self.config.min_speech {
                Some(Utterance {
                    source: self.source.take().unwrap_or(source),
                    timestamp: self.started.take().unwrap_or(timestamp),
                    samples: std::mem::take(&mut self.utterance),
                })
            } else {
                self.reset();
                None
            };
            self.reset();
            return Ok(utterance);
        }
        Ok(None)
    }

    fn flush(&mut self) -> Option<Utterance> {
        if self.voiced_for >= self.config.min_speech {
            let result = Some(Utterance {
                source: self.source.take()?,
                timestamp: self.started.take()?,
                samples: std::mem::take(&mut self.utterance),
            });
            self.reset();
            result
        } else {
            self.reset();
            None
        }
    }

    fn reset(&mut self) {
        self.utterance.clear();
        self.pre_roll.clear();
        self.source = None;
        self.started = None;
        self.silent_for = Duration::ZERO;
        self.voiced_for = Duration::ZERO;
    }
}

struct Utterance {
    source: AudioSource,
    timestamp: Duration,
    samples: Vec<f32>,
}

enum Job {
    Decode(Utterance, u64),
    Mark(u64),
}

fn speech_worker(
    context: OfflineRecognizer,
    capture: CaptureSession,
    config: SpeechConfig,
    tx: SyncSender<SpeechEvent>,
    stop: Arc<AtomicBool>,
    flush: Arc<AtomicU64>,
) {
    // Segmentation continues while one shared model decodes bounded utterances.
    let (decode_tx, decode_rx) = mpsc::sync_channel::<Job>(4);
    let output = tx.clone();
    let decoder = match thread::Builder::new()
        .name("speech-decoder".into())
        .spawn(move || {
            while let Ok(job) = decode_rx.recv() {
                match job {
                    Job::Decode(utterance, id) => transcribe(&context, utterance, id, &output),
                    Job::Mark(ticket) => send_blocking(&output, SpeechEvent::Flushed { ticket }),
                }
            }
        }) {
        Ok(worker) => worker,
        Err(_) => {
            send_blocking(
                &tx,
                SpeechEvent::Error {
                    id: 0,
                    source: None,
                    message: "Couldn't start local transcription".into(),
                },
            );
            return;
        }
    };
    // Each source has its own VAD and frame remainder; microphone speech never enters system audio.
    let mut segmenters = [Segmenter::new(config.clone()), Segmenter::new(config)];
    let mut remainders = [
        Vec::with_capacity(FRAME_SAMPLES),
        Vec::with_capacity(FRAME_SAMPLES),
    ];
    let mut remainder_times = [Duration::ZERO; 2];
    let mut sequences = [None, None];
    let mut capture_errors = [0, 0];
    let mut next_id = 1u64;
    let mut flushed = flush.load(Ordering::Acquire);
    while !stop.load(Ordering::Acquire) {
        let wanted = flush.load(Ordering::Acquire);
        if wanted != flushed {
            for segmenter in &mut segmenters {
                if let Some(utterance) = segmenter.flush() {
                    queue_utterance(&decode_tx, utterance, next_id, &tx);
                    next_id += 1;
                }
            }
            let _ = decode_tx.send(Job::Mark(wanted));
            flushed = wanted;
        }
        for (source, metrics) in capture.metrics() {
            let index = match source {
                AudioSource::System => 0,
                AudioSource::Microphone => 1,
            };
            if metrics.error_count > capture_errors[index] {
                capture_errors[index] = metrics.error_count;
                send_blocking(
                    &tx,
                    SpeechEvent::Error {
                        id: next_id,
                        source: Some(source),
                        message: "Audio capture stopped for this source. Check the audio device."
                            .into(),
                    },
                );
                next_id += 1;
            }
        }
        if let Some(frame) = capture.recv_timeout(Duration::from_millis(50)) {
            let index = match frame.source {
                AudioSource::System => 0,
                AudioSource::Microphone => 1,
            };
            if sequences[index].is_some_and(|previous| frame.sequence != previous + 1) {
                if let Some(utterance) = segmenters[index].flush() {
                    queue_utterance(&decode_tx, utterance, next_id, &tx);
                    next_id += 1;
                }
                remainders[index].clear();
                send_blocking(
                    &tx,
                    SpeechEvent::Error {
                        id: next_id,
                        source: Some(frame.source),
                        message: "Audio frames were dropped; this transcript has a gap.".into(),
                    },
                );
                next_id += 1;
            }
            sequences[index] = Some(frame.sequence);
            let mut offset = 0;
            while offset < frame.samples.len() {
                if remainders[index].is_empty() {
                    remainder_times[index] =
                        frame.timestamp + Duration::from_secs_f64(offset as f64 / 16000.0);
                }
                let count =
                    (FRAME_SAMPLES - remainders[index].len()).min(frame.samples.len() - offset);
                remainders[index].extend_from_slice(&frame.samples[offset..offset + count]);
                offset += count;
                if remainders[index].len() == FRAME_SAMPLES {
                    match segmenters[index].push(
                        frame.source,
                        remainder_times[index],
                        &remainders[index],
                    ) {
                        Ok(Some(utterance)) => {
                            queue_utterance(&decode_tx, utterance, next_id, &tx);
                            next_id += 1;
                        }
                        Ok(None) => {}
                        Err(message) => {
                            send_blocking(
                                &tx,
                                SpeechEvent::Error {
                                    id: next_id,
                                    source: Some(frame.source),
                                    message,
                                },
                            );
                            next_id += 1;
                        }
                    }
                    remainders[index].clear();
                }
            }
        } else if capture.is_finished() {
            send_blocking(
                &tx,
                SpeechEvent::Error {
                    id: next_id,
                    source: None,
                    message: "Audio capture is no longer running. Restart capture after checking your devices.".into(),
                },
            );
            next_id += 1;
            break;
        }
    }
    // Stop capture first, then flush only the bounded utterances already segmented.
    if let Err(message) = capture.stop() {
        send_blocking(
            &tx,
            SpeechEvent::Error {
                id: next_id,
                source: None,
                message,
            },
        );
        next_id += 1;
    }
    for segmenter in &mut segmenters {
        if let Some(utterance) = segmenter.flush() {
            let _ = decode_tx.send(Job::Decode(utterance, next_id));
            next_id += 1;
        }
    }
    drop(decode_tx);
    if decoder.join().is_err() {
        send_blocking(
            &tx,
            SpeechEvent::Error {
                id: next_id,
                source: None,
                message: "Local transcription stopped unexpectedly".into(),
            },
        );
    }
}

fn queue_utterance(
    tx: &SyncSender<Job>,
    utterance: Utterance,
    id: u64,
    output: &SyncSender<SpeechEvent>,
) {
    let source = utterance.source;
    if tx.try_send(Job::Decode(utterance, id)).is_err() {
        send_blocking(output, SpeechEvent::Error { id, source: Some(source), message: "Transcription couldn't keep up; some audio was dropped. Your finalized notes are preserved.".into() });
    }
}

pub fn load_recognizer(dir: &Path) -> Result<OfflineRecognizer, String> {
    let file = |name: &str| Some(dir.join(name).to_string_lossy().into_owned());
    let threads = thread::available_parallelism().map_or(1, |n| n.get().min(4));
    let config = OfflineRecognizerConfig {
        model_config: OfflineModelConfig {
            transducer: OfflineTransducerModelConfig {
                encoder: file("encoder.int8.onnx"),
                decoder: file("decoder.int8.onnx"),
                joiner: file("joiner.int8.onnx"),
            },
            tokens: file("tokens.txt"),
            num_threads: threads as i32,
            provider: Some("cpu".into()),
            model_type: Some("nemo_transducer".into()),
            ..Default::default()
        },
        decoding_method: Some("greedy_search".into()),
        ..Default::default()
    };
    OfflineRecognizer::create(&config).ok_or_else(|| "Couldn't load the speech model".into())
}

pub fn decode(context: &OfflineRecognizer, samples: &[f32]) -> Result<String, String> {
    if samples.is_empty() || samples.len() > 15 * 16000 || samples.iter().any(|x| !x.is_finite()) {
        return Err("Speech decode requires finite audio between 0 and 15 seconds".into());
    }
    let stream = context.create_stream();
    stream.accept_waveform(16000, samples);
    context.decode(&stream);
    let text = stream
        .get_result()
        .map(|result| result.text)
        .unwrap_or_default();
    if text.len() > 64 * 1024 {
        return Err("Speech output exceeded its bound".into());
    }
    Ok(text.trim().to_owned())
}

fn transcribe(
    context: &OfflineRecognizer,
    utterance: Utterance,
    id: u64,
    tx: &SyncSender<SpeechEvent>,
) {
    match decode(context, &utterance.samples) {
        Ok(text) if !text.is_empty() => send_blocking(
            tx,
            SpeechEvent::Final {
                id,
                source: utterance.source,
                timestamp: utterance.timestamp,
                revision: 1,
                text,
            },
        ),
        Ok(_) => {}
        Err(message) => send_blocking(
            tx,
            SpeechEvent::Error {
                id,
                source: Some(utterance.source),
                message,
            },
        ),
    }
}

fn send_blocking(tx: &SyncSender<SpeechEvent>, event: SpeechEvent) {
    let _ = tx.send(event);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn segmentation_rejects_bad_vad_frame_without_clipping() {
        let config = SpeechConfig {
            model_path: PathBuf::from("model"),
            ..SpeechConfig::default()
        };
        let mut segmenter = Segmenter::new(config);
        assert!(segmenter
            .push(AudioSource::Microphone, Duration::ZERO, &[0.0; 319])
            .is_err());
        assert!(segmenter.utterance.is_empty());
    }

    #[test]
    fn quiet_padding_does_not_satisfy_minimum_speech() {
        let mut segmenter = Segmenter::new(SpeechConfig::default());
        segmenter.source = Some(AudioSource::System);
        segmenter.started = Some(Duration::ZERO);
        segmenter.utterance = vec![0.0; 16000];
        segmenter.voiced_for = Duration::from_millis(20);
        assert!(segmenter.flush().is_none());
        segmenter.source = Some(AudioSource::System);
        segmenter.started = Some(Duration::ZERO);
        segmenter.utterance = vec![0.0; 1600];
        segmenter.voiced_for = Duration::from_millis(100);
        assert!(segmenter.flush().is_some());
        assert_eq!(segmenter.voiced_for, Duration::ZERO);
    }

    #[test]
    fn stop_preserves_finalized_events_when_worker_panics() {
        let (tx, events) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || {
            tx.send(SpeechEvent::Final {
                id: 1,
                source: AudioSource::System,
                timestamp: Duration::ZERO,
                revision: 1,
                text: "Saved speech".into(),
            })
            .unwrap();
            panic!("simulated speech worker failure");
        });
        let session = SpeechSession {
            events,
            stop: Arc::new(AtomicBool::new(false)),
            worker: Some(worker),
        };
        let pending = session.stop().unwrap();
        assert!(matches!(&pending[0], SpeechEvent::Final { text, .. } if text == "Saved speech"));
        assert!(matches!(&pending[1], SpeechEvent::Error { .. }));
    }
}
