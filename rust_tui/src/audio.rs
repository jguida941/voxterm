use crate::log_debug;
use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
#[cfg(feature = "high-quality-audio")]
use rubato::{InterpolationParameters, InterpolationType, Resampler, SincFixedIn, WindowFunction};
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Target format for transcription (mono channel, 16 kHz sample rate)
/// The Whisper model requires mono audio at 16 kHz for accurate transcription
pub const TARGET_RATE: u32 = 16_000;
pub const TARGET_CHANNELS: u32 = 1;
const SAMPLE_RATE: u32 = TARGET_RATE;

/// Wraps the system input device abstraction so the rest of the app can ask for
/// "speech-ready" samples without touching cpal or thinking about sample rates.
pub struct Recorder {
    device: cpal::Device,
}

impl Recorder {
    /// List microphone names so the CLI can expose a human-friendly selector.
    pub fn list_devices() -> Result<Vec<String>> {
        let host = cpal::default_host();
        let devices = host.input_devices().context("no input devices available")?;
        let mut names = Vec::new();
        for device in devices {
            if let Ok(name) = device.name() {
                names.push(name);
            }
        }
        Ok(names)
    }

    /// Create a recorder, optionally forcing a specific device so users can pick
    /// the right microphone when a laptop exposes multiple inputs.
    pub fn new(preferred_device: Option<&str>) -> Result<Self> {
        let host = cpal::default_host();
        let device = match preferred_device {
            Some(name) => {
                let mut devices = host.input_devices().context("no input devices available")?;
                devices
                    .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                    .ok_or_else(|| anyhow!("input device '{name}' not found"))?
            }
            None => host
                .default_input_device()
                .context("no default input device available")?,
        };
        Ok(Self { device })
    }

    /// Record audio for `seconds`, normalize the incoming format, and return
    /// 16 kHz mono data that Whisper can consume directly.
    pub fn record(&self, seconds: u64) -> Result<Vec<f32>> {
        // Get the device's default config so we know the native format and channel count.
        let default_config = self.device.default_input_config()?;
        let format = default_config.sample_format();
        let device_config: StreamConfig = default_config.clone().into();
        let device_sample_rate = device_config.sample_rate.0;
        let channels = usize::from(device_config.channels.max(1));
        let device_name = self
            .device
            .name()
            .unwrap_or_else(|_| "unknown input device".to_string());

        log_debug(&format!(
            "Recorder config: format={format:?} sample_rate={device_sample_rate}Hz channels={channels}"
        ));

        // cpal delivers samples on a callback thread; collect them in a shared
        // buffer so we can keep ownership on the caller side.
        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let buffer_clone = buffer.clone();

        // Keep the error callback quiet in the UI and mirror issues into the log.
        let err_fn = |err| log_debug(&format!("audio_stream_error: {err}"));

        // Convert every supported sample type to f32 up front so the rest of the
        // pipeline can stay format-agnostic.
        let stream = match format {
            SampleFormat::F32 => self.device.build_input_stream(
                &device_config,
                move |data: &[f32], _| {
                    if let Ok(mut buf) = buffer_clone.lock() {
                        append_downmixed_samples(&mut buf, data, channels, |sample| sample);
                    }
                },
                err_fn,
                None,
            )?,
            SampleFormat::I16 => self.device.build_input_stream(
                &device_config,
                move |data: &[i16], _| {
                    if let Ok(mut buf) = buffer_clone.lock() {
                        append_downmixed_samples(&mut buf, data, channels, |sample| {
                            sample as f32 / 32_768.0_f32
                        });
                    }
                },
                err_fn,
                None,
            )?,
            SampleFormat::U16 => self.device.build_input_stream(
                &device_config,
                move |data: &[u16], _| {
                    if let Ok(mut buf) = buffer_clone.lock() {
                        append_downmixed_samples(&mut buf, data, channels, |sample| {
                            (sample as f32 - 32_768.0_f32) / 32_768.0_f32
                        });
                    }
                },
                err_fn,
                None,
            )?,
            other => return Err(anyhow!("unsupported sample format: {other:?}")),
        };

        stream.play()?;
        std::thread::sleep(Duration::from_secs(seconds));
        if let Err(err) = stream.pause() {
            log_debug(&format!("failed to pause audio stream: {err}"));
        }
        drop(stream);

        let samples = buffer.lock().unwrap();

        if samples.is_empty() {
            return Err(anyhow!(
                "no samples captured from '{device_name}'; check microphone permissions and availability"
            ));
        }

        // Transcription assumes 16 kHz mono, so resample if the hardware rate differs.
        let processed = resample_to_target_rate(&samples, device_sample_rate);
        Ok(processed)
    }
}

