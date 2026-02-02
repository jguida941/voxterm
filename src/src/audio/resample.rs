use super::TARGET_RATE;
use crate::log_debug;
use anyhow::{anyhow, Result};
#[cfg(feature = "high-quality-audio")]
use rubato::{InterpolationParameters, InterpolationType, Resampler, SincFixedIn, WindowFunction};
use std::cmp::Ordering as CmpOrdering;
use std::f32::consts::PI;
#[cfg(test)]
use std::sync::atomic::AtomicUsize;
#[cfg(any(test, feature = "high-quality-audio"))]
use std::sync::atomic::{AtomicBool, Ordering};

// Derived from 16 kHz target and practical ratio bounds (~0.01x .. 8x).
pub(super) const MIN_DEVICE_RATE: u32 = 2_000;
pub(super) const MAX_DEVICE_RATE: u32 = 1_600_000;
pub(super) const MIN_RESAMPLE_RATIO: f64 = TARGET_RATE as f64 / MAX_DEVICE_RATE as f64;
pub(super) const MAX_RESAMPLE_RATIO: f64 = TARGET_RATE as f64 / MIN_DEVICE_RATE as f64;
const MAX_DOWNSAMPLING_TAPS: usize = 129;

#[cfg(feature = "high-quality-audio")]
pub(super) static RESAMPLER_WARNING_SHOWN: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
pub(super) static RESAMPLE_FALLBACK_COUNT: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
pub(super) static RESAMPLE_WARN_COUNT: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
pub(super) static FORCE_RUBATO_ERROR: AtomicBool = AtomicBool::new(false);

pub(super) fn resample_to_target_rate(input: &[f32], device_rate: u32) -> Vec<f32> {
    // Guard rails
    if device_rate == 0 {
        return input.to_vec(); // avoid div-by-zero elsewhere
    }
    if input.is_empty() {
        return input.to_vec();
    }
    if device_rate == TARGET_RATE {
        return input.to_vec();
    }

    #[cfg(feature = "high-quality-audio")]
    {
        match resample_with_rubato(input, device_rate) {
            Ok(output) => output,
            Err(err) => {
                #[cfg(test)]
                RESAMPLE_FALLBACK_COUNT.fetch_add(1, Ordering::Relaxed);
                // CRITICAL: Use AcqRel ordering to prevent data race
                if !RESAMPLER_WARNING_SHOWN.swap(true, Ordering::AcqRel) {
                    #[cfg(test)]
                    RESAMPLE_WARN_COUNT.fetch_add(1, Ordering::Relaxed);
                    log_debug(&format!(
                        "high-quality resampler failed ({err}); falling back to basic path"
                    ));
                }
                basic_resample(input, device_rate)
            }
        }
    }

    #[cfg(not(feature = "high-quality-audio"))]
    {
        basic_resample(input, device_rate)
    }
}

