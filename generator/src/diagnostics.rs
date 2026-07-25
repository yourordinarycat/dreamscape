use crate::articles::writer::WriterError;

#[derive(thiserror::Error, Debug)]
pub enum GeneratorError {
    #[error(transparent)]
    WriterError(#[from] WriterError),
}

#[derive(Debug, Default)]
pub struct Diagnostics {
    pub errors: Vec<GeneratorError>,
}

impl Diagnostics {
    pub fn failed(&self) -> bool {
        self.errors.len() > 0
    }

    pub fn merge(&mut self, other: Diagnostics) {
        self.errors.extend(other.errors.into_iter());
    }
}