# Makefile for JunOS E2E Validation
#
# Usage:
#   make e2e          - Full E2E run (check prerequisites, run tests, report)
#   make e2e-smoke    - Run only smoke tests (fast feedback)
#   make e2e-clean    - Remove orphaned containers and report files
#   make check-runtime - Verify Docker or Podman is available
#   make check-image  - Verify cRPD image is loaded locally
#   make load-image   - Load cRPD image from tarball
#   make help         - Show this help message
#
# Configuration variables:
#   CRPD_TARBALL  - Path to cRPD image tarball for `make load-image`
#   TEST_FILTER   - Filter specific tests (passed to cargo test)
#   CONTAINER_RT  - Force container runtime: docker | podman | auto (default: auto)

.PHONY: e2e e2e-smoke e2e-clean check-runtime check-image load-image help

# ─── Configuration ────────────────────────────────────────────────────────────

CONFIG_FILE := config/crpd.toml
CRPD_IMAGE  := $(shell grep '^image' $(CONFIG_FILE) 2>/dev/null | sed 's/.*= *"\(.*\)"/\1/')

# Fallback if extraction fails
ifeq ($(CRPD_IMAGE),)
  CRPD_IMAGE := crpd:23.4R1.10
endif

CRPD_TARBALL  ?=
CONTAINER_RT  ?= auto
TEST_FILTER   ?=

# ─── Container Runtime Detection ─────────────────────────────────────────────

ifeq ($(CONTAINER_RT),auto)
  RUNTIME := $(shell docker info >/dev/null 2>&1 && echo docker || (podman info >/dev/null 2>&1 && echo podman))
else
  RUNTIME := $(CONTAINER_RT)
endif

# ─── Cargo Test Command ──────────────────────────────────────────────────────

CARGO_TEST_ENV := RUSTCONF_INTEGRATION_TEST=1 RUSTCONF_CONTAINER_IMAGE=$(CRPD_IMAGE)

ifdef TEST_FILTER
  CARGO_TEST_CMD := cargo test --package rustconf-integration-tests $(TEST_FILTER)
else
  CARGO_TEST_CMD := cargo test --package rustconf-integration-tests
endif

CARGO_TEST_SMOKE := cargo test --package rustconf-integration-tests smoke

# ─── Targets ─────────────────────────────────────────────────────────────────

help: ## Show this help message
	@echo "JunOS E2E Validation Targets"
	@echo "════════════════════════════════════════════════════════════════"
	@echo ""
	@echo "  make e2e            Full E2E run: check prerequisites, run tests, report"
	@echo "  make e2e-smoke      Run only smoke tests (fast connectivity check)"
	@echo "  make e2e-clean      Remove orphaned containers and report files"
	@echo "  make check-runtime  Verify Docker or Podman is available"
	@echo "  make check-image    Verify cRPD image is loaded locally"
	@echo "  make load-image     Load cRPD image from tarball (set CRPD_TARBALL=path)"
	@echo "  make help           Show this help message"
	@echo ""
	@echo "Configuration:"
	@echo "  CRPD_IMAGE    = $(CRPD_IMAGE)"
	@echo "  CONTAINER_RT  = $(CONTAINER_RT) (resolved: $(RUNTIME))"
	@echo "  CRPD_TARBALL  = $(or $(CRPD_TARBALL),(not set))"
	@echo "  TEST_FILTER   = $(or $(TEST_FILTER),(all tests))"
	@echo ""
	@echo "Examples:"
	@echo "  make e2e                              # Run full E2E suite"
	@echo "  make e2e TEST_FILTER=crud             # Run only CRUD tests"
	@echo "  make e2e-smoke                        # Quick connectivity check"
	@echo "  make load-image CRPD_TARBALL=crpd.tar # Load image from tarball"
	@echo ""

check-runtime: ## Verify Docker or Podman is available
	@if [ -z "$(RUNTIME)" ]; then \
		echo ""; \
		echo "ERROR: No container runtime found."; \
		echo ""; \
		echo "Please install one of the following:"; \
		echo "  - Docker: https://docs.docker.com/get-docker/"; \
		echo "  - Podman: https://podman.io/getting-started/installation"; \
		echo ""; \
		exit 1; \
	fi
	@echo "Container runtime: $(RUNTIME)"

check-image: check-runtime ## Verify cRPD image is loaded locally
	@echo "Checking for cRPD image: $(CRPD_IMAGE)"
	@if ! $(RUNTIME) image inspect $(CRPD_IMAGE) >/dev/null 2>&1; then \
		echo ""; \
		echo "ERROR: cRPD image '$(CRPD_IMAGE)' not found locally."; \
		echo ""; \
		echo "To obtain the cRPD image, choose one of:"; \
		echo ""; \
		echo "  1. Load from tarball:"; \
		echo "     make load-image CRPD_TARBALL=/path/to/crpd.tar"; \
		echo ""; \
		echo "  2. Pull from Juniper registry (requires authentication):"; \
		echo "     $(RUNTIME) login enterprise-hub.juniper.net"; \
		echo "     $(RUNTIME) pull enterprise-hub.juniper.net/$(CRPD_IMAGE)"; \
		echo "     $(RUNTIME) tag enterprise-hub.juniper.net/$(CRPD_IMAGE) $(CRPD_IMAGE)"; \
		echo ""; \
		echo "  3. Check the image tag in $(CONFIG_FILE) matches your local image:"; \
		echo "     $(RUNTIME) images | grep crpd"; \
		echo ""; \
		exit 1; \
	fi
	@LOCAL_TAG=$$($(RUNTIME) image inspect $(CRPD_IMAGE) --format '{{index .RepoTags 0}}' 2>/dev/null); \
	if [ "$$LOCAL_TAG" != "$(CRPD_IMAGE)" ] && [ -n "$$LOCAL_TAG" ]; then \
		echo "WARNING: Local image tag '$$LOCAL_TAG' differs from configured '$(CRPD_IMAGE)'"; \
	fi
	@echo "Image available: $(CRPD_IMAGE)"

