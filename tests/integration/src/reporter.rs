//! Conformance reporting for structured integration test result output.
//!
//! [`ConformanceReporter`] collects [`TestResult`] values during a test session and
//! produces human-readable text reports (grouped by YANG module) and JUnit XML reports
//! for CI consumption.

use std::collections::BTreeMap;
use std::fmt;

/// Status of a single test execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStatus {
    /// The test passed.
    Pass,
    /// The test failed.
    Fail,
    /// The test was skipped.
    Skip {
        /// Reason the test was skipped (e.g., unsupported YANG construct).
        reason: String,
    },
}

impl fmt::Display for TestStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestStatus::Pass => write!(f, "Pass"),
            TestStatus::Fail => write!(f, "Fail"),
            TestStatus::Skip { reason } => write!(f, "Skip: {reason}"),
        }
    }
}

/// Additional details recorded when a test fails or produces conformance warnings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestDetails {
    /// Expected value or behavior (for failures).
    pub expected: Option<String>,
    /// Actual value or behavior observed (for failures).
    pub actual: Option<String>,
    /// The RESTCONF request that was sent.
    pub request: Option<String>,
    /// The RESTCONF response that was received.
    pub response: Option<String>,
    /// Conformance warnings (structural differences, non-fatal issues).
    pub conformance_warnings: Vec<String>,
}

/// A single test result entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestResult {
    /// YANG module this test covers (e.g., "ietf-interfaces").
    pub yang_module: String,
    /// Operation description (e.g., "GET /data/ietf-interfaces:interfaces").
    pub operation: String,
    /// Pass, Fail, or Skip status.
    pub status: TestStatus,
    /// Optional details (expected/actual values, request/response, warnings).
    pub details: Option<TestDetails>,
}

/// Collects test results and produces structured conformance reports.
///
/// Results are grouped by YANG module in the generated reports. Each reporter
/// instance is associated with a single emulator name for identification in
/// multi-emulator test runs.
pub struct ConformanceReporter {
    results: Vec<TestResult>,
    emulator_name: String,
}

impl ConformanceReporter {
    /// Create a new reporter for the given emulator.
    pub fn new(emulator_name: &str) -> Self {
        Self {
            results: Vec::new(),
            emulator_name: emulator_name.to_string(),
        }
    }

    /// Record a test result.
    pub fn record(&mut self, result: TestResult) {
        self.results.push(result);
    }

    /// Return summary counts: `(pass, fail, skip)`.
    pub fn summary(&self) -> (usize, usize, usize) {
        let mut pass = 0usize;
        let mut fail = 0usize;
        let mut skip = 0usize;

        for r in &self.results {
            match r.status {
                TestStatus::Pass => pass += 1,
                TestStatus::Fail => fail += 1,
                TestStatus::Skip { .. } => skip += 1,
            }
        }

        (pass, fail, skip)
    }

    /// The emulator name this reporter is associated with.
    pub fn emulator_name(&self) -> &str {
        &self.emulator_name
    }

    /// All recorded results.
    pub fn results(&self) -> &[TestResult] {
        &self.results
    }

    /// Generate a human-readable text report grouped by YANG module.
    ///
    /// The report format:
    /// ```text
    /// === Conformance Report: <emulator> ===
    ///
    /// Module: <yang_module> (<count> tests)
    ///   ✓ <operation>
    ///   ✗ <operation>
    ///       Expected: ...
    ///       Actual:   ...
    ///       Request:  ...
    ///       Response: ...
    ///   ⊘ <operation>
    ///       Skipped: <reason>
    ///
    /// Summary: N passed, N failed, N skipped
    /// ```
    pub fn generate_text_report(&self) -> String {
        let grouped = self.group_by_module();
        let (pass, fail, skip) = self.summary();
        let total: usize = pass + fail + skip;

        let mut out = String::new();

        out.push_str(&format!(
            "=== Conformance Report: {} ===\n",
            self.emulator_name
        ));

        if self.results.is_empty() {
            out.push_str("\nNo test results recorded.\n");
            return out;
        }

        for (module, results) in &grouped {
            out.push_str(&format!("\nModule: {} ({} tests)\n", module, results.len()));

            for result in results {
                match &result.status {
                    TestStatus::Pass => {
                        out.push_str(&format!("  \u{2713} {}\n", result.operation));
                    }
                    TestStatus::Fail => {
                        out.push_str(&format!("  \u{2717} {}\n", result.operation));
                        if let Some(ref details) = result.details {
                            write_failure_details(&mut out, details);
                        }
                    }
                    TestStatus::Skip { reason } => {
                        out.push_str(&format!("  \u{2298} {}\n", result.operation));
                        out.push_str(&format!("      Skipped: {reason}\n"));
                    }
                }
            }
        }

        out.push_str(&format!(
            "\nSummary: {pass} passed, {fail} failed, {skip} skipped (of {total} total)\n"
        ));

        out
    }

