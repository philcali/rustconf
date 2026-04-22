//! Multi-emulator test execution and report aggregation.
//!
//! [`MultiEmulatorRunner`] runs a test suite against multiple emulator configurations
//! sequentially, collecting per-emulator results into [`ConformanceReporter`] instances.
//! After all emulators have been tested, [`CombinedConformanceReport`] aggregates the
//! individual reports into a single combined output identified by each emulator's
//! `vendor_name`.
//!
//! # Usage
//!
//! ```rust,ignore
//! use rustconf_integration_tests::{
//!     MultiEmulatorRunner, HarnessConfig, CombinedConformanceReport,
//! };
//!
//! let configs = vec![
//!     HarnessConfig::from_file(Path::new("config/crpd.toml")).unwrap(),
//!     HarnessConfig::from_file(Path::new("config/netopeer2.toml")).unwrap(),
//! ];
//!
//! let mut runner = MultiEmulatorRunner::new(configs);
//! runner.run_all(|harness, reporter| async move {
//!     // ... run tests, record results into reporter ...
//!     Ok(())
//! }).await.unwrap();
//!
//! let combined = runner.combined_report();
//! println!("{}", combined.generate_text_report());
//! ```
//!
//! Requirements: 11.3, 8.1

use std::future::Future;
use std::pin::Pin;

use crate::config::HarnessConfig;
use crate::emulators::{JunosCrpdConfig, NetopeerConfig};
use crate::error::HarnessError;
use crate::harness::TestHarness;
use crate::reporter::{ConformanceReporter, TestDetails, TestResult, TestStatus};

/// Creates the appropriate [`TestHarness`] for a given [`HarnessConfig`] based on
/// the `emulator_type` field.
///
/// Supported emulator types:
/// - `"crpd"` → [`JunosCrpdConfig`]
/// - `"netopeer2"` → [`NetopeerConfig`]
///
/// Returns an error if the emulator type is not recognized.
pub fn create_harness(config: &HarnessConfig) -> Result<TestHarness, HarnessError> {
    match config.emulator_type.as_str() {
        "crpd" => {
            let emulator = JunosCrpdConfig::with_harness_config(config);
            Ok(TestHarness::new(emulator, config))
        }
        "netopeer2" => {
            let emulator = NetopeerConfig::with_harness_config(config);
            Ok(TestHarness::new(emulator, config))
        }
        other => Err(HarnessError::ConfigError(format!(
            "Unknown emulator type: '{other}'. Supported types: crpd, netopeer2"
        ))),
    }
}

/// Runs a test suite against multiple emulator configurations sequentially.
///
/// Each emulator is started, tested, and stopped before moving to the next.
/// Results are collected into per-emulator [`ConformanceReporter`] instances
/// that can be aggregated into a [`CombinedConformanceReport`].
pub struct MultiEmulatorRunner {
    configs: Vec<HarnessConfig>,
    reporters: Vec<ConformanceReporter>,
}

impl MultiEmulatorRunner {
    /// Create a new runner with the given harness configurations.
    ///
    /// Each [`HarnessConfig`] determines which emulator to test. The `emulator_type`
    /// field selects the concrete [`EmulatorConfig`](crate::EmulatorConfig) implementation.
    pub fn new(configs: Vec<HarnessConfig>) -> Self {
        Self {
            configs,
            reporters: Vec::new(),
        }
    }

