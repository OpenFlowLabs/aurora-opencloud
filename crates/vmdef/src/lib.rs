use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Instructions {
    SetName(String),
    SetMemory(String),
    AddCPU(String),
    AddDevice {
        kind: DeviceKind,
        model: Option<String>,
        options: HashMap<String, String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeviceKind {
    Network,
    Disk,
    Special(String),
}
