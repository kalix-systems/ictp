use crate::errors::*;
use crate::types::*;

use async_trait::async_trait;
use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait HashLookup {
    type Error;
    async fn lookup<T: DeserializeOwned + Send>(&self, of: HashOf<T>) -> Result<T, Self::Error>;
}

#[async_trait]
pub trait HashPut: HashLookup {
    async fn add<T: Serialize + Send + Sync>(&self, data: &T) -> Result<(), Self::Error>;
}

#[async_trait]
impl HashLookup for dashmap::DashMap<Hash, Bytes> {
    type Error = Error;
    async fn lookup<T: DeserializeOwned + Send>(&self, of: HashOf<T>) -> Result<T, Self::Error> {
        let bytes = self.async_get(of.hash()).await.ok_or(HashNotFound)?;
        Ok(serde_cbor::from_slice(&bytes)?)
    }
}

#[async_trait]
impl HashPut for dashmap::DashMap<Hash, Bytes> {
    async fn add<T: Serialize + Send + Sync>(
        &self,
        data: &T,
    ) -> Result<(), <Self as HashLookup>::Error> {
        let bytes = Bytes::from(serde_cbor::to_vec(data)?);
        let hash = Hash::calculate(&bytes)?;
        self.insert(hash, bytes);
        Ok(())
    }
}