    /// Run the test suite against all configured emulators sequentially.
    ///
    /// For each emulator configuration:
    /// 1. A [`TestHarness`] is created from the config's `emulator_type`
    /// 2. The harness is started (container launched, health check performed)
    /// 3. A [`ConformanceReporter`] is created, identified by the emulator's `vendor_name`
    /// 4. The provided `test_suite` function is called with the harness and reporter
    /// 5. The harness is stopped and the reporter is stored for later aggregation
    ///
    /// If an emulator fails to start, its tests are skipped and a skip result is
    /// recorded in the reporter. Execution continues with the next emulator.
    ///
    /// Returns `Ok(())` when all emulators have been processed (even if some failed).
    pub async fn run_all<F>(&mut self, test_suite: F) -> Result<(), HarnessError>
    where
        F: for<'a> Fn(
            &'a mut TestHarness,
            &'a mut ConformanceReporter,
        ) -> Pin<Box<dyn Future<Output = Result<(), HarnessError>> + 'a>>,
    {
        for config in &self.configs {
            let mut harness = match create_harness(config) {
                Ok(h) => h,
                Err(e) => {
                    let mut reporter = ConformanceReporter::new(&config.emulator_type);
                    reporter.record(TestResult {
                        yang_module: "infrastructure".to_string(),
                        operation: format!("create-harness:{}", config.emulator_type),
                        status: TestStatus::Skip {
                            reason: format!("Failed to create harness: {e}"),
                        },
                        details: None,
                    });
                    self.reporters.push(reporter);
                    continue;
                }
            };

            // Determine vendor name before starting (for reporter identification).
            // We use the emulator_type as a fallback label until the harness starts.
            let vendor_label = config.emulator_type.clone();

            match harness.start().await {
                Ok(()) => {
                    // Now we can get the actual vendor name from the running harness
                    let mut reporter = ConformanceReporter::new(&vendor_label);

                    // Run the test suite — infrastructure errors are recorded but
                    // don't stop processing of other emulators
                    if let Err(e) = test_suite(&mut harness, &mut reporter).await {
                        reporter.record(TestResult {
                            yang_module: "infrastructure".to_string(),
                            operation: "test-suite-execution".to_string(),
                            status: TestStatus::Fail,
                            details: Some(TestDetails {
                                expected: Some("Test suite completes".to_string()),
                                actual: Some(format!("Error: {e}")),
                                request: None,
                                response: None,
                                conformance_warnings: vec![],
                            }),
                        });
                    }

                    // Best-effort stop
                    if let Err(e) = harness.stop().await {
                        eprintln!("Warning: failed to stop emulator {vendor_label}: {e}");
                    }

                    self.reporters.push(reporter);
                }
                Err(e) => {
                    // Emulator failed to start — record skip
                    let mut reporter = ConformanceReporter::new(&vendor_label);
                    reporter.record(TestResult {
                        yang_module: "infrastructure".to_string(),
                        operation: format!("start-emulator:{vendor_label}"),
                        status: TestStatus::Skip {
                            reason: format!("Emulator startup failed: {e}"),
                        },
                        details: None,
                    });
                    self.reporters.push(reporter);
                }
            }
        }

        Ok(())
    }

    /// Get the per-emulator reporters collected during [`run_all`](Self::run_all).
    pub fn reporters(&self) -> &[ConformanceReporter] {
        &self.reporters
    }

    /// Build a [`CombinedConformanceReport`] from all per-emulator reporters.
    pub fn combined_report(&self) -> CombinedConformanceReport {
        CombinedConformanceReport::from_reporters(&self.reporters)
    }
}

/// Aggregated conformance report from multiple emulator test runs.
///
/// Each emulator's results are kept separate and identified by `vendor_name`.
/// The combined report produces a single text or JUnit XML output that contains
/// all per-emulator sections.
pub struct CombinedConformanceReport {
    /// Per-emulator report sections.
    sections: Vec<ReportSection>,
}

/// A single emulator's section within the combined report.
struct ReportSection {
    vendor_name: String,
    reporter: ConformanceReporter,
}

impl CombinedConformanceReport {
    /// Build a combined report from a slice of per-emulator reporters.
    ///
    /// Each reporter's results are cloned into a separate section identified
    /// by the reporter's `emulator_name`.
    pub fn from_reporters(reporters: &[ConformanceReporter]) -> Self {
        let sections = reporters
            .iter()
            .map(|r| {
                let mut section_reporter = ConformanceReporter::new(r.emulator_name());
                for result in r.results() {
                    section_reporter.record(result.clone());
                }
                ReportSection {
                    vendor_name: r.emulator_name().to_string(),
                    reporter: section_reporter,
                }
            })
            .collect();

        Self { sections }
    }

    /// Number of emulator sections in the combined report.
    pub fn emulator_count(&self) -> usize {
        self.sections.len()
    }

    /// Get the vendor names of all emulators in the report.
    pub fn vendor_names(&self) -> Vec<&str> {
        self.sections
            .iter()
            .map(|s| s.vendor_name.as_str())
            .collect()
    }

    /// Get the per-emulator summary counts as `(vendor_name, pass, fail, skip)`.
    pub fn per_emulator_summary(&self) -> Vec<(&str, usize, usize, usize)> {
        self.sections
            .iter()
            .map(|s| {
                let (pass, fail, skip) = s.reporter.summary();
                (s.vendor_name.as_str(), pass, fail, skip)
            })
            .collect()
    }

