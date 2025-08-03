use std::{
    fmt::Display,
    future::Future,
    marker::PhantomData,
    path::{Path, PathBuf},
    pin::Pin,
};

use crate::data_structures::{
    graphs::{EdgeData, Graph},
    Index,
};

pub mod osm;

pub struct DataSet {
    pub download: fn() -> Pin<Box<dyn Future<Output = ()>>>,
}

#[derive(Debug)]
pub enum DataSetReaderError {
    TooLarge(u32, u32),
    InputError(String),
    ConsistencyError(String),
    Other(String),
}

impl Display for DataSetReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSetReaderError::TooLarge(size, limit) => {
                write!(
                    f,
                    "Dataset size {size} is larger than the given limit {limit}."
                )
            }
            DataSetReaderError::InputError(s) => write!(f, "{s}"),
            DataSetReaderError::ConsistencyError(s) => write!(f, "{s}"),
            DataSetReaderError::Other(s) => write!(f, "{s}"),
        }
    }
}

pub trait GraphReader<G, I, ED, O>
where
    G: Graph<I, ED>,
    I: Index,
    ED: EdgeData,
{
    fn read_from(path: impl AsRef<Path>, options: &O) -> Result<G, DataSetReaderError>;
}

pub struct GraphSetIterator<R, G, I, ED, O>
where
    G: Graph<I, ED>,
    I: Index,
    ED: EdgeData,
    R: GraphReader<G, I, ED, O>,
{
    path_iterator: Box<dyn Iterator<Item = PathBuf>>,
    options: O,
    _phantom: PhantomData<(R, G, I, ED)>,
}

impl<R, G, I, ED, O> GraphSetIterator<R, G, I, ED, O>
where
    G: Graph<I, ED>,
    I: Index,
    ED: EdgeData,
    R: GraphReader<G, I, ED, O>,
{
    pub fn new(paths: impl IntoIterator<Item = PathBuf>, sort: bool, options: O) -> Self {
        let mut paths: Vec<PathBuf> = paths.into_iter().collect();
        if sort {
            // use file size as instance size approximation; go from small to large
            paths.sort_by_cached_key(|e| e.metadata().map(|m| m.len()).unwrap_or(0));
        }
        let path_iterator = Box::new(paths.into_iter());
        GraphSetIterator {
            path_iterator,
            options,
            _phantom: PhantomData,
        }
    }
}

impl<R, G, I, ED, O> Iterator for GraphSetIterator<R, G, I, ED, O>
where
    G: Graph<I, ED>,
    I: Index,
    ED: EdgeData,
    R: GraphReader<G, I, ED, O>,
{
    type Item = GraphSetEntry<G, I, ED>;

    fn next(&mut self) -> Option<Self::Item> {
        for path in self.path_iterator.by_ref() {
            log::info!("Read graph from {}.", path.display());
            let graph = match R::read_from(&path, &self.options) {
                Ok(g) => g,
                Err(why) => match why {
                    DataSetReaderError::TooLarge(..) => {
                        log::info!("{why}");
                        continue;
                    }
                    _ => {
                        log::error!("{why}");
                        continue;
                    }
                },
            };
            return Some(GraphSetEntry {
                graph,
                path,
                _phantom: PhantomData,
            });
        }
        None
    }
}

pub struct GraphSetEntry<G, I, ED>
where
    G: Graph<I, ED>,
    I: Index,
    ED: EdgeData,
{
    pub graph: G,
    pub path: PathBuf,
    _phantom: PhantomData<(I, ED)>,
}