/// Downmix multi-channel input to mono while applying the provided converter so
/// Whisper receives a single channel regardless of the microphone layout.
fn append_downmixed_samples<T, F>(buf: &mut Vec<f32>, data: &[T], channels: usize, mut convert: F)
where
    T: Copy,
    F: FnMut(T) -> f32,
{
    if channels <= 1 {
        buf.extend(data.iter().copied().map(&mut convert));
        return;
    }

    // Average each interleaved frame to produce a mono representation.
    let mut acc = 0.0f32;
    let mut count = 0usize;
    for sample in data.iter().copied() {
        acc += convert(sample);
        count += 1;
        if count == channels {
            buf.push(acc / channels as f32);
            acc = 0.0;
            count = 0;
        }
    }
    if count > 0 {
        buf.push(acc / count as f32);
    }
}

#[cfg(feature = "high-quality-audio")]
fn resample_to_target_rate(input: &[f32], device_rate: u32) -> Vec<f32> {
    // Guard rails
    if device_rate == 0 {
        return input.to_vec(); // avoid div-by-zero elsewhere
    }
    if input.is_empty() || device_rate == TARGET_RATE {
        return input.to_vec();
    }

    match resample_with_rubato(input, device_rate) {
        Ok(output) => output,
        Err(err) => {
            log_debug(&format!(
                "high-quality resampler failed ({err}); falling back to basic path"
            ));
            basic_resample(input, device_rate)
        }
    }
}

#[cfg(feature = "high-quality-audio")]
fn resample_with_rubato(input: &[f32], device_rate: u32) -> Result<Vec<f32>> {
    // Defensive early guard
    if device_rate == 0 || input.is_empty() || device_rate == TARGET_RATE {
        return Ok(input.to_vec());
    }

    let ratio = TARGET_RATE as f64 / device_rate as f64;
    let chunk = 256usize;
    let params = InterpolationParameters {
        sinc_len: 64,
        f_cutoff: 0.90, // safer cutoff
        interpolation: InterpolationType::Cubic,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    //           ratio,  drift, params, channels, chunk
    let mut rs = SincFixedIn::<f32>::new(ratio, 2.0, params, 1, chunk)
        .map_err(|e| anyhow!("failed to construct sinc resampler: {e:?}"))?;

    // pre-allocate
    let expect = ((input.len() as u64) * TARGET_RATE as u64 / device_rate as u64) as usize + 8;
    let mut out = Vec::with_capacity(expect);

    let mut idx = 0usize;
    let mut seg = vec![0.0f32; chunk]; // reuse buffer
    while idx < input.len() {
        let end = (idx + chunk).min(input.len());
        let len = end - idx;
        seg[..len].copy_from_slice(&input[idx..end]);
        if len < chunk {
            let pad = seg.get(len.wrapping_sub(1)).copied().unwrap_or(0.0);
            for s in &mut seg[len..] {
                *s = pad;
            }
        }
        let produced = rs
            .process(std::slice::from_ref(&seg), None)
            .map_err(|e| anyhow!("resampler process failed: {e:?}"))?;
        out.extend_from_slice(&produced[0]);
        idx = end;
    }

    if out.len() > expect {
        out.truncate(expect);
    } else if out.len() < expect {
        out.resize(expect, *out.last().unwrap_or(&0.0));
    }
    Ok(out)
}

#[cfg(not(feature = "high-quality-audio"))]
fn resample_to_target_rate(input: &[f32], device_rate: u32) -> Vec<f32> {
    basic_resample(input, device_rate)
}

fn basic_resample(input: &[f32], device_rate: u32) -> Vec<f32> {
    // Guard rails
    if device_rate == 0 {
        return input.to_vec(); // avoid div-by-zero elsewhere
    }
    if input.is_empty() || device_rate == TARGET_RATE {
        return input.to_vec();
    }

    // Ratio > 1 means upsampling, < 1 means downsampling.
    let ratio = TARGET_RATE as f32 / device_rate as f32;
    let filtered = if device_rate > TARGET_RATE {
        // When decimating we run a small FIR low-pass to avoid aliasing.
        let taps = downsampling_tap_count(device_rate);
        low_pass_fir(input, device_rate, taps)
    } else {
        input.to_vec()
    };
    resample_linear(&filtered, ratio)
}

/// Lightweight linear resampler used after optional filtering; works well for
/// short speech snippets where phase accuracy matters less than latency.
fn resample_linear(input: &[f32], ratio: f32) -> Vec<f32> {
    let input_len = input.len();
    let output_len = (input_len as f32 * ratio).round() as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f32 / ratio;
        let idx = src_idx.floor() as usize;
        let frac = src_idx - idx as f32;

        if idx + 1 < input_len {
            let sample = input[idx] * (1.0 - frac) + input[idx + 1] * frac;
            output.push(sample);
        } else if idx < input_len {
            output.push(input[idx]);
        } else {
            let pad = input.last().copied().unwrap_or(0.0);
            output.push(pad);
        }
    }

    output
}

