use crate::{
    articles::{discovery::ArticleDiscoveryError, writer::WriterError},
    components::discovery::ComponentDiscoveryError,
    layouts::LayoutDiscoveryError,
    resources::discovery::ResourceDiscoveryError,
};

#[derive(thiserror::Error, Debug)]
pub enum GeneratorError {
    #[error(transparent)]
    ArticleDiscoveryError(#[from] ArticleDiscoveryError),

    #[error(transparent)]
    ComponentDiscoveryError(#[from] ComponentDiscoveryError),

    #[error(transparent)]
    LayoutDiscoveryError(#[from] LayoutDiscoveryError),

    #[error(transparent)]
    ResourceDiscoveryError(#[from] ResourceDiscoveryError),

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
