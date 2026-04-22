//! Multi-emulator test execution and report aggregation tests.
//!
//! These tests validate that the multi-emulator runner correctly:
//! - Runs test suites against multiple emulator configs sequentially
//! - Aggregates results into per-emulator conformance reports
//! - Identifies each emulator's results by `vendor_name`
//! - Produces a combined conformance report at the end
//!
//! Tier 1 tests (no emulator required) validate the report aggregation logic.
//! Tier 2 tests (gated on `RUSTCONF_INTEGRATION_TEST=1`) validate live execution.
//!
//! Requirements: 11.3, 8.1

mod common;

use rustconf_integration_tests::{
    CombinedConformanceReport, ConformanceReporter, HarnessConfig, MultiEmulatorRunner,
    TestDetails, TestResult, TestStatus,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_pass(module: &str, operation: &str) -> TestResult {
    TestResult {
        yang_module: module.to_string(),
        operation: operation.to_string(),
        status: TestStatus::Pass,
        details: None,
    }
}

fn make_fail(module: &str, operation: &str) -> TestResult {
    TestResult {
        yang_module: module.to_string(),
        operation: operation.to_string(),
        status: TestStatus::Fail,
        details: Some(TestDetails {
            expected: Some("200".to_string()),
            actual: Some("500".to_string()),
            request: Some(format!("GET {operation}")),
            response: Some("Internal Server Error".to_string()),
            conformance_warnings: vec![],
        }),
    }
}

fn make_skip(module: &str, operation: &str, reason: &str) -> TestResult {
    TestResult {
        yang_module: module.to_string(),
        operation: operation.to_string(),
        status: TestStatus::Skip {
            reason: reason.to_string(),
        },
        details: None,
    }
}

// ---------------------------------------------------------------------------
// Tier 1: Combined report aggregation (no emulator needed)
// ---------------------------------------------------------------------------

/// Verify that a combined report from multiple emulators contains exactly N
/// sections, one per emulator, each identified by vendor_name.
#[test]
fn test_combined_report_section_count_matches_emulator_count() {
    let mut r1 = ConformanceReporter::new("Juniper cRPD");
    r1.record(make_pass("ietf-interfaces", "GET /interfaces"));

    let mut r2 = ConformanceReporter::new("Netopeer2");
    r2.record(make_pass("ietf-interfaces", "GET /interfaces"));

    let report = CombinedConformanceReport::from_reporters(&[r1, r2]);

    assert_eq!(
        report.emulator_count(),
        2,
        "Combined report should have exactly 2 sections for 2 emulators"
    );
    assert_eq!(report.vendor_names(), vec!["Juniper cRPD", "Netopeer2"]);
}

/// Verify that results from one emulator do not appear under another's section.
#[test]
fn test_combined_report_no_cross_contamination() {
    let mut crpd = ConformanceReporter::new("Juniper cRPD");
    crpd.record(make_pass("junos-conf", "GET /junos-config"));
    crpd.record(make_fail("junos-conf", "PUT /junos-config"));

    let mut netopeer = ConformanceReporter::new("Netopeer2");
    netopeer.record(make_pass("ietf-system", "GET /system"));
    netopeer.record(make_skip("ietf-system", "DELETE /system", "read-only"));

    let report = CombinedConformanceReport::from_reporters(&[crpd, netopeer]);

    // cRPD results
    let crpd_results = report.results_for_vendor("Juniper cRPD").unwrap();
    assert_eq!(crpd_results.len(), 2);
    for r in crpd_results {
        assert!(
            r.yang_module == "junos-conf",
            "cRPD section should only contain junos-conf results, got: {}",
            r.yang_module
        );
    }

    // Netopeer2 results
    let netopeer_results = report.results_for_vendor("Netopeer2").unwrap();
    assert_eq!(netopeer_results.len(), 2);
    for r in netopeer_results {
        assert!(
            r.yang_module == "ietf-system",
            "Netopeer2 section should only contain ietf-system results, got: {}",
            r.yang_module
        );
    }
}

/// Verify that the combined text report includes all emulator sections
/// and an overall summary.
#[test]
fn test_combined_text_report_structure() {
    let mut r1 = ConformanceReporter::new("Juniper cRPD");
    r1.record(make_pass("ietf-interfaces", "GET /interfaces"));
    r1.record(make_pass("ietf-interfaces", "PUT /interfaces"));

    let mut r2 = ConformanceReporter::new("Netopeer2");
    r2.record(make_pass("ietf-system", "GET /system"));
    r2.record(make_fail("ietf-system", "PATCH /system"));
    r2.record(make_skip("ietf-routing", "GET /routing", "not loaded"));

    let report = CombinedConformanceReport::from_reporters(&[r1, r2]);
    let text = report.generate_text_report();

    // Header
    assert!(text.contains("Combined Conformance Report"));

    // Per-emulator sections identified by vendor_name
    assert!(text.contains("=== Conformance Report: Juniper cRPD ==="));
    assert!(text.contains("=== Conformance Report: Netopeer2 ==="));

    // Overall summary aggregates across emulators
    assert!(text.contains("Overall Summary: 3 passed, 1 failed, 1 skipped"));
    assert!(text.contains("across 2 emulators)"));
}

/// Verify that per-emulator summaries are correct.
#[test]
fn test_per_emulator_summary_counts() {
    let mut r1 = ConformanceReporter::new("Emulator-A");
    r1.record(make_pass("m", "o1"));
    r1.record(make_pass("m", "o2"));
    r1.record(make_fail("m", "o3"));

    let mut r2 = ConformanceReporter::new("Emulator-B");
    r2.record(make_skip("m", "o4", "reason"));

    let mut r3 = ConformanceReporter::new("Emulator-C");
    r3.record(make_pass("m", "o5"));
    r3.record(make_fail("m", "o6"));
    r3.record(make_fail("m", "o7"));

    let report = CombinedConformanceReport::from_reporters(&[r1, r2, r3]);

    let summaries = report.per_emulator_summary();
    assert_eq!(summaries.len(), 3);
    assert_eq!(summaries[0], ("Emulator-A", 2, 1, 0));
    assert_eq!(summaries[1], ("Emulator-B", 0, 0, 1));
    assert_eq!(summaries[2], ("Emulator-C", 1, 2, 0));

    assert_eq!(report.overall_summary(), (3, 3, 1));
}

/// Verify that the combined JUnit XML report has the correct structure.
#[test]
fn test_combined_junit_xml_report() {
    let mut r1 = ConformanceReporter::new("Juniper cRPD");
    r1.record(make_pass("ietf-interfaces", "GET /interfaces"));

    let mut r2 = ConformanceReporter::new("Netopeer2");
    r2.record(make_fail("ietf-system", "PUT /system"));

    let report = CombinedConformanceReport::from_reporters(&[r1, r2]);
    let xml = report.generate_junit_xml();

    // Valid XML structure
    assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(xml.contains("name=\"multi-emulator-conformance\""));
    assert!(xml.contains("tests=\"2\""));
    assert!(xml.contains("failures=\"1\""));
    assert!(xml.contains("skipped=\"0\""));

    // Contains test suites from both emulators
    assert!(xml.contains("<testsuite"));
    assert!(xml.contains("</testsuites>"));
}

/// Verify that an empty combined report handles gracefully.
#[test]
fn test_combined_report_empty() {
    let report = CombinedConformanceReport::from_reporters(&[]);

    assert_eq!(report.emulator_count(), 0);
    assert_eq!(report.overall_summary(), (0, 0, 0));

    let text = report.generate_text_report();
    assert!(text.contains("No emulator results recorded."));
}

/// Verify that the MultiEmulatorRunner initializes with empty reporters.
#[test]
fn test_runner_initial_state() {
    let configs = vec![
        HarnessConfig {
            emulator_type: "crpd".to_string(),
            ..HarnessConfig::default()
        },
        HarnessConfig {
            emulator_type: "netopeer2".to_string(),
            ..HarnessConfig::default()
        },
    ];

    let runner = MultiEmulatorRunner::new(configs);
    assert!(
        runner.reporters().is_empty(),
        "Runner should have no reporters before run_all"
    );

    let combined = runner.combined_report();
    assert_eq!(combined.emulator_count(), 0);
}

// ---------------------------------------------------------------------------
// Tier 2: Live multi-emulator execution (requires emulator)
// ---------------------------------------------------------------------------

/// Run a test suite against multiple emulators and verify the combined report.
///
/// This test requires `RUSTCONF_INTEGRATION_TEST=1` and a container runtime.
#[tokio::test]
async fn test_multi_emulator_live_execution() {
    skip_unless_integration!();
    skip_unless_emulator!();

    let configs = vec![
        HarnessConfig {
            emulator_type: "crpd".to_string(),
            ..HarnessConfig::from_env()
        },
        HarnessConfig {
            emulator_type: "netopeer2".to_string(),
            ..HarnessConfig::from_env()
        },
    ];

    let mut runner = MultiEmulatorRunner::new(configs);

    runner
        .run_all(|harness, reporter| {
            Box::pin(async move {
                // Simple smoke test: record a pass for each emulator
                reporter.record(TestResult {
                    yang_module: "smoke-test".to_string(),
                    operation: format!("health-check:{}", harness.base_url()),
                    status: TestStatus::Pass,
                    details: None,
                });
                Ok(())
            })
        })
        .await
        .expect("run_all should complete without infrastructure error");

    let combined = runner.combined_report();

    // Should have results from both emulators (or skip entries if startup failed)
    assert!(
        combined.emulator_count() >= 1,
        "Should have at least one emulator section"
    );

    let text = combined.generate_text_report();
    assert!(
        text.contains("Combined Conformance Report"),
        "Combined report should have header"
    );
    assert!(
        text.contains("Overall Summary"),
        "Combined report should have overall summary"
    );
}