load-image: check-runtime ## Load cRPD image from tarball
	@if [ -z "$(CRPD_TARBALL)" ]; then \
		echo ""; \
		echo "ERROR: CRPD_TARBALL not set."; \
		echo ""; \
		echo "Usage: make load-image CRPD_TARBALL=/path/to/crpd-image.tar"; \
		echo ""; \
		exit 1; \
	fi
	@if [ ! -f "$(CRPD_TARBALL)" ]; then \
		echo ""; \
		echo "ERROR: Tarball not found: $(CRPD_TARBALL)"; \
		echo ""; \
		exit 1; \
	fi
	@echo "Loading cRPD image from: $(CRPD_TARBALL)"
	$(RUNTIME) load -i $(CRPD_TARBALL)
	@echo "Image loaded. Verifying..."
	@$(RUNTIME) image inspect $(CRPD_IMAGE) >/dev/null 2>&1 && \
		echo "Success: $(CRPD_IMAGE) is now available." || \
		echo "WARNING: Image loaded but '$(CRPD_IMAGE)' not found. You may need to retag the image."

e2e: check-image ## Full E2E run: check prerequisites, run tests, report
	@echo ""
	@echo "════════════════════════════════════════════════════════════════"
	@echo " JunOS E2E Validation"
	@echo " Image: $(CRPD_IMAGE)"
	@echo " Runtime: $(RUNTIME)"
	@echo "════════════════════════════════════════════════════════════════"
	@echo ""
	@START_TIME=$$(date +%s); \
	EXIT_CODE=0; \
	$(CARGO_TEST_ENV) $(CARGO_TEST_CMD) -- --nocapture 2>&1 || EXIT_CODE=$$?; \
	END_TIME=$$(date +%s); \
	ELAPSED=$$((END_TIME - START_TIME)); \
	echo ""; \
	echo "════════════════════════════════════════════════════════════════"; \
	echo " E2E Validation Complete"; \
	echo " Elapsed: $${ELAPSED}s"; \
	if [ $$EXIT_CODE -eq 0 ]; then \
		echo " Result: PASSED"; \
	else \
		echo " Result: FAILED (exit code $$EXIT_CODE)"; \
	fi; \
	echo "════════════════════════════════════════════════════════════════"; \
	echo ""; \
	if [ -f reports/e2e-conformance.txt ]; then \
		echo "Conformance report: reports/e2e-conformance.txt"; \
	fi; \
	if [ -f reports/e2e-junit.xml ]; then \
		echo "JUnit report: reports/e2e-junit.xml"; \
	fi; \
	exit $$EXIT_CODE

e2e-smoke: check-image ## Run only smoke tests (fast connectivity check)
	@echo ""
	@echo "════════════════════════════════════════════════════════════════"
	@echo " JunOS E2E Smoke Tests"
	@echo " Image: $(CRPD_IMAGE)"
	@echo " Runtime: $(RUNTIME)"
	@echo "════════════════════════════════════════════════════════════════"
	@echo ""
	@START_TIME=$$(date +%s); \
	EXIT_CODE=0; \
	$(CARGO_TEST_ENV) $(CARGO_TEST_SMOKE) -- --nocapture 2>&1 || EXIT_CODE=$$?; \
	END_TIME=$$(date +%s); \
	ELAPSED=$$((END_TIME - START_TIME)); \
	echo ""; \
	echo "════════════════════════════════════════════════════════════════"; \
	echo " Smoke Tests Complete"; \
	echo " Elapsed: $${ELAPSED}s"; \
	if [ $$EXIT_CODE -eq 0 ]; then \
		echo " Result: PASSED"; \
	else \
		echo " Result: FAILED (exit code $$EXIT_CODE)"; \
	fi; \
	echo "════════════════════════════════════════════════════════════════"; \
	exit $$EXIT_CODE

e2e-clean: ## Remove orphaned containers and report files
	@echo "Cleaning up E2E validation artifacts..."
	@if [ -n "$(RUNTIME)" ]; then \
		CONTAINERS=$$($(RUNTIME) ps -a --filter "label=rustconf-e2e" -q 2>/dev/null); \
		if [ -n "$$CONTAINERS" ]; then \
			echo "Removing orphaned E2E containers..."; \
			$(RUNTIME) rm -f $$CONTAINERS; \
		else \
			echo "No orphaned containers found."; \
		fi; \
	else \
		echo "No container runtime available, skipping container cleanup."; \
	fi
	@rm -f reports/e2e-conformance.txt reports/e2e-junit.xml
	@echo "Report files cleaned."
	@echo "Done."
