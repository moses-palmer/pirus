use std::io::Read;

use hound;
use num;
use pulse_simple;

/// The number of channels for a sample.
pub const CHANNELS: usize = 1;

/// The number of samples in an envelope when applying a low pass filter on a
/// sample to determine whether it is silent.
pub const ENVELOPE_SIZE: usize = 4096;


pub trait Sampleable
    : hound::Sample + num::ToPrimitive + num::Signed + pulse_simple::Sampleable + Copy + Default + Send {
}

impl Sampleable for i16 {}

impl Sampleable for i32 {}

impl Sampleable for f32 {}


pub struct Sample<S>
    where S: Sampleable
{
    data: Vec<[S; CHANNELS]>,
    sample_rate: u32,
}


impl<S> Sample<S>
    where S: Sampleable
{
    /// Create a sample consisting of only silence.
    ///
    /// # Arguments
    /// * `lenght` - The length, in individual samples, of the sample.
    pub fn silence(length: usize) -> Sample<S> {
        let mut buffer = Vec::with_capacity(length);
        for _ in 0..length {
            buffer.push([S::default()]);
        }

        Sample {
            data: buffer,
            sample_rate: 48000,
        }
    }

    /// Creates a sample from a WAV file.
    ///
    /// The length is taken from the WAV header.
    ///
    /// # Arguments
    /// *  `wav` - The WAV file reader.
    pub fn from_wav<W>(wav: &mut hound::WavReader<W>) -> Sample<S>
        where S: Sampleable,
              W: Read
    {
        let buffer: Vec<[S; CHANNELS]> =
            wav.samples().map(|s| [s.unwrap()]).collect();

        Sample {
            data: buffer,
            sample_rate: wav.spec().sample_rate,
        }
    }

    /// The actual sample data.
    pub fn data(&self) -> &[[S; CHANNELS]] {
        &self.data[..]
    }

    /// The mutable sample data.
    pub fn data_mut(&mut self) -> &mut [[S; CHANNELS]] {
        &mut self.data[..]
    }

    /// The sample rate of this sample.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn is_silent(&self) -> bool {
        // Apply a crude low pass filter
        let lowpass = self.data
            .chunks(ENVELOPE_SIZE)
            .map(|envelope| {
                envelope.iter()
                    .flat_map(|s| s.iter())
                    .map(|i| {
                        i.abs().to_f32().unwrap_or(0f32) / ENVELOPE_SIZE as f32
                    })
                    .sum()
            })
            .collect::<Vec<f32>>();

        // Calculate the mean of the lowpass values
        let mean = lowpass.iter().sum::<f32>() / lowpass.len() as f32;
        let std_dev = lowpass.iter()
            .map(|v| (v - mean) * (v - mean))
            .sum::<f32>()
            .sqrt() / (lowpass.len() - 1) as f32;

        println!("std_dev = {}", std_dev);

        true
    }
}
