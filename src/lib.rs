extern crate hound;
extern crate num;
extern crate pulse_simple;


mod sample;
pub use sample::{CHANNELS, Sample, Sampleable};

mod track;
pub use track::Track;


pub trait Player {}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
