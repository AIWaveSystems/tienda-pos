use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait Entity: Clone + Serialize + DeserializeOwned + Send + Sync + 'static {
    const COLLECTION: &'static str;

    fn id(&self) -> &str;
}
