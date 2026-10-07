use std::env;
use std::time::Duration;

fn main() {
    #[cfg(windows)]
    {
        let mut args = env::args().skip(1);
        let seconds = args
            .next()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|seconds| (1..=7200).contains(seconds));
        let Some(seconds) = seconds else {
            eprintln!("usage: audio_probe <seconds 1..7200> [microphone-device-id]");
            std::process::exit(2);
        };
        match harness_lib::audio::run_probe_with_mic(
            Duration::from_secs(seconds),
            args.next().as_deref(),
        ) {
            Ok(report) => {
                for device in report.devices {
                    println!(
                        "device direction={} default={} id={} label={}",
                        device.direction, device.is_default, device.id, device.label
                    );
                }
                for stream in report.streams {
                    println!(
                        "stream source={} label={} peak={:.4} samples={} packets={} dropped={} errors={}",
                        stream.source,
                        stream.device_label,
                        stream.peak_level,
                        stream.samples,
                        stream.packets,
                        stream.dropped_packets,
                        stream.error_count
                    );
                }
            }
            Err(error) => {
                eprintln!("audio probe failed: {error}");
                std::process::exit(1);
            }
        }
    }

    #[cfg(not(windows))]
    {
        let _ = env::args();
        let _ = Duration::from_secs(0);
        eprintln!("audio probe requires Windows WASAPI");
    }
}