    /// Generate a JUnit XML report for CI integration.
    ///
    /// Produces a `<testsuites>` document with one `<testsuite>` per YANG module.
    /// Failed tests include `<failure>` elements; skipped tests include `<skipped>` elements.
    pub fn generate_junit_xml(&self) -> String {
        let grouped = self.group_by_module();
        let (pass, fail, skip) = self.summary();
        let total = pass + fail + skip;

        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str(&format!(
            "<testsuites name=\"{}\" tests=\"{}\" failures=\"{}\" skipped=\"{}\">\n",
            xml_escape(&self.emulator_name),
            total,
            fail,
            skip,
        ));

        for (module, results) in &grouped {
            let suite_pass = results
                .iter()
                .filter(|r| matches!(r.status, TestStatus::Pass))
                .count();
            let suite_fail = results
                .iter()
                .filter(|r| matches!(r.status, TestStatus::Fail))
                .count();
            let suite_skip = results
                .iter()
                .filter(|r| matches!(r.status, TestStatus::Skip { .. }))
                .count();

            xml.push_str(&format!(
                "  <testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" skipped=\"{}\">\n",
                xml_escape(module),
                suite_pass + suite_fail + suite_skip,
                suite_fail,
                suite_skip,
            ));

            for result in results {
                xml.push_str(&format!(
                    "    <testcase name=\"{}\" classname=\"{}\"",
                    xml_escape(&result.operation),
                    xml_escape(module),
                ));

                match &result.status {
                    TestStatus::Pass => {
                        xml.push_str(" />\n");
                    }
                    TestStatus::Fail => {
                        xml.push_str(">\n");
                        let message = build_failure_message(result);
                        xml.push_str(&format!(
                            "      <failure message=\"Test failed\">{}</failure>\n",
                            xml_escape(&message),
                        ));
                        xml.push_str("    </testcase>\n");
                    }
                    TestStatus::Skip { reason } => {
                        xml.push_str(">\n");
                        xml.push_str(&format!(
                            "      <skipped message=\"{}\" />\n",
                            xml_escape(reason),
                        ));
                        xml.push_str("    </testcase>\n");
                    }
                }
            }

            xml.push_str("  </testsuite>\n");
        }

        xml.push_str("</testsuites>\n");
        xml
    }

    /// Group results by YANG module, preserving insertion order within each module.
    ///
    /// Uses `BTreeMap` so modules appear in sorted order in reports.
    fn group_by_module(&self) -> BTreeMap<&str, Vec<&TestResult>> {
        let mut groups: BTreeMap<&str, Vec<&TestResult>> = BTreeMap::new();
        for result in &self.results {
            groups.entry(&result.yang_module).or_default().push(result);
        }
        groups
    }
}

/// Write indented failure details to the output string.
fn write_failure_details(out: &mut String, details: &TestDetails) {
    if let Some(ref expected) = details.expected {
        out.push_str(&format!("      Expected: {expected}\n"));
    }
    if let Some(ref actual) = details.actual {
        out.push_str(&format!("      Actual:   {actual}\n"));
    }
    if let Some(ref request) = details.request {
        out.push_str(&format!("      Request:  {request}\n"));
    }
    if let Some(ref response) = details.response {
        out.push_str(&format!("      Response: {response}\n"));
    }
    for warning in &details.conformance_warnings {
        out.push_str(&format!("      Warning:  {warning}\n"));
    }
}

