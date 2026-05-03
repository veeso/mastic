//! Get profile flow.

use db_utils::repository::Repository;
use did::common::UserProfile;
use did::user::{GetProfileError, GetProfileResponse};

use crate::domain::profile::ProfileRepository;
use crate::error::CanisterError;

/// Gets the profile of the user.
pub fn get_profile() -> GetProfileResponse {
    let user = match ProfileRepository::oneshot().get_profile() {
        Ok(profile) => profile,
        Err(CanisterError::Settings(_)) => {
            return GetProfileResponse::Err(GetProfileError::NotFound);
        }
        Err(e) => return GetProfileResponse::Err(GetProfileError::Internal(e.to_string())),
    };

    GetProfileResponse::Ok(UserProfile {
        handle: user.handle.0,
        display_name: user.display_name.into_opt().map(|s| s.0),
        bio: user.bio.into_opt().map(|s| s.0),
        avatar: user.avatar_data.into_opt().map(|b| b.0),
        header: user.header_data.into_opt().map(|b| b.0),
        created_at: user.created_at.0,
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::setup;

    #[test]
    fn test_should_get_profile_after_setup() {
        setup();

        let response = get_profile();

        let GetProfileResponse::Ok(profile) = response else {
            panic!("expected Ok, got {response:?}");
        };
        assert_eq!(profile.handle, "rey_canisteryo");
        assert!(profile.display_name.is_none());
        assert!(profile.bio.is_none());
        assert!(profile.avatar.is_none());
        assert!(profile.header.is_none());
        assert!(profile.created_at > 0);
    }

    #[test]
    fn test_should_return_not_found_when_no_profile_exists() {
        // initialize the database schema without creating a profile
        ic_dbms_canister::prelude::DBMS_CONTEXT.with(|ctx| {
            crate::schema::Schema::register_tables(ctx).expect("should register schema");
        });

        let response = get_profile();

        assert_eq!(response, GetProfileResponse::Err(GetProfileError::NotFound));
    }
}
