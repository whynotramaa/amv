//! Finite WASAPI capture diagnostic. Audio stays in bounded memory queues.

use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AudioSource {
    System,
    Microphone,
}

#[derive(Debug)]
pub struct AudioFrame {
    pub source: AudioSource,
    pub sequence: u64,
    pub timestamp: Duration,
    pub samples: Vec<f32>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CaptureMetrics {
    pub packets: u64,
    pub dropped_packets: u64,
    pub gap_frames: u64,
    pub error_count: u64,
}

#[cfg(not(windows))]
pub struct CaptureSession;

#[cfg(not(windows))]
impl CaptureSession {
    pub fn start(_selected_mic_id: Option<&str>) -> Result<Self, String> {
        Err("WASAPI capture requires Windows".to_owned())
    }

    pub fn recv_timeout(&self, _timeout: Duration) -> Option<AudioFrame> {
        None
    }
    pub fn stop(self) -> Result<Vec<StreamReport>, String> {
        Ok(Vec::new())
    }
    pub fn metrics(&self) -> Vec<(AudioSource, CaptureMetrics)> {
        Vec::new()
    }
    pub fn is_finished(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub direction: &'static str,
    pub label: String,
    pub id: String,
    pub is_default: bool,
}

#[derive(Clone, Debug)]
pub struct StreamReport {
    pub source: &'static str,
    pub device_label: String,
    pub peak_level: f32,
    pub samples: u64,
    pub packets: u64,
    pub dropped_packets: u64,
    pub error_count: u64,
}

#[derive(Clone, Debug)]
pub struct ProbeReport {
    pub devices: Vec<DeviceInfo>,
    pub streams: Vec<StreamReport>,
}

#[cfg(not(windows))]
pub fn run_probe(_duration: Duration) -> Result<ProbeReport, String> {
    Err("WASAPI diagnostics require Windows".to_owned())
}

#[cfg(not(windows))]
pub fn run_probe_with_mic(
    _duration: Duration,
    _selected_mic_id: Option<&str>,
) -> Result<ProbeReport, String> {
    Err("WASAPI diagnostics require Windows".to_owned())
}

#[cfg(not(windows))]
pub fn list_devices() -> Result<Vec<DeviceInfo>, String> {
    Err("Audio device discovery requires Windows".into())
}

#[cfg(windows)]
mod windows {
    use super::{
        decode_samples, AudioFrame, AudioSource, CaptureMetrics, DeviceInfo, ProbeReport,
        StreamReport,
    };
    use std::ptr;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{mpsc, Arc};
    use std::thread;
    use std::time::{Duration, Instant};
    use wasapi::{deinitialize, initialize_mta, DeviceEnumerator, Direction};
    use windows::core::HSTRING;
    use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0, WAIT_TIMEOUT};
    use windows::Win32::Media::Audio::{
        IAudioCaptureClient, IAudioClient, IMMDeviceEnumerator, MMDeviceEnumerator,
        AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM,
        AUDCLNT_STREAMFLAGS_EVENTCALLBACK, AUDCLNT_STREAMFLAGS_LOOPBACK,
        AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY, WAVEFORMATEX,
    };
    use windows::Win32::Media::Multimedia::WAVE_FORMAT_IEEE_FLOAT;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
    };
    use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};

    const QUEUE_CAPACITY: usize = 8;
    const TARGET_RATE: u32 = 16_000;
    const MAX_PACKET_FRAMES: usize = 16_000;

    struct SourceConfig {
        source: &'static str,
        label: String,
        id: String,
        device_direction: Direction,
    }

    struct Counters {
        source: AudioSource,
        packets: AtomicU64,
        dropped_packets: AtomicU64,
        gap_frames: AtomicU64,
        error_count: AtomicU64,
    }

    pub struct CaptureSession {
        receivers: Vec<(mpsc::Receiver<AudioFrame>, Arc<Counters>)>,
        next_source: std::cell::Cell<usize>,
        stop: Arc<AtomicBool>,
        workers: Vec<thread::JoinHandle<StreamReport>>,
    }

    impl CaptureSession {
        pub fn start(selected_mic_id: Option<&str>) -> Result<Self, String> {
            let (_, sources) = enumerate(selected_mic_id)?;
            if sources.is_empty() {
                return Err("No audio input or system playback device is available".into());
            }
            let stop = Arc::new(AtomicBool::new(false));
            let mut workers = Vec::with_capacity(sources.len());
            let mut receivers = Vec::with_capacity(sources.len());
            for config in sources {
                let (tx, rx) = mpsc::sync_channel(QUEUE_CAPACITY);
                let stop_for_thread = Arc::clone(&stop);
                let source = if config.source == "microphone" {
                    AudioSource::Microphone
                } else {
                    AudioSource::System
                };
                let counters = Arc::new(Counters::new(source));
                let counters_for_thread = Arc::clone(&counters);
                let name = config.source.to_owned();
                match thread::Builder::new()
                    .name(name)
                    .spawn(move || capture_source(config, tx, stop_for_thread, counters_for_thread))
                {
                    Ok(worker) => workers.push(worker),
                    Err(error) => {
                        stop.store(true, Ordering::Release);
                        for worker in workers {
                            let _ = worker.join();
                        }
                        return Err(error.to_string());
                    }
                }
                receivers.push((rx, counters));
            }
            Ok(Self {
                receivers,
                next_source: std::cell::Cell::new(0),
                stop,
                workers,
            })
        }

        pub fn recv_timeout(&self, timeout: Duration) -> Option<AudioFrame> {
            let deadline = Instant::now() + timeout;
            loop {
                for offset in 0..self.receivers.len() {
                    let index = (self.next_source.get() + offset) % self.receivers.len();
                    let (receiver, _) = &self.receivers[index];
                    if let Ok(frame) = receiver.try_recv() {
                        self.next_source.set((index + 1) % self.receivers.len());
                        return Some(frame);
                    }
                }
                if Instant::now() >= deadline || self.stop.load(Ordering::Acquire) {
                    return None;
                }
                thread::sleep(Duration::from_millis(2));
            }
        }

        pub fn stop(mut self) -> Result<Vec<StreamReport>, String> {
            self.stop.store(true, Ordering::Release);
            let mut reports = Vec::with_capacity(self.workers.len());
            let mut failed = false;
            for worker in self.workers.drain(..) {
                match worker.join() {
                    Ok(report) => reports.push(report),
                    Err(_) => failed = true,
                }
            }
            if failed {
                Err("capture thread panicked".to_owned())
            } else {
                Ok(reports)
            }
        }

        pub fn metrics(&self) -> Vec<(AudioSource, CaptureMetrics)> {
            self.receivers
                .iter()
                .map(|(_, counters)| {
                    let source = counters.source;
                    (
                        source,
                        CaptureMetrics {
                            packets: counters.packets.load(Ordering::Relaxed),
                            dropped_packets: counters.dropped_packets.load(Ordering::Relaxed),
                            gap_frames: counters.gap_frames.load(Ordering::Relaxed),
                            error_count: counters.error_count.load(Ordering::Relaxed),
                        },
                    )
                })
                .collect()
        }

        pub fn is_finished(&self) -> bool {
            self.workers.iter().all(thread::JoinHandle::is_finished)
        }
    }

    impl Drop for CaptureSession {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Release);
            for worker in self.workers.drain(..) {
                let _ = worker.join();
            }
        }
    }

    impl Counters {
        fn new(source: AudioSource) -> Self {
            Self {
                source,
                packets: AtomicU64::new(0),
                dropped_packets: AtomicU64::new(0),
                gap_frames: AtomicU64::new(0),
                error_count: AtomicU64::new(0),
            }
        }
    }

    pub fn run_probe(duration: Duration) -> Result<ProbeReport, String> {
        run_probe_with_mic(duration, None)
    }

    pub fn run_probe_with_mic(
        duration: Duration,
        selected_mic_id: Option<&str>,
    ) -> Result<ProbeReport, String> {
        if !(Duration::from_secs(1)..=Duration::from_secs(7200)).contains(&duration) {
            return Err("probe duration must be between 1 and 7200 seconds".to_owned());
        }
        let devices = enumerate(selected_mic_id)?.0;
        let session = CaptureSession::start(selected_mic_id)?;
        let mut consumer_stats = std::collections::HashMap::<AudioSource, (u64, f32)>::new();
        let deadline = Instant::now() + duration;
        while Instant::now() < deadline {
            if let Some(frame) = session.recv_timeout(Duration::from_millis(10)) {
                let stats = consumer_stats.entry(frame.source).or_default();
                stats.0 = stats.0.saturating_add(frame.samples.len() as u64);
                stats.1 = frame
                    .samples
                    .iter()
                    .fold(stats.1, |peak, sample| peak.max(sample.abs()));
            }
        }
        let mut streams = session.stop()?;
        for stream in &mut streams {
            let source = if stream.source == "microphone" {
                AudioSource::Microphone
            } else {
                AudioSource::System
            };
            if let Some((samples, peak)) = consumer_stats.remove(&source) {
                stream.samples = samples;
                stream.peak_level = peak;
            }
        }
        Ok(ProbeReport { devices, streams })
    }

    pub fn list_devices() -> Result<Vec<DeviceInfo>, String> {
        enumerate(None).map(|(devices, _)| devices)
    }

    fn enumerate(
        selected_mic_id: Option<&str>,
    ) -> Result<(Vec<DeviceInfo>, Vec<SourceConfig>), String> {
        if let Err(error) = initialize_mta().ok() {
            return Err(error.to_string());
        }
        let result = enumerate_inner(selected_mic_id);
        deinitialize();
        result
    }

    fn enumerate_inner(
        selected_mic_id: Option<&str>,
    ) -> Result<(Vec<DeviceInfo>, Vec<SourceConfig>), String> {
        let enumerator = DeviceEnumerator::new().map_err(|error| error.to_string())?;
        let default_render = enumerator.get_default_device(&Direction::Render).ok();
        let default_capture = enumerator.get_default_device(&Direction::Capture).ok();
        let default_render_id = default_render.as_ref().and_then(|d| d.get_id().ok());
        let default_capture_id = default_capture.as_ref().and_then(|d| d.get_id().ok());

        let mut devices = Vec::new();
        for (direction, name) in [
            (Direction::Render, "render"),
            (Direction::Capture, "microphone"),
        ] {
            let collection = enumerator
                .get_device_collection(&direction)
                .map_err(|error| error.to_string())?;
            for device in &collection {
                let device = device.map_err(|error| error.to_string())?;
                let id = device.get_id().map_err(|error| error.to_string())?;
                let label = device
                    .get_friendlyname()
                    .unwrap_or_else(|_| "<unnamed>".to_owned());
                let is_default = match direction {
                    Direction::Render => default_render_id.as_deref() == Some(id.as_str()),
                    Direction::Capture => default_capture_id.as_deref() == Some(id.as_str()),
                };
                devices.push(DeviceInfo {
                    direction: name,
                    label,
                    id: id.clone(),
                    is_default,
                });
            }
        }
        let render = default_render
            .zip(default_render_id)
            .map(|(device, id)| SourceConfig {
                source: "system",
                label: device
                    .get_friendlyname()
                    .unwrap_or_else(|_| "<unnamed>".to_owned()),
                id,
                device_direction: Direction::Render,
            });
        let microphone = match selected_mic_id.or(default_capture_id.as_deref()) {
            Some(id) => devices
                .iter()
                .find(|d| d.direction == "microphone" && d.id == id)
                .map(|d| SourceConfig {
                    source: "microphone",
                    label: d.label.clone(),
                    id: d.id.clone(),
                    device_direction: Direction::Capture,
                }),
            None => None,
        };
        if selected_mic_id.is_some() && microphone.is_none() {
            return Err("selected microphone is not an enumerated capture device".to_owned());
        }
        Ok((devices, render.into_iter().chain(microphone).collect()))
    }

    fn capture_source(
        config: SourceConfig,
        tx: mpsc::SyncSender<AudioFrame>,
        stop: Arc<AtomicBool>,
        counters: Arc<Counters>,
    ) -> StreamReport {
        let mut report = StreamReport {
            source: config.source,
            device_label: config.label.clone(),
            peak_level: 0.0,
            samples: 0,
            packets: 0,
            dropped_packets: 0,
            error_count: 0,
        };
        capture_source_inner(&config, tx, &stop, &counters);
        report.packets = counters.packets.load(Ordering::Relaxed);
        report.dropped_packets = counters.dropped_packets.load(Ordering::Relaxed);
        report.error_count += counters.error_count.load(Ordering::Relaxed);
        report
    }

    fn capture_source_inner(
        config: &SourceConfig,
        tx: mpsc::SyncSender<AudioFrame>,
        stop: &AtomicBool,
        counters: &Counters,
    ) {
        let result = (|| -> Result<(), String> {
            // Each worker owns COM on its own thread; no COM interface crosses the thread boundary.
            let _com = ComGuard::new()?;
            let enumerator: IMMDeviceEnumerator =
                unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
                    .map_err(|error| error.to_string())?;
            let device = unsafe { enumerator.GetDevice(&HSTRING::from(&config.id)) }
                .map_err(|error| error.to_string())?;
            let audio_client: IAudioClient =
                unsafe { device.Activate(CLSCTX_ALL, None) }.map_err(|error| error.to_string())?;
            let format = WAVEFORMATEX {
                wFormatTag: WAVE_FORMAT_IEEE_FLOAT as u16,
                nChannels: 1,
                nSamplesPerSec: TARGET_RATE,
                nAvgBytesPerSec: TARGET_RATE * 4,
                nBlockAlign: 4,
                wBitsPerSample: 32,
                cbSize: 0,
            };
            let flags = AUDCLNT_STREAMFLAGS_EVENTCALLBACK
                | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM
                | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY
                | if config.device_direction == Direction::Render {
                    AUDCLNT_STREAMFLAGS_LOOPBACK
                } else {
                    0
                };
            unsafe {
                audio_client.Initialize(AUDCLNT_SHAREMODE_SHARED, flags, 0, 0, &format, None)
            }
            .map_err(|error| error.to_string())?;
            let event = unsafe { CreateEventW(None, false, false, None) }
                .map_err(|error| error.to_string())?;
            let _event_close = HandleGuard(event);
            unsafe { audio_client.SetEventHandle(event) }.map_err(|error| error.to_string())?;
            let capture: IAudioCaptureClient =
                unsafe { audio_client.GetService() }.map_err(|error| error.to_string())?;
            unsafe { audio_client.Start() }.map_err(|error| error.to_string())?;

            let source = if config.source == "microphone" {
                AudioSource::Microphone
            } else {
                AudioSource::System
            };
            let mut sequence = 0;
            let mut timestamp_samples = 0u64;
            while !stop.load(Ordering::Acquire) {
                loop {
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    let frames = unsafe { capture.GetNextPacketSize() }
                        .map_err(|error| error.to_string())?;
                    if frames == 0 {
                        break;
                    }
                    if frames as usize > MAX_PACKET_FRAMES {
                        return Err("WASAPI packet exceeds diagnostic bound".to_owned());
                    }
                    let mut data = ptr::null_mut();
                    let mut flags = 0;
                    let mut frame_count = frames;
                    unsafe {
                        capture.GetBuffer(&mut data, &mut frame_count, &mut flags, None, None)
                    }
                    .map_err(|error| error.to_string())?;
                    // GetBuffer succeeded: ReleaseBuffer is mandatory on every path below.
                    let packet = if frame_count as usize > MAX_PACKET_FRAMES {
                        Err("WASAPI packet exceeds diagnostic bound".to_owned())
                    } else {
                        let silent = flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0;
                        let bytes = if silent || data.is_null() || frame_count == 0 {
                            None
                        } else {
                            // WASAPI owns this non-null packet until ReleaseBuffer below.
                            // The negotiated mono float32 format is four bytes per frame.
                            Some(unsafe {
                                std::slice::from_raw_parts(data, frame_count as usize * 4)
                            })
                        };
                        decode_samples(bytes, frame_count as usize, silent)
                    };
                    let release = unsafe { capture.ReleaseBuffer(frame_count) };
                    if let Err(error) = release {
                        return Err(error.to_string());
                    }
                    let samples = packet?;
                    counters.packets.fetch_add(1, Ordering::Relaxed);
                    let frame = AudioFrame {
                        source,
                        sequence,
                        timestamp: Duration::from_secs_f64(
                            timestamp_samples as f64 / TARGET_RATE as f64,
                        ),
                        samples,
                    };
                    sequence = sequence.saturating_add(1);
                    timestamp_samples =
                        timestamp_samples.saturating_add(frame.samples.len() as u64);
                    let sample_count = frame.samples.len() as u64;
                    if tx.try_send(frame).is_err() {
                        counters.dropped_packets.fetch_add(1, Ordering::Relaxed);
                        counters
                            .gap_frames
                            .fetch_add(sample_count, Ordering::Relaxed);
                    }
                }
                match unsafe { WaitForSingleObject(event, 100) } {
                    WAIT_OBJECT_0 | WAIT_TIMEOUT => {}
                    _ => return Err("WASAPI event wait failed".to_owned()),
                }
            }
            let _ = unsafe { audio_client.Stop() };
            Ok(())
        })();
        if result.is_err() {
            counters.error_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    struct ComGuard;
    impl ComGuard {
        fn new() -> Result<Self, String> {
            unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
                .ok()
                .map(|_| Self)
                .map_err(|e| e.to_string())
        }
    }
    impl Drop for ComGuard {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }

    struct HandleGuard(windows::Win32::Foundation::HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

#[cfg(any(windows, test))]
fn decode_samples(data: Option<&[u8]>, frames: usize, silent: bool) -> Result<Vec<f32>, String> {
    if frames > 16_000 {
        return Err("packet exceeds the 16,000-frame bound".into());
    }
    if silent || frames == 0 {
        return Ok(vec![0.0; frames]);
    }
    let data = data.ok_or("null non-silent packet")?;
    if data.len() != frames * 4 {
        return Err("short packet".into());
    }
    let samples: Vec<f32> = data
        .as_chunks::<4>()
        .0
        .iter()
        .map(|bytes| f32::from_le_bytes(*bytes))
        .collect();
    if samples.iter().any(|sample| !sample.is_finite()) {
        return Err("non-finite audio sample".into());
    }
    Ok(samples)
}

#[cfg(windows)]
pub use windows::list_devices;
#[cfg(windows)]
pub use windows::run_probe;
#[cfg(windows)]
pub use windows::run_probe_with_mic;
#[cfg(windows)]
pub use windows::CaptureSession;

#[cfg(test)]
mod tests {
    use super::decode_samples;
    #[test]
    fn packet_decode_handles_silence_and_bounds() {
        let audio = [1.0_f32.to_le_bytes(), (-0.25_f32).to_le_bytes()].concat();
        assert_eq!(decode_samples(None, 2, true).unwrap(), [0.0, 0.0]);
        assert_eq!(
            decode_samples(Some(&audio), 2, false).unwrap(),
            [1.0, -0.25]
        );
        assert!(decode_samples(None, 2, false).is_err());
        assert!(decode_samples(Some(&audio[..4]), 2, false).is_err());
        assert!(decode_samples(None, 16_001, true).is_err());
        assert!(decode_samples(Some(&f32::NAN.to_le_bytes()), 1, false).is_err());
        assert!(decode_samples(None, 0, false).unwrap().is_empty());
    }
}
