use sample::{Sample, Sampleable};


pub struct Track<S>
    where S: Sampleable
{
    /// The samples of this track.
    samples: Vec<Sample<S>>,
}


impl<S> Track<S>
    where S: Sampleable
{
    pub fn samples(&self) -> &Vec<Sample<S>> {
        &self.samples
    }
}
