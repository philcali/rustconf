//! Build script for the integration test crate.
//!
//! Discovers vendor YANG models from the `juniper-yang` git submodule and
//! invokes `rustconf::RustconfBuilder` to generate client and server code
//! into `src/generated/`.
//!
//! The submodule is expected at `yang/juniper-yang/` and contains versioned
//! Juniper YANG models organised by release (e.g. `23.4/23.4R1/`).
//!
//! If the submodule is not initialised or no `.yang` files are found, the
//! script emits a warning and writes an empty `mod.rs` so the crate still
//! compiles. Integration tests that depend on generated code will be skipped
//! at runtime.
//!
//! Fallback: if the submodule is absent, the script also checks the local
//! `yang/juniper/` and `yang/ietf/` directories (populated manually).

use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Submodule layout constants
// ---------------------------------------------------------------------------

/// Root of the Juniper YANG git submodule (relative to this crate).
const SUBMODULE_ROOT: &str = "yang/juniper-yang";

/// Default Junos release to use for model discovery.
/// This should match the cRPD image tag in `config/crpd.toml`.
const DEFAULT_JUNOS_RELEASE: &str = "23.4";
const DEFAULT_JUNOS_REVISION: &str = "23.4R1";

/// Fallback local directories (manually populated, kept from the original
/// setup so that developers who prefer copying models still have a path).
const FALLBACK_VENDOR_DIR: &str = "yang/juniper";
const FALLBACK_IETF_DIR: &str = "yang/ietf";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect all `.yang` files in `dir` (non-recursive).
fn collect_yang_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "yang"))
        .collect();
    files.sort();
    files
}

