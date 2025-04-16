// extr_mod.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraOptions {
    pub enabled: bool,
    pub slim_mode: bool,
}

impl ExtraOptions {
    pub fn new() -> Self {
        Self {
            enabled: true,
            slim_mode: false,
        }
    }
}