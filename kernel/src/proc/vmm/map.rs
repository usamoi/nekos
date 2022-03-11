use crate::prelude::*;

pub trait MapProc: Send + Sync + Map + RandomRead + RandomWrite + MapIndex {}