/// Build a failure message string from a failed test result for JUnit XML.
fn build_failure_message(result: &TestResult) -> String {
    let mut msg = String::new();
    if let Some(ref details) = result.details {
        if let Some(ref expected) = details.expected {
            msg.push_str(&format!("Expected: {expected}\n"));
        }
        if let Some(ref actual) = details.actual {
            msg.push_str(&format!("Actual: {actual}\n"));
        }
        if let Some(ref request) = details.request {
            msg.push_str(&format!("Request: {request}\n"));
        }
        if let Some(ref response) = details.response {
            msg.push_str(&format!("Response: {response}\n"));
        }
        for warning in &details.conformance_warnings {
            msg.push_str(&format!("Warning: {warning}\n"));
        }
    }
    msg
}

/// Escape special XML characters in a string.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

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
                actual: Some("405".to_string()),
                request: Some("PATCH /restconf/data/test".to_string()),
                response: Some(r#"{"error": "method not allowed"}"#.to_string()),
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

    #[test]
    fn test_empty_reporter() {
        let reporter = ConformanceReporter::new("test-emulator");
        assert_eq!(reporter.summary(), (0, 0, 0));
        assert_eq!(reporter.emulator_name(), "test-emulator");
        assert!(reporter.results().is_empty());

        let report = reporter.generate_text_report();
        assert!(report.contains("No test results recorded."));
    }

    #[test]
    fn test_summary_counts() {
        let mut reporter = ConformanceReporter::new("crpd");
        reporter.record(make_pass("mod-a", "GET /a"));
        reporter.record(make_pass("mod-a", "PUT /a"));
        reporter.record(make_fail("mod-b", "PATCH /b"));
        reporter.record(make_skip("mod-c", "DELETE /c", "not supported"));

        assert_eq!(reporter.summary(), (2, 1, 1));
    }

    #[test]
    fn test_text_report_header_and_summary() {
        let mut reporter = ConformanceReporter::new("Juniper cRPD 23.4");
        reporter.record(make_pass(
            "ietf-interfaces",
            "GET /data/ietf-interfaces:interfaces",
        ));
        reporter.record(make_fail(
            "ietf-interfaces",
            "PATCH /data/ietf-interfaces:interfaces",
        ));
        reporter.record(make_skip(
            "junos-conf",
            "DELETE /data/junos-conf:config",
            "delete-on-remove not supported",
        ));

        let report = reporter.generate_text_report();

        // Header
        assert!(report.contains("=== Conformance Report: Juniper cRPD 23.4 ==="));

        // Module grouping
        assert!(report.contains("Module: ietf-interfaces (2 tests)"));
        assert!(report.contains("Module: junos-conf (1 tests)"));

        // Pass marker
        assert!(report.contains("\u{2713} GET /data/ietf-interfaces:interfaces"));

        // Fail marker with details
        assert!(report.contains("\u{2717} PATCH /data/ietf-interfaces:interfaces"));
        assert!(report.contains("Expected: 200"));
        assert!(report.contains("Actual:   405"));
        assert!(report.contains("Request:  PATCH /restconf/data/test"));

        // Skip marker with reason
        assert!(report.contains("\u{2298} DELETE /data/junos-conf:config"));
        assert!(report.contains("Skipped: delete-on-remove not supported"));

        // Summary
        assert!(report.contains("Summary: 1 passed, 1 failed, 1 skipped (of 3 total)"));
    }

    #[test]
    fn test_text_report_conformance_warnings() {
        let mut reporter = ConformanceReporter::new("test");
        reporter.record(TestResult {
            yang_module: "mod-a".to_string(),
            operation: "GET /a".to_string(),
            status: TestStatus::Fail,
            details: Some(TestDetails {
                expected: None,
                actual: None,
                request: None,
                response: None,
                conformance_warnings: vec![
                    "Missing Content-Type header".to_string(),
                    "Extra JSON key: foo".to_string(),
                ],
            }),
        });

        let report = reporter.generate_text_report();
        assert!(report.contains("Warning:  Missing Content-Type header"));
        assert!(report.contains("Warning:  Extra JSON key: foo"));
    }

    #[test]
    fn test_text_report_modules_sorted() {
        let mut reporter = ConformanceReporter::new("test");
        reporter.record(make_pass("z-module", "GET /z"));
        reporter.record(make_pass("a-module", "GET /a"));
        reporter.record(make_pass("m-module", "GET /m"));

        let report = reporter.generate_text_report();

        let pos_a = report.find("Module: a-module").unwrap();
        let pos_m = report.find("Module: m-module").unwrap();
        let pos_z = report.find("Module: z-module").unwrap();
        assert!(pos_a < pos_m);
        assert!(pos_m < pos_z);
    }

    #[test]
    fn test_junit_xml_structure() {
        let mut reporter = ConformanceReporter::new("crpd");
        reporter.record(make_pass("ietf-interfaces", "GET /interfaces"));
        reporter.record(make_fail("ietf-interfaces", "PATCH /interfaces"));
        reporter.record(make_skip("junos-conf", "DELETE /config", "unsupported"));

        let xml = reporter.generate_junit_xml();

        // XML header
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));

        // Root element with totals
        assert!(xml.contains("<testsuites name=\"crpd\" tests=\"3\" failures=\"1\" skipped=\"1\">"));

        // Test suites per module
        assert!(xml.contains("<testsuite name=\"ietf-interfaces\""));
        assert!(xml.contains("<testsuite name=\"junos-conf\""));

        // Pass case (self-closing)
        assert!(xml.contains("<testcase name=\"GET /interfaces\" classname=\"ietf-interfaces\" />"));

        // Fail case with failure element
        assert!(xml.contains("<testcase name=\"PATCH /interfaces\""));
        assert!(xml.contains("<failure message=\"Test failed\">"));
        assert!(xml.contains("Expected: 200"));
        assert!(xml.contains("Actual: 405"));
        assert!(xml.contains("</failure>"));

        // Skip case
        assert!(xml.contains("<testcase name=\"DELETE /config\""));
        assert!(xml.contains("<skipped message=\"unsupported\" />"));

        // Closing
        assert!(xml.contains("</testsuites>"));
    }

    #[test]
    fn test_junit_xml_escapes_special_chars() {
        let mut reporter = ConformanceReporter::new("test <&> emulator");
        reporter.record(TestResult {
            yang_module: "mod\"a".to_string(),
            operation: "GET /data?foo=bar&baz=1".to_string(),
            status: TestStatus::Pass,
            details: None,
        });

        let xml = reporter.generate_junit_xml();
        assert!(xml.contains("test &lt;&amp;&gt; emulator"));
        assert!(xml.contains("mod&quot;a"));
        assert!(xml.contains("foo=bar&amp;baz=1"));
    }

    #[test]
    fn test_junit_xml_per_suite_counts() {
        let mut reporter = ConformanceReporter::new("test");
        reporter.record(make_pass("mod-a", "GET /a1"));
        reporter.record(make_pass("mod-a", "GET /a2"));
        reporter.record(make_fail("mod-a", "PUT /a3"));
        reporter.record(make_skip("mod-b", "GET /b1", "reason"));

        let xml = reporter.generate_junit_xml();

        // mod-a: 3 tests, 1 failure, 0 skipped
        assert!(xml.contains("<testsuite name=\"mod-a\" tests=\"3\" failures=\"1\" skipped=\"0\">"));
        // mod-b: 1 test, 0 failures, 1 skipped
        assert!(xml.contains("<testsuite name=\"mod-b\" tests=\"1\" failures=\"0\" skipped=\"1\">"));
    }

    #[test]
    fn test_record_preserves_order_within_module() {
        let mut reporter = ConformanceReporter::new("test");
        reporter.record(make_pass("mod-a", "op-1"));
        reporter.record(make_pass("mod-a", "op-2"));
        reporter.record(make_pass("mod-a", "op-3"));

        let results = reporter.results();
        assert_eq!(results[0].operation, "op-1");
        assert_eq!(results[1].operation, "op-2");
        assert_eq!(results[2].operation, "op-3");
    }

    #[test]
    fn test_test_status_display() {
        assert_eq!(TestStatus::Pass.to_string(), "Pass");
        assert_eq!(TestStatus::Fail.to_string(), "Fail");
        assert_eq!(
            TestStatus::Skip {
                reason: "not supported".to_string()
            }
            .to_string(),
            "Skip: not supported"
        );
    }

    #[test]
    fn test_fail_without_details() {
        let mut reporter = ConformanceReporter::new("test");
        reporter.record(TestResult {
            yang_module: "mod-a".to_string(),
            operation: "GET /a".to_string(),
            status: TestStatus::Fail,
            details: None,
        });

        let report = reporter.generate_text_report();
        assert!(report.contains("\u{2717} GET /a"));
        // No details lines should appear
        assert!(!report.contains("Expected:"));
        assert!(!report.contains("Actual:"));
    }
}
