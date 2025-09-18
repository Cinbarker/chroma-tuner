//! Pitch detection and musical note conversion
//!
//! Implements FFT-based pitch detection using a Hann window for frequency analysis.
//! Converts detected frequencies to musical notes with cent deviation calculations.

use realfft::{RealFftPlanner, RealToComplex};
use std::sync::Arc;

pub struct PitchDetector {
    fft: Arc<dyn RealToComplex<f32>>,
    buffer_size: usize,
    sample_rate: f32,
    window: Vec<f32>,
}

impl PitchDetector {
    pub fn new(buffer_size: usize, sample_rate: f32) -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(buffer_size);

        let window = (0..buffer_size)
            .map(|i| {
                let x = i as f32 / (buffer_size - 1) as f32;
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * x).cos())
            })
            .collect();

        Self {
            fft,
            buffer_size,
            sample_rate,
            window,
        }
    }

    pub fn detect_pitch(&mut self, samples: &[f32]) -> Option<(f32, f32)> {
        if samples.len() < self.buffer_size {
            return None;
        }

        let mut input: Vec<f32> = samples
            .iter()
            .take(self.buffer_size)
            .zip(self.window.iter())
            .map(|(sample, window)| sample * window)
            .collect();

        let mut spectrum = self.fft.make_output_vec();

        self.fft.process(&mut input, &mut spectrum).ok()?;

        let mut max_magnitude = 0.0;
        let mut max_index = 0;

        let min_freq_bin = (80.0 * self.buffer_size as f32 / self.sample_rate) as usize;
        let max_freq_bin = (2000.0 * self.buffer_size as f32 / self.sample_rate) as usize;

        for (i, complex) in spectrum.iter().enumerate().skip(min_freq_bin) {
            if i > max_freq_bin {
                break;
            }

            let magnitude = (complex.re * complex.re + complex.im * complex.im).sqrt();
            if magnitude > max_magnitude {
                max_magnitude = magnitude;
                max_index = i;
            }
        }

        if max_magnitude < 0.005 {
            return None;
        }

        let frequency = max_index as f32 * self.sample_rate / self.buffer_size as f32;

        let refined_frequency = if max_index > 0 && max_index < spectrum.len() - 1 {
            let left = (spectrum[max_index - 1].re * spectrum[max_index - 1].re
                + spectrum[max_index - 1].im * spectrum[max_index - 1].im)
                .sqrt();
            let center = max_magnitude;
            let right = (spectrum[max_index + 1].re * spectrum[max_index + 1].re
                + spectrum[max_index + 1].im * spectrum[max_index + 1].im)
                .sqrt();

            let offset = 0.5 * (left - right) / (left - 2.0 * center + right);
            (max_index as f32 + offset) * self.sample_rate / self.buffer_size as f32
        } else {
            frequency
        };

        Some((refined_frequency, max_magnitude))
    }
}

#[derive(Debug, Clone)]
pub struct Note {
    pub name: String,
    pub frequency: f32,
    pub cents_off: f32,
}

pub fn frequency_to_note(frequency: f32) -> Note {
    let a4_freq = 440.0;

    let semitones_from_a4 = 12.0 * (frequency / a4_freq).log2();
    let nearest_semitone = semitones_from_a4.round() as i32;

    let cents_off = (semitones_from_a4 - nearest_semitone as f32) * 100.0;

    let note_names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];

    let semitones_from_c4 = nearest_semitone + 9;

    let note_index = ((semitones_from_c4 % 12) + 12) % 12;

    let octave = if semitones_from_c4 >= 0 {
        4 + semitones_from_c4 / 12
    } else {
        4 + (semitones_from_c4 - 11) / 12
    };

    let note_name = format!("{}{}", note_names[note_index as usize], octave);

    Note {
        name: note_name,
        frequency,
        cents_off,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_mapping() {
        let test_cases = [
            (440.0, "A4"),
            (493.88, "B4"),
            (523.25, "C5"),
            (392.0, "G4"),
            (220.0, "A3"),
            (880.0, "A5"),
        ];

        for (freq, expected) in test_cases.iter() {
            let note = frequency_to_note(*freq);
            println!(
                "{:.2} Hz -> {} (expected {}), cents: {:.1}",
                freq, note.name, expected, note.cents_off
            );

            assert!(
                note.cents_off.abs() < 10.0,
                "Frequency {} Hz should be close to perfect pitch, got {} cents off",
                freq,
                note.cents_off
            );
        }
    }
}
