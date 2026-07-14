use rustfft::{Fft, FftPlanner, num_complex::Complex, num_traits::Zero};

use crate::enums::{VISUALIZER_FFT_SIZE, VISUALIZER_MIN_HZ};

pub struct VisualizerProcessor {
    fft: std::sync::Arc<dyn Fft<f32>>,
    fft_buffer: Vec<Complex<f32>>,
    smoothed_bars: Vec<f32>,
    adaptive_level: f32,
    fast_energy: f32,
    slow_energy: f32,
}

impl VisualizerProcessor {
   pub fn new(bar_count: usize) -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(VISUALIZER_FFT_SIZE);
        Self {
            fft,
            fft_buffer: vec![Complex::zero(); VISUALIZER_FFT_SIZE],
            smoothed_bars: vec![0.0; bar_count.max(1)],
            adaptive_level: 0.08,
            fast_energy: 0.0,
            slow_energy: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.smoothed_bars.fill(0.0);
        self.adaptive_level = 0.08;
        self.fast_energy = 0.0;
        self.slow_energy = 0.0;
    }

    pub fn ensure_bar_count(&mut self, count: usize) {
        let count = count.max(1);
        if self.smoothed_bars.len() != count {
            self.smoothed_bars = vec![0.0; count];
            self.adaptive_level = 0.08;
        }
    }

    pub fn decay_only(&mut self, out: &mut [f32]) -> i32 {
        self.ensure_bar_count(out.len());
        for value in &mut self.smoothed_bars {
            *value *= 0.93;
        }
        self.fast_energy *= 0.90;
        self.slow_energy *= 0.97;
        for (index, value) in out.iter_mut().enumerate() {
            *value = self.smoothed_bars[index];
        }
        out.len() as i32
    }

    pub fn compute(&mut self, samples: &[f32], channels: usize, sample_rate: u32, out: &mut [f32], playing: bool) -> i32 {
        if out.is_empty() {
            return 0;
        }

        self.ensure_bar_count(out.len());

        if !playing || samples.is_empty() || channels == 0 || sample_rate == 0 {
            return self.decay_only(out);
        }

        let frame_count = samples.len() / channels;
        if frame_count == 0 {
            return self.decay_only(out);
        }

        let window_frames = VISUALIZER_FFT_SIZE.min(frame_count);
        let start_frame = frame_count.saturating_sub(window_frames);

        self.fft_buffer.fill(Complex::zero());

        let mut rms_acc = 0.0f32;
        for index in 0..window_frames {
            let src_frame = start_frame + index;
            let mut mixed = 0.0f32;
            for ch in 0..channels {
                mixed += samples[src_frame * channels + ch];
            }
            let mono = mixed / channels as f32;
            rms_acc += mono * mono;

            let target = VISUALIZER_FFT_SIZE - window_frames + index;
            self.fft_buffer[target].re = mono * hann_window(index, window_frames);
        }

        self.fft.process(&mut self.fft_buffer);

        let half = VISUALIZER_FFT_SIZE / 2;
        let mut magnitudes = vec![0.0f32; half];
        let norm = window_frames.max(1) as f32;
        for (bin, value) in self.fft_buffer.iter().take(half).enumerate() {
            magnitudes[bin] = (value.re * value.re + value.im * value.im).sqrt() / norm;
        }

        let rms = (rms_acc / window_frames.max(1) as f32).sqrt();
        self.fast_energy = self.fast_energy * 0.50 + rms * 0.50;
        self.slow_energy = self.slow_energy * 0.97 + rms * 0.03;
        let beat = ((self.fast_energy - self.slow_energy) * 11.0).clamp(0.0, 1.0);

        let min_hz = VISUALIZER_MIN_HZ;
        let max_hz = ((sample_rate as f32) * 0.46).max(min_hz + 1.0);
        let bar_count = out.len();
        let mut raw_bars = vec![0.0f32; bar_count];

        for bar in 0..bar_count {
            let t0 = bar as f32 / bar_count as f32;
            let t1 = (bar + 1) as f32 / bar_count as f32;
            let f0 = log_interp(min_hz, max_hz, t0);
            let f1 = log_interp(min_hz, max_hz, t1);

            let b0 = hz_to_bin(f0, sample_rate, VISUALIZER_FFT_SIZE).max(1);
            let b1 = hz_to_bin(f1, sample_rate, VISUALIZER_FFT_SIZE).max(b0 + 1);
            let end = b1.min(magnitudes.len());
            let start = b0.min(end.saturating_sub(1));

            let mut energy = 0.0f32;
            let mut count = 0usize;
            for value in &magnitudes[start..end] {
                energy += *value;
                count += 1;
            }

            let mut raw = if count > 0 { energy / count as f32 } else { 0.0 };
            raw *= 1.0 + beat * 0.55 * (1.0 - t0);
            raw_bars[bar] = raw;
        }

        let frame_peak = raw_bars.iter().copied().fold(0.0f32, f32::max);
        self.adaptive_level = self.adaptive_level * 0.95 + frame_peak.max(0.0001) * 0.05;
        let level = self.adaptive_level.max(0.0001);

        let mut spatial = vec![0.0f32; bar_count];
        for index in 0..bar_count {
            let left = if index > 0 { raw_bars[index - 1] } else { raw_bars[index] };
            let center = raw_bars[index];
            let right = if index + 1 < bar_count {
                raw_bars[index + 1]
            } else {
                raw_bars[index]
            };
            spatial[index] = left * 0.20 + center * 0.60 + right * 0.20;
        }

        for (index, raw) in spatial.iter().enumerate() {
            let mut target = (raw / level).clamp(0.0, 2.0);
            target = target.powf(0.78) * 0.70;
            target = target.clamp(0.0, 1.0);

            let current = self.smoothed_bars[index];
            let alpha = if target > current { 0.34 } else { 0.08 };
            self.smoothed_bars[index] = current + (target - current) * alpha;
            out[index] = self.smoothed_bars[index];
        }

        bar_count as i32
    }
}

fn hann_window(index: usize, len: usize) -> f32 {
    if len <= 1 {
        return 1.0;
    }
    let x = index as f32 / (len - 1) as f32;
    (0.5 - 0.5 * (2.0 * std::f32::consts::PI * x).cos()).clamp(0.0, 1.0)
}

fn log_interp(min: f32, max: f32, t: f32) -> f32 {
    min * (max / min).powf(t.clamp(0.0, 1.0))
}

fn hz_to_bin(freq: f32, sample_rate: u32, fft_size: usize) -> usize {
    let nyquist = sample_rate as f32 / 2.0;
    let clamped = freq.clamp(0.0, nyquist);
    ((clamped / sample_rate as f32) * fft_size as f32) as usize
}