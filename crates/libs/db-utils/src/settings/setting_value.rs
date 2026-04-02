use std::fmt;

use serde::{Deserialize, Serialize};
use wasm_dbms_api::prelude::*;

/// A value to be used in the settings table.
#[derive(
    Debug,
    Clone,
    Encode,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    CustomDataType,
)]
#[type_tag = "setting_value"]
pub struct SettingValue {
    pub value: Value,
}

impl From<Value> for SettingValue {
    fn from(value: Value) -> Self {
        Self { value }
    }
}

impl Default for SettingValue {
    fn default() -> Self {
        Self { value: Value::Null }
    }
}

impl fmt::Display for SettingValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

impl DataType for SettingValue {}
