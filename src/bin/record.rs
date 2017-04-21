extern crate hound;
extern crate pulse_simple;

extern crate pirus;

use std::io::{Read, Seek, Write};
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::sync::{Arc, Barrier};
use std::thread::{JoinHandle, spawn};

use hound::{WavReader, WavWriter};
use pulse_simple::{Playback, Record};

use pirus::{CHANNELS, Sample, Sampleable};


/// An opaque context used to control a background thread
type ThreadContext = (JoinHandle<()>, Sender<()>);


/// The name of the recorder for PulseAudio
static NAME: &'static str = "pirus";

/// The description of the recorder for PulseAudio
static DESCRIPTION: &'static str = "pirus";

/// The file played in the background
static INPUT_FILE: &'static str = "input.wav";

/// The output file
static OUTPUT_FILE: &'static str = "output.wav";


fn main() {
    let mut reader = WavReader::open(INPUT_FILE).unwrap();
    let mut writer = WavWriter::create(OUTPUT_FILE, reader.spec()).unwrap();

    run::<i16, _>(Sample::from_wav(&mut reader), &mut writer);
}


/// The actual main function.
///
/// This function takes the parsed input arguments.
fn run<S, W>(background_sample: Sample<S>, writer: &mut WavWriter<W>)
    where S: 'static + Sampleable,
          W: Seek + Write
{
    let sample_rate = background_sample.sample_rate();
    let sample_length = background_sample.data().len();

    // We use a barrier to make recording start when playback starts
    let barrier = Arc::new(Barrier::new(2));

    // Prepare the recorder and a buffer
    let recorder: Record<[S; CHANNELS]> =
        Record::new(NAME, DESCRIPTION, None, sample_rate);
    let mut sample: Sample<S> = Sample::silence(sample_length);

    // Start a thread for the background loop and wait for it to become ready
    let thread_ctx = background_loop_start::<S>(barrier.clone(),
                                                background_sample);
    barrier.wait();

    // Wait for a sample of non-silence
    wait_for_sound(&recorder, &mut sample);

    // Record until interrupted
    loop {
        for sample in sample.data() {
            for s in sample {
                writer.write_sample(*s).unwrap();
            }
        }
        if sample.is_silent() {
            break;
        }

        recorder.read(&mut sample.data_mut()[..]);
    }

    // Stop the background thread
    background_loop_stop(thread_ctx);
}


/// Waits for a non-silent sample.
///
/// # Arguments
/// *  `recorder` - The recorder to use to capture samples.
/// *  `sample` - A buffer to hold the final non-silent sample.
fn wait_for_sound<S>(recorder: &Record<[S; CHANNELS]>, sample: &mut Sample<S>)
    where S: 'static + Sampleable
{
    loop {
        recorder.read(&mut sample.data_mut()[..]);
        if !sample.is_silent() {
            break;
        }
    }
}


/// Spawns a thread to play a sample continuously in the background.
///
/// # Arguments
/// *  `barrier` - The barrier ensuring that the plaback and recording start at
///    the same time.
/// *  `sample` - The sample to play.
fn background_loop_start<S>(barrier: Arc<Barrier>,
                            sample: Sample<S>)
                            -> ThreadContext
    where S: 'static + Sampleable
{
    let (tx, rx) = channel();
    let thread = spawn(move || {
        let player =
            Playback::new(NAME, DESCRIPTION, None, sample.sample_rate());

        // Wait for the recording thread to start recording...
        barrier.wait();

        // Play until interrupted
        loop {
            player.write(sample.data());

            // If we manage to receive anything over the channel, or the channel
            // has been disconnected, break
            match rx.try_recv() {
                Ok(_) |
                Err(TryRecvError::Disconnected) => break,

                Err(TryRecvError::Empty) => {}
            }
        }
    });

    (thread, tx)
}


/// Stops the background thread and waits for it to terminate.
///
/// # Arguments
/// *  `(thread, tx)` - The thread context returned when starting the thread.
fn background_loop_stop((thread, tx): ThreadContext) {
    tx.send(()).unwrap();
    thread.join().unwrap();
}
