//! Internet computer utilities for canister development.

use candid::Principal;

/// Returns this canister's own principal.
pub fn canister_id() -> Principal {
    #[cfg(target_family = "wasm")]
    {
        ic_cdk::api::canister_self()
    }
    #[cfg(not(target_family = "wasm"))]
    {
        // dummy canister principal for non-wasm targets (e.g., during unit tests)
        Principal::from_text("aaaaa-aa").expect("it should be valid")
    }
}

/// Returns the caller's principal.
pub fn caller() -> Principal {
    #[cfg(target_family = "wasm")]
    {
        ic_cdk::api::msg_caller()
    }
    #[cfg(not(target_family = "wasm"))]
    {
        // dummy principal for non-wasm targets (e.g., during unit tests)
        Principal::from_text("ghsi2-tqaaa-aaaan-aaaca-cai").expect("it should be valid")
    }
}

/// Returns whether the given principal is a controller of the canister.
pub fn is_controller(_principal: &Principal) -> bool {
    #[cfg(target_family = "wasm")]
    {
        ic_cdk::api::is_controller(_principal)
    }
    #[cfg(not(target_family = "wasm"))]
    {
        true
    }
}

/// Format a millisecond-precision Unix timestamp as RFC 3339 (UTC, seconds precision).
///
/// Produced output looks like `2026-04-22T18:30:00Z`. Used to render
/// timestamps in outbound ActivityPub payloads.
pub fn rfc3339(ms: u64) -> String {
    let secs = ms / 1_000;
    let (year, month, day, hour, minute, second) = civil_from_unix(secs);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Convert Unix seconds to civil (year, month, day, hour, minute, second) in UTC.
///
/// Uses the Howard Hinnant `days_from_civil` algorithm in reverse. A local
/// implementation is preferred over adding a `chrono`/`time` dependency since
/// the canister only needs integer-second UTC rendering.
fn civil_from_unix(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let days = (secs / 86_400) as i64;
    let sod = (secs % 86_400) as u32;
    let hour = sod / 3_600;
    let minute = (sod / 60) % 60;
    let second = sod % 60;

    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (y as u32, m as u32, d as u32, hour, minute, second)
}

/// Returns the current time in milliseconds since the UNIX epoch.
pub fn now() -> u64 {
    #[cfg(target_family = "wasm")]
    {
        ic_cdk::api::time() / 1_000_000 // convert nanoseconds to milliseconds
    }
    #[cfg(not(target_family = "wasm"))]
    {
        // return the current time in milliseconds since the UNIX epoch for non-wasm targets
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64
    }
}

/// Schedules a future to run after a delay.
///
/// On wasm targets, delegates to [`ic_cdk_timers::set_timer`].
/// On non-wasm targets (unit tests), spawns a tokio task that sleeps then
/// executes the future.
pub fn set_timer<F>(delay: std::time::Duration, future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    #[cfg(target_family = "wasm")]
    {
        ic_cdk_timers::set_timer(delay, future);
    }
    #[cfg(not(target_family = "wasm"))]
    {
        // Drop the future; the async steps of the state machine are tested
        // independently with tokio in their own module.
        let _ = (delay, future);
    }
}

/// A utility module for canister trapping.
///
/// This module provides a function and a macro to facilitate trapping the canister execution.
/// On WebAssembly targets, it uses `ic_cdk::trap`, while on non-Wasm targets, it uses Rust's standard panic mechanism.
/// On non-Wasm targets, it just `panic!`s with the provided message.
pub fn trap(msg: String) -> ! {
    #[cfg(target_family = "wasm")]
    {
        ic_cdk::trap(msg);
    }
    #[cfg(not(target_family = "wasm"))]
    {
        std::panic::panic_any(msg)
    }
}

/// A macro for trapping the canister execution.
///
/// # Examples
///
/// ```rust,no_run
/// use ic_utils::trap;
/// let name = "IC DBMS Canister";
/// trap!("Hello, {name}!");
/// trap!("Canister trapped.");
/// trap!("This is a debug message with a value: {}", 42);
/// ```
#[macro_export]
macro_rules! trap {
    ($($key:tt $(:$capture:tt)? $(= $value:expr)?),+; $($arg:tt)+) => ({
        $crate::utils::trap(format!($($arg)+));
    });

    ( $($arg:tt)+) => ({
        $crate::trap(format!($($arg)+));
    });
}

#[macro_export]
macro_rules! log {
    ($($key:tt $(:$capture:tt)? $(= $value:expr)?),+; $($arg:tt)+) => ({
        #[cfg(target_family = "wasm")]
        {
            ic_cdk::println!("{}", format!($($arg)+));
        }
        #[cfg(not(target_family = "wasm"))]
        {
            println!("[DEBUG] {}", format!($($arg)+));
        }
    });

    ( $($arg:tt)+) => ({
        #[cfg(target_family = "wasm")]
        {
            ic_cdk::println!("{}", format!($($arg)+));
        }
        #[cfg(not(target_family = "wasm"))]
        {
            println!("[DEBUG] {}", format!($($arg)+));
        }
    });
}

#[cfg(test)]
mod tests {

    #[test]
    #[should_panic(expected = "This is a test trap message with value: 100")]
    fn test_trap_macro() {
        crate::trap!("This is a test trap message with value: {}", 100);
    }

    #[test]
    fn test_rfc3339_epoch() {
        assert_eq!(crate::rfc3339(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn test_rfc3339_known_instant() {
        // 2021-01-01T00:00:00Z = 1_609_459_200 seconds = 1_609_459_200_000 ms
        assert_eq!(crate::rfc3339(1_609_459_200_000), "2021-01-01T00:00:00Z");
    }

    #[test]
    fn test_rfc3339_mid_day() {
        // 2021-01-02T03:04:05Z = 1_609_556_645 seconds
        assert_eq!(crate::rfc3339(1_609_556_645_000), "2021-01-02T03:04:05Z");
    }
}