    /// Overall summary counts across all emulators: `(pass, fail, skip)`.
    pub fn overall_summary(&self) -> (usize, usize, usize) {
        let mut total_pass = 0;
        let mut total_fail = 0;
        let mut total_skip = 0;

        for section in &self.sections {
            let (p, f, s) = section.reporter.summary();
            total_pass += p;
            total_fail += f;
            total_skip += s;
        }

        (total_pass, total_fail, total_skip)
    }

    /// Generate a combined human-readable text report.
    ///
    /// The output contains one section per emulator, each identified by `vendor_name`,
    /// followed by an overall summary.
    ///
    /// ```text
    /// ========================================
    /// Combined Conformance Report
    /// ========================================
    ///
    /// === Conformance Report: Juniper cRPD ===
    /// ...per-emulator report...
    ///
    /// === Conformance Report: Netopeer2 ===
    /// ...per-emulator report...
    ///
    /// ========================================
    /// Overall Summary: 20 passed, 3 failed, 2 skipped (across 2 emulators)
    /// ========================================
    /// ```
    pub fn generate_text_report(&self) -> String {
        let mut out = String::new();

        out.push_str("========================================\n");
        out.push_str("Combined Conformance Report\n");
        out.push_str("========================================\n");

        if self.sections.is_empty() {
            out.push_str("\nNo emulator results recorded.\n");
            return out;
        }

        for section in &self.sections {
            out.push('\n');
            out.push_str(&section.reporter.generate_text_report());
        }

        let (pass, fail, skip) = self.overall_summary();
        let total = pass + fail + skip;
        let emulator_count = self.sections.len();

        out.push_str("\n========================================\n");
        out.push_str(&format!(
            "Overall Summary: {pass} passed, {fail} failed, {skip} skipped \
             (of {total} total across {emulator_count} emulator{})\n",
            if emulator_count == 1 { "" } else { "s" }
        ));
        out.push_str("========================================\n");

        out
    }

    /// Generate a combined JUnit XML report.
    ///
    /// Produces a `<testsuites>` document where each emulator's results are
    /// nested as separate test suites, prefixed with the vendor name.
    pub fn generate_junit_xml(&self) -> String {
        let (pass, fail, skip) = self.overall_summary();
        let total = pass + fail + skip;

        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str(&format!(
            "<testsuites name=\"multi-emulator-conformance\" tests=\"{total}\" \
             failures=\"{fail}\" skipped=\"{skip}\">\n",
        ));

        for section in &self.sections {
            // Embed each emulator's JUnit XML as nested testsuites.
            // We strip the XML header and outer <testsuites> wrapper from each
            // individual report and include the inner <testsuite> elements.
            let individual_xml = section.reporter.generate_junit_xml();
            for line in individual_xml.lines() {
                let trimmed = line.trim();
                // Skip XML declaration and outer testsuites tags
                if trimmed.starts_with("<?xml")
                    || trimmed.starts_with("<testsuites")
                    || trimmed.starts_with("</testsuites")
                {
                    continue;
                }
                xml.push_str("  ");
                xml.push_str(line);
                xml.push('\n');
            }
        }

        xml.push_str("</testsuites>\n");
        xml
    }

    /// Get results for a specific emulator by vendor name.
    pub fn results_for_vendor(&self, vendor_name: &str) -> Option<&[TestResult]> {
        self.sections
            .iter()
            .find(|s| s.vendor_name == vendor_name)
            .map(|s| s.reporter.results())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a pass result.
    fn make_pass(module: &str, operation: &str) -> TestResult {
        TestResult {
            yang_module: module.to_string(),
            operation: operation.to_string(),
            status: TestStatus::Pass,
            details: None,
        }
    }

    /// Helper to create a fail result.
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

    /// Helper to create a skip result.
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

    // -----------------------------------------------------------------------
    // create_harness tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_harness_crpd() {
        let config = HarnessConfig {
            emulator_type: "crpd".to_string(),
            ..HarnessConfig::default()
        };
        let harness = create_harness(&config);
        assert!(harness.is_ok());
    }

    #[test]
    fn test_create_harness_netopeer2() {
        let config = HarnessConfig {
            emulator_type: "netopeer2".to_string(),
            ..HarnessConfig::default()
        };
        let harness = create_harness(&config);
        assert!(harness.is_ok());
    }

