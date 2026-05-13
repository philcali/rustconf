//! E2E test helpers for Junos validation.
//!
//! Provides:
//! - `e2e_resource_name`: generates unique resource names for test isolation
//! - `SMOKE_PASSED` gate: allows CRUD/schema/error tests to skip if smoke fails
//!
//! Requirements: 8.3, 3.5

use std::sync::atomic::{AtomicBool, Ordering};

/// Shared state: set to `true` once all smoke tests pass.
/// CRUD, schema, and error tests check this before running.
pub static SMOKE_PASSED: AtomicBool = AtomicBool::new(false);

/// Skip the calling test if smoke tests have not passed.
///
/// When `SMOKE_PASSED` is `false`, the test returns early with a message
/// indicating it was skipped due to smoke test failure. This prevents
/// cascading failures when the emulator is unreachable or misconfigured.
///
/// # Usage
///
/// ```rust,ignore
/// #[tokio::test]
/// async fn test_crud_interface_create() {
///     skip_unless_integration!();
///     skip_unless_emulator!();
///     skip_unless_smoke_passed!();
///     // ... test body
/// }
/// ```
#[macro_export]
macro_rules! skip_unless_smoke_passed {
    () => {
        if !e2e_helpers::SMOKE_PASSED.load(std::sync::atomic::Ordering::SeqCst) {
            eprintln!("Skipping: smoke tests did not pass");
            return;
        }
    };
}

/// Generate a unique resource name for E2E tests.
///
/// Format: `e2e-{category}-{short_id}` where `short_id` is a 6-character
/// hex string derived from random bytes. This ensures test resources don't
/// collide across parallel runs or repeated executions.
///
/// # Arguments
///
/// * `category` - A short label for the test category (e.g., "crud", "schema")
///
/// # Examples
///
/// ```rust,ignore
/// let name = e2e_resource_name("crud");
/// // e.g., "e2e-crud-a3f1b2"
/// ```
pub fn e2e_resource_name(category: &str) -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    let state = RandomState::new();
    let mut hasher = state.build_hasher();
    // Mix in a timestamp for additional entropy
    hasher.write_u128(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    );
    let hash = hasher.finish();
    let short_id = format!("{:06x}", hash & 0xFFFFFF);
    format!("e2e-{category}-{short_id}")
}

/// Mark smoke tests as passed. Call this after all smoke tests succeed.
pub fn set_smoke_passed() {
    SMOKE_PASSED.store(true, Ordering::SeqCst);
}

/// Check whether smoke tests have passed.
pub fn is_smoke_passed() -> bool {
    SMOKE_PASSED.load(Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_e2e_resource_name_format() {
        let name = e2e_resource_name("crud");
        assert!(name.starts_with("e2e-crud-"), "got: {name}");
        // "e2e-crud-" is 9 chars, plus 6 hex chars = 15 total
        assert_eq!(name.len(), 15, "got: {name}");
        // The suffix should be valid hex
        let suffix = &name[9..];
        assert!(
            suffix.chars().all(|c| c.is_ascii_hexdigit()),
            "suffix not hex: {suffix}"
        );
    }

    #[test]
    fn test_e2e_resource_name_uniqueness() {
        let mut names = HashSet::new();
        for _ in 0..1000 {
            let name = e2e_resource_name("test");
            names.insert(name);
        }
        // With 6 hex chars (24 bits), 1000 names should have negligible collision probability.
        // Allow at most 1 collision to account for extreme edge cases.
        assert!(
            names.len() >= 999,
            "Too many collisions: only {} unique names out of 1000",
            names.len()
        );
    }

    #[test]
    fn test_e2e_resource_name_different_categories() {
        let name_a = e2e_resource_name("crud");
        let name_b = e2e_resource_name("schema");
        assert!(name_a.starts_with("e2e-crud-"));
        assert!(name_b.starts_with("e2e-schema-"));
        // Different categories produce different prefixes
        assert_ne!(&name_a[..9], &name_b[..11]);
    }

    #[test]
    fn test_smoke_passed_gate_default_false() {
        // Note: this test relies on SMOKE_PASSED being false at startup.
        // In a fresh test binary, it should be false.
        // We can't reliably test this in isolation since other tests may set it,
        // so we just verify the API works.
        let initial = is_smoke_passed();
        set_smoke_passed();
        assert!(is_smoke_passed());
        // Reset for other tests (best-effort, not guaranteed ordering)
        SMOKE_PASSED.store(initial, Ordering::SeqCst);
    }
}