#[cfg(feature = "high-quality-audio")]
pub(super) fn resample_with_rubato(input: &[f32], device_rate: u32) -> Result<Vec<f32>> {
    // Defensive early guard
    if device_rate == 0 {
        return Ok(input.to_vec());
    }
    if input.is_empty() {
        return Ok(input.to_vec());
    }
    if device_rate == TARGET_RATE {
        return Ok(input.to_vec());
    }

    if !(MIN_DEVICE_RATE..=MAX_DEVICE_RATE).contains(&device_rate) {
        return Err(anyhow!(
            "unsupported device sample rate {device_rate}Hz for resampling"
        ));
    }
    let ratio = TARGET_RATE as f64 / device_rate as f64;
    if !(MIN_RESAMPLE_RATIO..=MAX_RESAMPLE_RATIO).contains(&ratio) {
        return Err(anyhow!("invalid resample ratio {ratio}"));
    }

    #[cfg(test)]
    if FORCE_RUBATO_ERROR.swap(false, Ordering::Relaxed) {
        return Err(anyhow!("forced rubato error"));
    }

    let chunk = 256usize;
    let params = InterpolationParameters {
        sinc_len: 64,
        f_cutoff: 0.90, // safer cutoff
        interpolation: InterpolationType::Cubic,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    //           ratio,  drift, params, chunk_size, channels
    let mut rs = SincFixedIn::<f32>::new(ratio, 2.0, params, chunk, 1)
        .map_err(|e| anyhow!("failed to construct sinc resampler: {e:?}"))?;

    // pre-allocate
    let max_len = ((input.len() as f64) * MAX_RESAMPLE_RATIO).ceil() as usize;
    let mut expect = ((input.len() as f64) * ratio).round() as usize;
    expect = expect.clamp(1, max_len).saturating_add(8);
    let mut out = Vec::with_capacity(expect);

    let mut idx = 0usize;
    let mut seg = vec![0.0f32; chunk]; // reuse buffer
    while idx < input.len() {
        let end = (idx + chunk).min(input.len());
        if end == idx {
            return Err(anyhow!("resampler made no progress"));
        }
        let len = end - idx;
        let pad = input.get(end.wrapping_sub(1)).copied().unwrap_or(0.0);
        seg.fill(pad);
        seg[..len].copy_from_slice(&input[idx..end]);
        let produced = rs
            .process(std::slice::from_ref(&seg), None)
            .map_err(|e| anyhow!("resampler process failed: {e:?}"))?;
        out.extend_from_slice(&produced[0]);
        idx = end;
    }

    match out.len().cmp(&expect) {
        CmpOrdering::Greater => {
            out.truncate(expect);
        }
        CmpOrdering::Less => {
            out.resize(expect, *out.last().unwrap_or(&0.0));
        }
        CmpOrdering::Equal => {}
    }
    Ok(out)
}

pub(super) fn basic_resample(input: &[f32], device_rate: u32) -> Vec<f32> {
    // Guard rails
    if device_rate == 0 {
        return input.to_vec(); // avoid div-by-zero elsewhere
    }
    if input.is_empty() {
        return input.to_vec();
    }
    if !(MIN_DEVICE_RATE..=MAX_DEVICE_RATE).contains(&device_rate) {
        return input.to_vec();
    }

    // Ratio > 1 means upsampling, < 1 means downsampling.
    let mut ratio = TARGET_RATE as f32 / device_rate as f32;
    ratio = ratio.clamp(MIN_RESAMPLE_RATIO as f32, MAX_RESAMPLE_RATIO as f32);
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
pub(super) fn resample_linear(input: &[f32], ratio: f32) -> Vec<f32> {
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
        } else {
            let pad = input.last().copied().unwrap_or(0.0);
            output.push(pad);
        }
    }

    output
}

/// Pick a tap count based on the downsampling ratio so the FIR remains short for
/// near-equal sample rates and longer when we're collapsing 48 kHz into 16 kHz.
pub(super) fn downsampling_tap_count(device_rate: u32) -> usize {
    let decimation_ratio = device_rate as f32 / TARGET_RATE as f32;
    let mut taps = (decimation_ratio * 4.0).ceil().max(11.0) as usize;
    if taps.is_multiple_of(2) {
        taps += 1;
    }
    taps.min(MAX_DOWNSAMPLING_TAPS)
}

/// Basic FIR low-pass that tames frequencies above the target Nyquist before we
/// drop samples. Prevents high-frequency speech from aliasing when users have
/// 44.1/48 kHz microphones.
pub(super) fn low_pass_fir(input: &[f32], device_rate: u32, taps: usize) -> Vec<f32> {
    if input.is_empty() {
        return input.to_vec();
    }
    if taps <= 1 {
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

pub(super) fn convert_frame_to_target(
    frame: Vec<f32>,
    device_rate: u32,
    target_rate: u32,
    desired_len: usize,
) -> Vec<f32> {
    if device_rate == target_rate {
        return adjust_frame_length(frame, desired_len);
    }
    let resampled = resample_to_target_rate(&frame, device_rate);
    adjust_frame_length(resampled, desired_len)
}

pub(super) fn adjust_frame_length(mut data: Vec<f32>, desired: usize) -> Vec<f32> {
    match data.len().cmp(&desired) {
        CmpOrdering::Greater => {
            data.truncate(desired);
        }
        CmpOrdering::Less => {
            let pad = *data.last().unwrap_or(&0.0);
            data.resize(desired, pad);
        }
        CmpOrdering::Equal => {}
    }
    data
}

/// Build the normalized Hamming-windowed sinc taps used by the FIR filter.
pub(super) fn design_low_pass(normalized_cutoff: f32, taps: usize) -> Vec<f32> {
    let mut coeffs = Vec::with_capacity(taps);
    let m = (taps - 1) as f32;

    for n in 0..taps {
        let centered = n as f32 - m / 2.0;
        let x = 2.0 * PI * normalized_cutoff * centered;
        let sinc = if centered == 0.0 {
            2.0 * normalized_cutoff
        } else {
            (2.0 * normalized_cutoff * x.sin()) / x
        };
        let window = if taps <= 1 {
            1.0
        } else {
            0.54 - 0.46 * ((2.0 * PI * n as f32) / m).cos()
        };
        coeffs.push(sinc * window);
    }

    let sum: f32 = coeffs.iter().sum();
    if sum != 0.0 {
        for coeff in coeffs.iter_mut() {
            *coeff /= sum;
        }
    }

    coeffs
}