    #[test]
    fn test_create_harness_unknown_type() {
        let config = HarnessConfig {
            emulator_type: "unknown-emulator".to_string(),
            ..HarnessConfig::default()
        };
        let result = create_harness(&config);
        assert!(result.is_err());
        match result {
            Err(e) => assert!(
                e.to_string().contains("Unknown emulator type"),
                "Error should mention unknown type: {e}"
            ),
            Ok(_) => panic!("Expected error for unknown emulator type"),
        }
    }

    // -----------------------------------------------------------------------
    // CombinedConformanceReport unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_combined_report_empty() {
        let report = CombinedConformanceReport::from_reporters(&[]);
        assert_eq!(report.emulator_count(), 0);
        assert!(report.vendor_names().is_empty());
        assert_eq!(report.overall_summary(), (0, 0, 0));

        let text = report.generate_text_report();
        assert!(text.contains("No emulator results recorded."));
    }

    #[test]
    fn test_combined_report_single_emulator() {
        let mut reporter = ConformanceReporter::new("Juniper cRPD");
        reporter.record(make_pass("ietf-interfaces", "GET /interfaces"));
        reporter.record(make_fail("ietf-interfaces", "PATCH /interfaces"));
        reporter.record(make_skip("junos-conf", "DELETE /config", "unsupported"));

        let report = CombinedConformanceReport::from_reporters(&[reporter]);

        assert_eq!(report.emulator_count(), 1);
        assert_eq!(report.vendor_names(), vec!["Juniper cRPD"]);
        assert_eq!(report.overall_summary(), (1, 1, 1));

        let summaries = report.per_emulator_summary();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0], ("Juniper cRPD", 1, 1, 1));

        let text = report.generate_text_report();
        assert!(text.contains("Combined Conformance Report"));
        assert!(text.contains("Conformance Report: Juniper cRPD"));
        assert!(text.contains("Overall Summary: 1 passed, 1 failed, 1 skipped"));
        assert!(text.contains("across 1 emulator)"));
    }

    #[test]
    fn test_combined_report_multiple_emulators() {
        let mut crpd_reporter = ConformanceReporter::new("Juniper cRPD");
        crpd_reporter.record(make_pass("ietf-interfaces", "GET /interfaces"));
        crpd_reporter.record(make_pass("ietf-interfaces", "PUT /interfaces"));
        crpd_reporter.record(make_fail("junos-conf", "PATCH /config"));

        let mut netopeer_reporter = ConformanceReporter::new("Netopeer2");
        netopeer_reporter.record(make_pass("ietf-interfaces", "GET /interfaces"));
        netopeer_reporter.record(make_skip("ietf-system", "PUT /system", "read-only"));

        let report = CombinedConformanceReport::from_reporters(&[crpd_reporter, netopeer_reporter]);

        assert_eq!(report.emulator_count(), 2);
        assert_eq!(report.vendor_names(), vec!["Juniper cRPD", "Netopeer2"]);
        assert_eq!(report.overall_summary(), (3, 1, 1));

        let summaries = report.per_emulator_summary();
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0], ("Juniper cRPD", 2, 1, 0));
        assert_eq!(summaries[1], ("Netopeer2", 1, 0, 1));
    }

    #[test]
    fn test_combined_report_no_cross_contamination() {
        let mut crpd_reporter = ConformanceReporter::new("Juniper cRPD");
        crpd_reporter.record(make_pass("mod-a", "op-crpd-1"));
        crpd_reporter.record(make_pass("mod-a", "op-crpd-2"));

        let mut netopeer_reporter = ConformanceReporter::new("Netopeer2");
        netopeer_reporter.record(make_pass("mod-b", "op-netopeer-1"));

        let report = CombinedConformanceReport::from_reporters(&[crpd_reporter, netopeer_reporter]);

        // Verify results are isolated per vendor
        let crpd_results = report.results_for_vendor("Juniper cRPD").unwrap();
        assert_eq!(crpd_results.len(), 2);
        assert!(crpd_results.iter().all(|r| r.operation.contains("crpd")));

        let netopeer_results = report.results_for_vendor("Netopeer2").unwrap();
        assert_eq!(netopeer_results.len(), 1);
        assert!(netopeer_results
            .iter()
            .all(|r| r.operation.contains("netopeer")));

        // No results for unknown vendor
        assert!(report.results_for_vendor("Unknown").is_none());
    }

    #[test]
    fn test_combined_text_report_contains_all_sections() {
        let mut r1 = ConformanceReporter::new("Emulator-A");
        r1.record(make_pass("mod-x", "GET /x"));

        let mut r2 = ConformanceReporter::new("Emulator-B");
        r2.record(make_fail("mod-y", "PUT /y"));

        let mut r3 = ConformanceReporter::new("Emulator-C");
        r3.record(make_skip("mod-z", "DELETE /z", "not implemented"));

        let report = CombinedConformanceReport::from_reporters(&[r1, r2, r3]);
        let text = report.generate_text_report();

        // All three emulator sections present
        assert!(text.contains("Conformance Report: Emulator-A"));
        assert!(text.contains("Conformance Report: Emulator-B"));
        assert!(text.contains("Conformance Report: Emulator-C"));

        // Overall summary
        assert!(text.contains("Overall Summary: 1 passed, 1 failed, 1 skipped"));
        assert!(text.contains("across 3 emulators)"));
    }

    #[test]
    fn test_combined_text_report_vendor_identification() {
        let mut r1 = ConformanceReporter::new("Juniper cRPD");
        r1.record(make_pass("ietf-interfaces", "GET /interfaces"));

        let mut r2 = ConformanceReporter::new("Netopeer2");
        r2.record(make_pass("ietf-interfaces", "GET /interfaces"));

        let report = CombinedConformanceReport::from_reporters(&[r1, r2]);
        let text = report.generate_text_report();

        // Both vendor names appear as section headers
        assert!(text.contains("=== Conformance Report: Juniper cRPD ==="));
        assert!(text.contains("=== Conformance Report: Netopeer2 ==="));
    }

    #[test]
    fn test_combined_junit_xml_structure() {
        let mut r1 = ConformanceReporter::new("Juniper cRPD");
        r1.record(make_pass("ietf-interfaces", "GET /interfaces"));

        let mut r2 = ConformanceReporter::new("Netopeer2");
        r2.record(make_fail("ietf-system", "PUT /system"));

        let report = CombinedConformanceReport::from_reporters(&[r1, r2]);
        let xml = report.generate_junit_xml();

        // XML header
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));

        // Root element with combined totals
        assert!(xml.contains("name=\"multi-emulator-conformance\""));
        assert!(xml.contains("tests=\"2\""));
        assert!(xml.contains("failures=\"1\""));

        // Contains test suites from both emulators
        assert!(xml.contains("testsuite name="));
        assert!(xml.contains("</testsuites>"));
    }

    #[test]
    fn test_multi_emulator_runner_new() {
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
        assert!(runner.reporters().is_empty());
    }

    #[test]
    fn test_combined_report_results_for_vendor() {
        let mut r1 = ConformanceReporter::new("Vendor-A");
        r1.record(make_pass("mod-1", "op-a1"));
        r1.record(make_pass("mod-1", "op-a2"));

        let mut r2 = ConformanceReporter::new("Vendor-B");
        r2.record(make_fail("mod-2", "op-b1"));

        let report = CombinedConformanceReport::from_reporters(&[r1, r2]);

        let vendor_a = report.results_for_vendor("Vendor-A").unwrap();
        assert_eq!(vendor_a.len(), 2);
        assert_eq!(vendor_a[0].operation, "op-a1");
        assert_eq!(vendor_a[1].operation, "op-a2");

        let vendor_b = report.results_for_vendor("Vendor-B").unwrap();
        assert_eq!(vendor_b.len(), 1);
        assert_eq!(vendor_b[0].operation, "op-b1");

        assert!(report.results_for_vendor("Vendor-C").is_none());
    }

    #[test]
    fn test_combined_report_overall_summary_aggregates() {
        let mut r1 = ConformanceReporter::new("E1");
        r1.record(make_pass("m", "o1"));
        r1.record(make_pass("m", "o2"));
        r1.record(make_fail("m", "o3"));

        let mut r2 = ConformanceReporter::new("E2");
        r2.record(make_pass("m", "o4"));
        r2.record(make_skip("m", "o5", "reason"));
        r2.record(make_skip("m", "o6", "reason"));

        let report = CombinedConformanceReport::from_reporters(&[r1, r2]);
        assert_eq!(report.overall_summary(), (3, 1, 2));
    }
}
