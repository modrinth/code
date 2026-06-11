#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct InstanceId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ContentSetId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ContentEntryId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct InstanceFileId(pub String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstanceRef<'a> {
    Id(&'a str),
    Path(&'a str),
}
