//! Argument inspection helpers for Directory canister query/update endpoints.

use db_utils::handle::{HandleSanitizer, HandleValidator};
use did::directory::SearchProfilesArgs;

/// Maximum number of profiles `search_profiles` may return in a single page.
const MAX_SEARCH_LIMIT: u64 = 50;

/// Validates [`SearchProfilesArgs`] before the query runs.
///
/// Rejects the call if:
/// - `limit` is `0` or greater than [`MAX_SEARCH_LIMIT`];
/// - the sanitized `query` (after [`HandleSanitizer`] strips `@`/whitespace and
///   lowercases) fails [`HandleValidator::check_handle`].
///
/// Empty queries are valid (they match all Active users, paginated).
pub fn inspect_search_profiles(args: &SearchProfilesArgs) -> Result<(), String> {
    if args.limit > MAX_SEARCH_LIMIT {
        return Err(format!("Limit cannot be greater than {MAX_SEARCH_LIMIT}"));
    }
    if args.limit == 0 {
        return Err("Limit cannot be zero".to_string());
    }

    // Empty queries are valid: the Directory returns all Active users, paginated.
    // Skip handle validation in that case since `HandleValidator` rejects empty input.
    let handle = HandleSanitizer::sanitize_handle(&args.query);
    if !handle.is_empty() {
        HandleValidator::check_handle(&handle)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(query: &str, limit: u64) -> SearchProfilesArgs {
        SearchProfilesArgs {
            query: query.to_string(),
            offset: 0,
            limit,
        }
    }

    #[test]
    fn test_should_accept_valid_args() {
        inspect_search_profiles(&args("alice", 10)).expect("valid args should pass");
    }

    #[test]
    fn test_should_accept_empty_query() {
        inspect_search_profiles(&args("", 10)).expect("empty query should pass");
    }

    #[test]
    fn test_should_accept_max_limit() {
        inspect_search_profiles(&args("alice", MAX_SEARCH_LIMIT))
            .expect("limit at MAX_SEARCH_LIMIT should pass");
    }

    #[test]
    fn test_should_reject_zero_limit() {
        let err = inspect_search_profiles(&args("alice", 0)).unwrap_err();
        assert!(err.contains("zero"), "got error: {err}");
    }

    #[test]
    fn test_should_reject_limit_above_max() {
        let err = inspect_search_profiles(&args("alice", MAX_SEARCH_LIMIT + 1)).unwrap_err();
        assert!(err.contains("greater than"), "got error: {err}");
    }

    #[test]
    fn test_should_reject_invalid_handle_after_sanitization() {
        // Handles must satisfy HandleValidator after sanitization. Disallowed characters fail.
        let err = inspect_search_profiles(&args("not a handle!", 10)).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn test_should_accept_query_with_at_prefix() {
        // `HandleSanitizer` strips leading `@` and lowercases; `@Alice` → `alice`.
        inspect_search_profiles(&args("@Alice", 10))
            .expect("@-prefixed query should pass after sanitization");
    }
}