/// Pick a tap count based on the downsampling ratio so the FIR remains short for
/// near-equal sample rates and longer when we're collapsing 48 kHz into 16 kHz.
fn downsampling_tap_count(device_rate: u32) -> usize {
    let decimation_ratio = device_rate as f32 / TARGET_RATE as f32;
    let mut taps = (decimation_ratio * 4.0).ceil().max(11.0) as usize;
    if taps % 2 == 0 {
        taps += 1;
    }
    taps
}

/// Basic FIR low-pass that tames frequencies above the target Nyquist before we
/// drop samples. Prevents high-frequency speech from aliasing when users have
/// 44.1/48 kHz microphones.
fn low_pass_fir(input: &[f32], device_rate: u32, taps: usize) -> Vec<f32> {
    if input.is_empty() || taps <= 1 {
        return input.to_vec();
    }

    let normalized_cutoff = (TARGET_RATE as f32 * 0.5 / device_rate as f32).min(0.499);
    let coeffs = design_low_pass(normalized_cutoff, taps);
    let half = taps / 2;
    let mut output = Vec::with_capacity(input.len());

    for n in 0..input.len() {
        let mut acc = 0.0;
        for (k, coeff) in coeffs.iter().enumerate() {
            // Use saturating arithmetic to prevent underflow
            if let Some(idx) = n.checked_add(k).and_then(|sum| sum.checked_sub(half)) {
                if let Some(sample) = input.get(idx) {
                    acc += *sample * coeff;
                }
            }
        }
        output.push(acc);
    }

    output
}

