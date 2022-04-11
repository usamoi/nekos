use crate::prelude::*;

pub trait MapUser: Send + Sync + Map + MapRead + MapWrite + MapIndex {}
