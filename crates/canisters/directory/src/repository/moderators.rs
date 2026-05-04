//! Moderators repository for the directory canister.

use candid::Principal;
use db_utils::repository::Repository;
use ic_dbms_canister::prelude::DBMS_CONTEXT;
use wasm_dbms_api::prelude::*;

use crate::error::{CanisterError, CanisterResult};
use crate::schema::{Moderator, ModeratorInsertRequest, Schema};

pub struct ModeratorsRepository {
    tx: Option<TransactionId>,
}

impl ModeratorsRepository {
    /// Adds a moderator to the directory canister.
    pub fn add_moderator(&self, principal: Principal) -> CanisterResult<()> {
        ic_utils::log!("ModeratorsRepository::add_moderator: inserting {principal}");
        DBMS_CONTEXT
            .with(|ctx| {
                self.db(ctx).insert::<Moderator>(ModeratorInsertRequest {
                    principal: ic_dbms_canister::prelude::Principal(principal),
                    created_at: ic_utils::now().into(),
                })
            })
            .map_err(CanisterError::from)
    }

    /// Returns true if the given principal is a moderator, false otherwise.
    pub fn is_moderator(&self, principal: Principal) -> CanisterResult<bool> {
        let principal = ic_dbms_canister::prelude::Principal(principal);
        DBMS_CONTEXT.with(|ctx| {
            let rows = self.db(ctx).select::<Moderator>(
                Query::builder()
                    .and_where(Filter::eq("principal", Value::from(principal)))
                    .limit(1)
                    .build(),
            )?;
            Ok::<_, CanisterError>(!rows.is_empty())
        })
    }

    /// Removes a moderator from the directory canister.
    pub fn remove_moderator(&self, principal: Principal) -> CanisterResult<()> {
        ic_utils::log!("ModeratorsRepository::remove_moderator: removing {principal}");
        let principal = ic_dbms_canister::prelude::Principal(principal);
        DBMS_CONTEXT
            .with(|ctx| {
                self.db(ctx).delete::<Moderator>(
                    DeleteBehavior::Cascade,
                    Some(Filter::eq("principal", Value::from(principal))),
                )
            })
            .map(|_| ())
            .map_err(CanisterError::from)
    }
}

impl Repository for ModeratorsRepository {
    type Schema = Schema;

    fn schema() -> Self::Schema {
        Schema
    }

    fn oneshot() -> Self {
        Self { tx: None }
    }

    fn with_transaction(tx: TransactionId) -> Self {
        Self { tx: Some(tx) }
    }

    fn tx(&self) -> Option<TransactionId> {
        self.tx
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::{rey_canisteryo, setup};

    #[test]
    fn test_should_add_and_check_moderator() {
        setup();

        ModeratorsRepository::oneshot()
            .add_moderator(rey_canisteryo())
            .expect("should add moderator");

        assert!(
            ModeratorsRepository::oneshot()
                .is_moderator(rey_canisteryo())
                .expect("should check moderator")
        );
    }

    #[test]
    fn test_should_remove_moderator() {
        setup();

        ModeratorsRepository::oneshot()
            .add_moderator(rey_canisteryo())
            .expect("should add moderator");
        ModeratorsRepository::oneshot()
            .remove_moderator(rey_canisteryo())
            .expect("should remove moderator");

        assert!(
            !ModeratorsRepository::oneshot()
                .is_moderator(rey_canisteryo())
                .expect("should check moderator")
        );
    }

    #[test]
    fn test_should_report_non_moderator_as_false() {
        setup();

        assert!(
            !ModeratorsRepository::oneshot()
                .is_moderator(rey_canisteryo())
                .expect("should check moderator")
        );
    }
}