/// Build the normalized Hamming-windowed sinc taps used by the FIR filter.
fn design_low_pass(normalized_cutoff: f32, taps: usize) -> Vec<f32> {
    let mut coeffs = Vec::with_capacity(taps);
    let m = (taps - 1) as f32;

    for n in 0..taps {
        let centered = n as f32 - m / 2.0;
        let x = 2.0 * PI * normalized_cutoff * centered;
        let sinc = if x.abs() < 1e-6 {
            2.0 * normalized_cutoff
        } else {
            (2.0 * normalized_cutoff * x.sin()) / x
        };
        let window = if m.abs() < f32::EPSILON {
            1.0
        } else {
            0.54 - 0.46 * ((2.0 * PI * n as f32) / m).cos()
        };
        coeffs.push(sinc * window);
    }

    let sum: f32 = coeffs.iter().sum();
    if sum.abs() > f32::EPSILON {
        for coeff in coeffs.iter_mut() {
            *coeff /= sum;
        }
    }

    coeffs
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn downmixes_multi_channel_audio() {
        let mut buf = Vec::new();
        let samples = [1.0f32, -1.0, 0.5, 0.5];
        append_downmixed_samples(&mut buf, &samples, 2, |sample| sample);
        assert_eq!(buf, vec![0.0, 0.5]);
    }

    #[test]
    fn preserves_single_channel_audio() {
        let mut buf = Vec::new();
        let samples = [0.1f32, 0.2, 0.3];
        append_downmixed_samples(&mut buf, &samples, 1, |sample| sample);
        assert_eq!(buf, samples);
    }

    #[test]
    fn resample_linear_scales_length() {
        let input = vec![0.0f32, 1.0, 2.0, 3.0];
        let result = resample_linear(&input, 0.5);
        assert!(result.len() < input.len());
        assert!((result.first().copied().unwrap_or_default() - 0.0).abs() < 1e-6);
    }

    #[cfg(not(feature = "high-quality-audio"))]
    #[test]
    fn resample_to_target_rate_adjusts_length() {
        let input = vec![0.0, 1.0, 0.5, -0.5, -1.0, 0.0];
        let result = resample_to_target_rate(&input, 48_000);
        assert!(result.len() < input.len());
    }

    #[cfg(feature = "high-quality-audio")]
    #[test]
    fn rubato_resampler_matches_expected_length() {
        let input: Vec<f32> = (0..960).map(|i| (i as f32 * 0.01).sin()).collect();
        let result = resample_to_target_rate(&input, 48_000);
        let expected = (input.len() as f64 * 16_000f64 / 48_000f64).round() as usize;
        let diff = (result.len() as isize - expected as isize).abs();
        assert!(
            diff <= 2,
            "expected {expected} samples, got {}, diff {diff}",
            result.len()
        );
    }

    #[cfg(feature = "high-quality-audio")]
    #[test]
    fn rubato_resampler_handles_upsample() {
        let input: Vec<f32> = (0..160).map(|i| (i as f32 * 0.05).cos()).collect();
        let result = resample_to_target_rate(&input, 8_000);
        let expected = (input.len() as f64 * 16_000f64 / 8_000f64).round() as usize;
        let diff = (result.len() as isize - expected as isize).abs();
        assert!(
            diff <= 2,
            "expected {expected} samples, got {}, diff {diff}",
            result.len()
        );
    }

    #[cfg(feature = "high-quality-audio")]
    #[test]
    fn rubato_rejects_aliasing_energy() {
        let signal = multi_tone_signal(&[(6_000.0, 1.0), (12_000.0, 1.0)], 48_000, 0.1);
        let resampled = resample_to_target_rate(&signal, 48_000);
        let wanted = goertzel_power(&resampled, SAMPLE_RATE, 6_000.0);
        let alias = goertzel_power(&resampled, SAMPLE_RATE, 4_000.0);
        assert!(wanted > 0.1, "wanted tone vanished (power={wanted})");
        assert!(
            alias < 0.01 * wanted,
            "alias not suppressed enough (wanted={wanted}, alias={alias})"
        );
    }

    #[cfg(not(feature = "high-quality-audio"))]
    #[test]
    fn fir_resampler_reduces_alias_vs_naive() {
        let signal = multi_tone_signal(&[(6_000.0, 1.0), (12_000.0, 1.0)], 48_000, 0.1);
        let filtered = resample_to_target_rate(&signal, 48_000);
        let ratio = SAMPLE_RATE as f32 / 48_000f32;
        let naive = resample_linear(&signal, ratio);
        let alias_filtered = goertzel_power(&filtered, SAMPLE_RATE, 4_000.0);
        let alias_naive = goertzel_power(&naive, SAMPLE_RATE, 4_000.0);
        assert!(
            alias_filtered < alias_naive * 0.6,
            "FIR path failed to reduce aliasing (filtered={alias_filtered}, naive={alias_naive})"
        );
    }

    fn multi_tone_signal(tones: &[(f32, f32)], sample_rate: u32, seconds: f32) -> Vec<f32> {
        let total_samples = (sample_rate as f32 * seconds) as usize;
        (0..total_samples)
            .map(|n| {
                tones.iter().fold(0.0, |acc, (freq, amp)| {
                    acc + amp * (2.0 * PI * freq * n as f32 / sample_rate as f32).sin()
                })
            })
            .collect()
    }

    fn goertzel_power(samples: &[f32], sample_rate: u32, target_hz: f32) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let len = samples.len() as f32;
        let normalized_freq = target_hz / sample_rate as f32;
        let omega = 2.0 * PI * normalized_freq;
        let coeff = 2.0 * omega.cos();
        let mut q1 = 0.0;
        let mut q2 = 0.0;
        for &sample in samples {
            let q0 = coeff * q1 - q2 + sample;
            q2 = q1;
            q1 = q0;
        }
        let power = q1 * q1 + q2 * q2 - coeff * q1 * q2;
        (power / len).max(0.0)
    }
}
