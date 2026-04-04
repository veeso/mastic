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
}
