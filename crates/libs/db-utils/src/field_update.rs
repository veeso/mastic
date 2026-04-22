//! Helpers for mapping [`FieldUpdate`] to database update patches.

use did::common::FieldUpdate;
use wasm_dbms_api::prelude::{DataType, Nullable};

/// Convert a [`FieldUpdate<T>`] into the `Option<Nullable<T>>` shape expected
/// by `wasm-dbms` update requests.
///
/// - [`FieldUpdate::Set`] becomes `Some(Nullable::Value(value))`
/// - [`FieldUpdate::Clear`] becomes `Some(Nullable::Null)`
/// - [`FieldUpdate::Leave`] becomes `None`
pub fn field_update_to_nullable<T>(field_update: FieldUpdate<T>) -> Option<Nullable<T>>
where
    T: DataType,
{
    match field_update {
        FieldUpdate::Set(value) => Some(Nullable::Value(value)),
        FieldUpdate::Clear => Some(Nullable::Null),
        FieldUpdate::Leave => None,
    }
}

#[cfg(test)]
mod tests {
    use wasm_dbms_api::prelude::Text;

    use super::*;

    #[test]
    fn test_set_maps_to_some_value() {
        let text: Text = "hi".into();
        let out = field_update_to_nullable(FieldUpdate::Set(text.clone()));
        assert_eq!(out, Some(Nullable::Value(text)));
    }

    #[test]
    fn test_clear_maps_to_some_null() {
        let out: Option<Nullable<Text>> = field_update_to_nullable(FieldUpdate::Clear);
        assert_eq!(out, Some(Nullable::Null));
    }

    #[test]
    fn test_leave_maps_to_none() {
        let out: Option<Nullable<Text>> = field_update_to_nullable(FieldUpdate::Leave);
        assert_eq!(out, None);
    }
}