/// Derive a valid Rust module name from a YANG file stem.
///
/// Strips the `@YYYY-MM-DD` revision suffix that Juniper models use,
/// replaces hyphens with underscores, and removes non-identifier characters.
fn module_name_from_yang(path: &Path) -> String {
    let stem = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Strip trailing @YYYY-MM-DD revision suffix.
    let base = match stem.rfind('@') {
        Some(idx) => &stem[..idx],
        None => &stem,
    };

    base.replace('-', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Resolve YANG model directories, preferring the git submodule.
///
/// Returns `(yang_files_to_generate, search_paths)` where `yang_files_to_generate`
/// is the list of `.yang` files to generate code from, and `search_paths` contains
/// directories to pass as import search paths (IETF deps, etc.).
///
/// Returns `None` if neither the submodule nor the fallback dirs contain models.
fn resolve_yang_dirs() -> Option<(Vec<PathBuf>, Vec<PathBuf>)> {
    let submodule = PathBuf::from(SUBMODULE_ROOT);

    // Try the submodule first.
    if submodule.is_dir() {
        let release_dir = submodule
            .join(DEFAULT_JUNOS_RELEASE)
            .join(DEFAULT_JUNOS_REVISION);

        let vendor_dir = release_dir.join("native/conf-and-rpcs/junos/conf/models");
        let ietf_dir = release_dir.join("ietf/models");

        let mut yang_files = Vec::new();
        let mut search = Vec::new();

        // IETF models are simpler and more likely to parse cleanly — include
        // them as generation targets alongside vendor models.
        if ietf_dir.is_dir() {
            yang_files.extend(collect_yang_files(&ietf_dir));
            search.push(ietf_dir);
        }

        if vendor_dir.is_dir() {
            yang_files.extend(collect_yang_files(&vendor_dir));
            search.push(vendor_dir);
        }

        if !yang_files.is_empty() {
            return Some((yang_files, search));
        }

        println!(
            "cargo:warning=Juniper YANG submodule found but release {}/{} \
             does not contain expected model directories. \
             Falling back to local yang/ directories.",
            DEFAULT_JUNOS_RELEASE, DEFAULT_JUNOS_REVISION,
        );
    }

    // Fallback: local directories.
    let vendor = PathBuf::from(FALLBACK_VENDOR_DIR);
    let ietf = PathBuf::from(FALLBACK_IETF_DIR);

    let mut yang_files = Vec::new();
    let mut search = Vec::new();

    if ietf.is_dir() {
        yang_files.extend(collect_yang_files(&ietf));
        search.push(ietf);
    }
    if vendor.is_dir() {
        yang_files.extend(collect_yang_files(&vendor));
        search.push(vendor);
    }

    if yang_files.is_empty() {
        return None;
    }

    Some((yang_files, search))
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let output_dir = PathBuf::from("src/generated");

    if let Err(e) = fs::create_dir_all(&output_dir) {
        println!(
            "cargo:warning=Failed to create generated output directory: {}",
            e
        );
        return;
    }

    let Some((yang_files, search_paths)) = resolve_yang_dirs() else {
        println!(
            "cargo:warning=No YANG model files found. Either initialise the \
             juniper-yang git submodule (git submodule update --init) or \
             populate yang/juniper/ manually. Integration tests that depend \
             on generated code will be skipped."
        );
        write_empty_mod(&output_dir);
        emit_rerun_directives(&[]);
        return;
    };

    if yang_files.is_empty() {
        println!("cargo:warning=YANG directories exist but contain no .yang files.");
        write_empty_mod(&output_dir);
        emit_rerun_directives(&search_paths);
        return;
    }

    let mut generated_modules: Vec<String> = Vec::new();
    let mut had_errors = false;

    for yang_file in &yang_files {
        let mod_name = module_name_from_yang(yang_file);
        if mod_name.is_empty() {
            println!(
                "cargo:warning=Skipping YANG file with unusable name: {}",
                yang_file.display()
            );
            continue;
        }

        let module_output = output_dir.join(&mod_name);

        let mut builder = rustconf::RustconfBuilder::new()
            .yang_file(yang_file)
            .output_dir(&module_output)
            .module_name(&mod_name)
            .enable_validation(true)
            .enable_restful_rpcs(true)
            .modular_output(true);

        for sp in &search_paths {
            builder = builder.search_path(sp);
        }

        match builder.generate() {
            Ok(()) => {
                println!(
                    "cargo:warning=Generated code for YANG module: {}",
                    yang_file.display()
                );
                generated_modules.push(mod_name);
            }
            Err(e) => {
                println!(
                    "cargo:warning=Skipping YANG module {} \
                     (unsupported construct or parse error): {:?}",
                    yang_file.display(),
                    e
                );
                had_errors = true;
            }
        }
    }

    if had_errors {
        println!(
            "cargo:warning=Some YANG modules were skipped due to errors. \
             Tests for those modules will be skipped at runtime."
        );
    }

    write_mod_rs(&output_dir, &generated_modules);
    emit_rerun_directives(&search_paths);
}

// ---------------------------------------------------------------------------
// Output helpers
// ---------------------------------------------------------------------------

fn write_empty_mod(output_dir: &Path) {
    let mod_path = output_dir.join("mod.rs");
    let _ = fs::write(
        &mod_path,
        "//! Auto-generated module (empty — no YANG models found).\n",
    );
}

fn write_mod_rs(output_dir: &Path, modules: &[String]) {
    let mut out = String::from(
        "//! Auto-generated modules from vendor YANG models.\n\
         //!\n\
         //! This file is regenerated by `build.rs` whenever YANG models change.\n\
         //! Do not edit manually.\n\n",
    );

    if modules.is_empty() {
        out.push_str("// No modules were generated.\n");
    } else {
        for m in modules {
            out.push_str(&format!("pub mod {};\n", m));
        }
    }

    let mod_path = output_dir.join("mod.rs");
    if let Err(e) = fs::write(&mod_path, out) {
        println!("cargo:warning=Failed to write generated mod.rs: {}", e);
    }
}

/// Emit `cargo:rerun-if-changed` directives so cargo re-runs this script
/// when the build script itself or any YANG model directory changes.
fn emit_rerun_directives(extra_dirs: &[PathBuf]) {
    println!("cargo:rerun-if-changed=build.rs");

    // Always watch the submodule root (detects init / update).
    println!("cargo:rerun-if-changed={}", SUBMODULE_ROOT);

    // Watch fallback dirs too.
    println!("cargo:rerun-if-changed={}", FALLBACK_VENDOR_DIR);
    println!("cargo:rerun-if-changed={}", FALLBACK_IETF_DIR);

    for dir in extra_dirs {
        println!("cargo:rerun-if-changed={}", dir.display());
    }
}
