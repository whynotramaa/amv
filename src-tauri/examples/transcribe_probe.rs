// Explicit developer probe; transcripts are printed only when this command is run.
use anyhow::{bail, Context, Result};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        bail!("usage: transcribe_probe <model-directory> <16kHz-mono-PCM16-WAV>");
    }
    let mut reader = hound::WavReader::open(&args[1]).context("open probe WAV")?;
    let spec = reader.spec();
    if spec.sample_rate != 16000
        || spec.channels != 1
        || spec.bits_per_sample != 16
        || spec.sample_format != hound::SampleFormat::Int
        || reader.duration() > 15 * 16000
    {
        bail!("Probe WAV must be mono PCM16 at 16kHz and at most 15 seconds");
    }
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.map(|s| f32::from(s) / 32768.0))
        .collect::<Result<_, _>>()?;
    let context = harness_lib::speech::load_recognizer(std::path::Path::new(&args[0]))
        .map_err(anyhow::Error::msg)?;
    let started = std::time::Instant::now();
    let text = harness_lib::speech::decode(&context, &samples).map_err(anyhow::Error::msg)?;
    println!(
        "audio_seconds={:.2} decode_seconds={:.2}\n{text}",
        samples.len() as f64 / 16000.0,
        started.elapsed().as_secs_f64()
    );
    Ok(())
}
