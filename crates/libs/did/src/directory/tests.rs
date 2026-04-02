use candid::{Decode, Encode};

use super::*;

#[test]
fn test_should_roundtrip_directory_install_args_init() {
    let args = DirectoryInstallArgs::Init {
        initial_moderator: candid::Principal::anonymous(),
        federation_canister: candid::Principal::anonymous(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, DirectoryInstallArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_directory_install_args_upgrade() {
    let args = DirectoryInstallArgs::Upgrade {};
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, DirectoryInstallArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_sign_up_request_ok() {
    let resp = SignUpRequest {
        handle: "alice".to_string(),
    };
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, SignUpRequest).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_sign_up_response_ok() {
    let resp = SignUpResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, SignUpResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_sign_up_response_err() {
    for error in [
        SignUpError::AlreadyRegistered,
        SignUpError::HandleTaken,
        SignUpError::InvalidHandle,
    ] {
        let resp = SignUpResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, SignUpResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_who_am_i_response_ok() {
    let resp = WhoAmIResponse::Ok(WhoAmI {
        handle: "alice".to_string(),
        user_canister: candid::Principal::anonymous(),
        canister_status: UserCanisterStatus::Active,
    });
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, WhoAmIResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_who_am_i_response_err() {
    let resp = WhoAmIResponse::Err(WhoAmIError::NotRegistered);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, WhoAmIResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_user_canister_response_ok() {
    let resp = UserCanisterResponse::Ok(candid::Principal::anonymous());
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, UserCanisterResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_user_canister_response_err() {
    let resp = UserCanisterResponse::Err(UserCanisterError::NotRegistered);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, UserCanisterResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_get_user_args() {
    let args = GetUserArgs {
        handle: "alice".to_string(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, GetUserArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_get_user_response_ok() {
    let resp = GetUserResponse::Ok(GetUser {
        handle: "alice".to_string(),
        canister_id: candid::Principal::anonymous(),
    });
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, GetUserResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_get_user_response_err() {
    let resp = GetUserResponse::Err(GetUserError::NotFound);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, GetUserResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_add_moderator_args() {
    let args = AddModeratorArgs {
        principal: candid::Principal::anonymous(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, AddModeratorArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_add_moderator_response_ok() {
    let resp = AddModeratorResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, AddModeratorResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_add_moderator_response_err() {
    for error in [
        AddModeratorError::Unauthorized,
        AddModeratorError::AlreadyModerator,
    ] {
        let resp = AddModeratorResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, AddModeratorResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_remove_moderator_args() {
    let args = RemoveModeratorArgs {
        principal: candid::Principal::anonymous(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, RemoveModeratorArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_remove_moderator_response_ok() {
    let resp = RemoveModeratorResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, RemoveModeratorResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_remove_moderator_response_err() {
    for error in [
        RemoveModeratorError::Unauthorized,
        RemoveModeratorError::NotModerator,
    ] {
        let resp = RemoveModeratorResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, RemoveModeratorResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_suspend_args() {
    let args = SuspendArgs {
        principal: candid::Principal::anonymous(),
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, SuspendArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_suspend_response_ok() {
    let resp = SuspendResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, SuspendResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_suspend_response_err() {
    for error in [SuspendError::Unauthorized, SuspendError::NotFound] {
        let resp = SuspendResponse::Err(error);
        let bytes = Encode!(&resp).unwrap();
        let decoded = Decode!(&bytes, SuspendResponse).unwrap();
        assert_eq!(resp, decoded);
    }
}

#[test]
fn test_should_roundtrip_search_profiles_args() {
    let args = SearchProfilesArgs {
        query: "alice".to_string(),
        offset: 0,
        limit: 10,
    };
    let bytes = Encode!(&args).unwrap();
    let decoded = Decode!(&bytes, SearchProfilesArgs).unwrap();
    assert_eq!(args, decoded);
}

#[test]
fn test_should_roundtrip_search_profiles_response_ok() {
    let resp = SearchProfilesResponse::Ok(vec![SearchProfileEntry {
        handle: "alice".to_string(),
        canister_id: candid::Principal::anonymous(),
    }]);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, SearchProfilesResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_search_profiles_response_err() {
    let resp = SearchProfilesResponse::Err(SearchProfilesError::Unauthorized);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, SearchProfilesResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_delete_profile_response_ok() {
    let resp = DeleteProfileResponse::Ok;
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, DeleteProfileResponse).unwrap();
    assert_eq!(resp, decoded);
}

#[test]
fn test_should_roundtrip_delete_profile_response_err() {
    let resp = DeleteProfileResponse::Err(DeleteProfileError::NotRegistered);
    let bytes = Encode!(&resp).unwrap();
    let decoded = Decode!(&bytes, DeleteProfileResponse).unwrap();
    assert_eq!(resp, decoded);
}
