use crate::types::*;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait HashLookup {
    type Error;
    async fn lookup<T: DeserializeOwned>(&self, of: HashOf<T>) -> Result<T, Self::Error>;
}

#[async_trait]
pub trait HashPut: HashLookup {
    async fn add<T: Serialize>(&self, data: &T) -> Result<(), Self::Error>;
}
