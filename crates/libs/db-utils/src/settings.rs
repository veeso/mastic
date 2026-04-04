//! Settings table utilities

mod setting_value;

use wasm_dbms::schema::DatabaseSchema;
use wasm_dbms::{DbmsContext, WasmDbmsDatabase};
use wasm_dbms_api::prelude::*;
use wasm_dbms_memory::{AccessControl, MemoryProvider};

pub use self::setting_value::SettingValue;

const COLUMN_KEY: &str = "key";
/// The name of the column that stores the setting value.
const COLUMN_VALUE: &str = "value";

pub type SettingsResult<T> = std::result::Result<T, SettingsError>;

/// Errors related to settings operations.
#[derive(thiserror::Error, Debug)]
pub enum SettingsError {
    /// Errors that occur when the canister is misconfigured.
    #[error("Bad configuration")]
    BadConfig,
    /// Errors related to database operations.
    #[error("Database error: {0}")]
    Database(#[from] DbmsError),
    /// Errors that occur when the canister is not properly initialized.
    #[error("Canister is not initialized")]
    Uninitialized,
}

/// Canister settings stored in the database.
///
/// We must use this kind of key-value table, because we cannot use stable structures when using ic-dbms-canister.
#[derive(Debug, Table, Clone, PartialEq, Eq)]
#[table = "settings"]
pub struct Settings {
    #[primary_key]
    pub key: Uint16,
    #[custom_type]
    pub value: SettingValue,
}

impl Settings {
    /// Sets a configuration key in the settings table, replacing any existing value.
    pub fn set_config_key<M, A>(
        ctx: &DbmsContext<M, A>,
        schema: impl DatabaseSchema<M, A>,
        key: u16,
        value: impl Into<Value>,
    ) -> SettingsResult<()>
    where
        M: MemoryProvider,
        A: AccessControl,
    {
        let tx_id = ctx.begin_transaction(crate::transaction::transaction_caller(ic_utils::now()));
        let mut db = WasmDbmsDatabase::from_transaction(ctx, schema, tx_id);
        // delete key if it already exists
        db.delete::<Settings>(
            DeleteBehavior::Restrict,
            Some(Filter::eq(COLUMN_KEY, key.into())),
        )?;
        db.insert::<Settings>(SettingsInsertRequest {
            key: key.into(),
            value: SettingValue::from(value.into()),
        })?;
        db.commit()?;

        Ok(())
    }

    /// A helper function to get a setting value from the database and convert it to the desired type.
    pub fn get_required_settings_value<M, A, F, T>(
        ctx: &DbmsContext<M, A>,
        schema: impl DatabaseSchema<M, A>,
        key: u16,
        take_value: F,
    ) -> SettingsResult<T>
    where
        M: MemoryProvider,
        A: AccessControl,
        F: FnOnce(&Value) -> SettingsResult<T>,
    {
        match Self::get_settings_value(ctx, schema, key, take_value)? {
            Some(value) => Ok(value),
            None => Err(SettingsError::Uninitialized),
        }
    }

    /// A helper function to get a setting value from the database and convert it to the desired type.
    pub fn get_settings_value<M, A, F, T>(
        ctx: &DbmsContext<M, A>,
        schema: impl DatabaseSchema<M, A>,
        key: u16,
        take_value: F,
    ) -> SettingsResult<Option<T>>
    where
        M: MemoryProvider,
        A: AccessControl,
        F: FnOnce(&Value) -> SettingsResult<T>,
    {
        let db = WasmDbmsDatabase::oneshot(ctx, schema);
        let rows = db.select::<Settings>(
            Query::builder()
                .field(COLUMN_VALUE)
                .and_where(Filter::eq(COLUMN_KEY, key.into()))
                .limit(1)
                .build(),
        )?;

        let Some(record) = rows.first() else {
            return Ok(None);
        };
        let Some(value) = record.value.as_ref() else {
            return Err(SettingsError::Uninitialized);
        };

        take_value(&value.value).map(Some)
    }

    /// A helper function to convert a setting value to a [`candid::Principal`].
    pub fn get_as_principal(value: &Value) -> SettingsResult<candid::Principal> {
        if let Value::Blob(principal_str) = value {
            Ok(candid::Principal::from_slice(&principal_str.0))
        } else {
            Err(SettingsError::BadConfig)
        }
    }

    /// Converts a setting value to a [`String`].
    pub fn get_as_string(value: &Value) -> SettingsResult<String> {
        if let Value::Text(text) = value {
            Ok(text.to_string())
        } else {
            Err(SettingsError::BadConfig)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_as_string() {
        let value = Value::from("https://mastic.social".to_string());
        let result = Settings::get_as_string(&value).expect("should extract string");
        assert_eq!(result, "https://mastic.social");
    }

    #[test]
    fn test_get_as_string_wrong_type() {
        let value = Value::from(42u64);
        let result = Settings::get_as_string(&value);
        assert!(result.is_err());
    }
}
