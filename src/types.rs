use crate::errors::*;

use bytes::Bytes;
use serde::*;
use sodiumoxide::crypto::{generichash as hash, sign};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i128),
    Unt(u128),
    Text(String),
    Bytes(Bytes),
    Vec(Vec<Value>),
    Map(BTreeMap<Bytes, Value>),
}

// TODO: arithmetic
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub struct u4(u8);

impl u4 {
    pub fn split_le(n: u8) -> (u4, u4) {
        (u4(n % 16), u4(n / 16))
    }

    pub fn as_u8(self) -> u8 {
        self.0
    }

    pub fn combine(self, high: u4) -> u8 {
        self.0 + (high.0 << 4)
    }
}

// CHANGE THIS WHEN THINGS CHANGE
pub const PROTOCOL_VERSION: &'static str = "NO";

pub const HASH_BYTES: usize = 32;
new_type! { /// An untyped hashcode
    public Hash(HASH_BYTES);
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Copy)]
pub struct HashOf<T> {
    tag: std::marker::PhantomData<T>,
    hash: Hash,
}

impl<T: Serialize> HashOf<T> {
    pub fn hash_ser(t: &T) -> Result<HashOf<T>, Error> {
        let mut state = hash::State::new(HASH_BYTES, None).map_err(|_| HashingError)?;
        state
            .update(&serde_cbor::to_vec(t)?)
            .map_err(|_| HashingError)?;
        let hash = state.finalize().map_err(|_| HashingError)?;
        Ok(HashOf {
            tag: std::marker::PhantomData,
            hash: Hash::from_slice(hash.as_ref()).ok_or(HashingError)?,
        })
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct SigOf<T> {
    tag: std::marker::PhantomData<T>,
    protocol: String,
    key: sign::PublicKey,
    sig: sign::Signature,
}

impl<T: Serialize> SigOf<T> {
    pub fn sign_ser(sk: &sign::SecretKey, pk: sign::PublicKey, t: &T) -> Result<SigOf<T>, Error> {
        let sig = sign::sign_detached(&serde_cbor::to_vec(t)?, sk);
        Ok(SigOf {
            tag: std::marker::PhantomData,
            protocol: String::from(PROTOCOL_VERSION),
            key: pk,
            sig,
        })
    }

    pub fn check_ser(&self, t: &T) -> Result<bool, Error> {
        Ok(sign::verify_detached(
            &self.sig,
            &serde_cbor::to_vec(t)?,
            &self.key,
        ))
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct MultiSigned<T> {
    body: T,
    sigs: HashOf<Vec<SigOf<T>>>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct BlockBody {
    prev: Option<HashOf<Block>>,
    version: u128,
    /// The timestamp (in epoch seconds), rounded down to the nearest `Block`.
    timestamp: u128,
    tree: HashOf<CTNode>,
    options: HashOf<BTreeMap<Bytes, Value>>,
}

pub type SignedBlock = MultiSigned<BlockBody>;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct Block {
    block: SignedBlock,
    sig: SigOf<SignedBlock>,
}

pub type CTNode = MultiSigned<CTBody>;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct CTBody {
    last_main: Option<HashOf<Block>>,
    path: Vec<u4>,
    children: BTreeMap<Vec<u4>, HashOf<CTNode>>,
    data_tree: Option<HashOf<DataNode>>,
    new_action: Option<HashOf<Action>>,
    // TODO: Consider more bits
    // but probably not
    // maybe it will need to change some time in the next 1000 years
    prize: u128,
    new_nodes: u128,
    total_gas: u128,
    total_stake: u128,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct DataNode {
    children: BTreeMap<Vec<u4>, HashOf<DataNode>>,
    fields: Value,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct Action {
    last_main: HashOf<Block>,
    fee: u128,
    command: Bytes,
    args: Vec<Bytes>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub enum Recip {
    Account(Hash),
    Init(HashOf<Bytes>),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct SendInfo {
    last_main: HashOf<Block>,
    sender: Hash,
    to: Recip,
    amount: u128,
    msg: Value,
}
