// =============================================================================
// ORBIS DSL BUILD SYSTEM
// =============================================================================
//
// This is the main build script for the Orbis DSL crate. It runs at compile
// time and generates:
//
// 1. **Pest Grammar Files**: Case-insensitive keyword matching rules and
//    component-specific attribute/event whitelisting for type-safe parsing.
//
// 2. **Component Documentation**: Auto-generated Markdown reference for all
//    supported components, attributes, and events.
//
// =============================================================================
// ARCHITECTURE OVERVIEW
// =============================================================================
//
// The build system is split into focused modules under the `build/` directory:
//
// ```
// build/
// ├── mod.rs              <- Module index (for normal compilation, not build.rs)
// ├── data_structures.rs  <- Core types (ComponentDef, AttributeDef, EventDef)
// ├── components.rs       <- Component definitions (synced with TypeScript)
// ├── grammar.rs          <- Pest grammar generation logic
// ├── documentation.rs    <- Markdown documentation generation
// └── utils.rs            <- Helper utilities (string manipulation)
// ```
//
// =============================================================================
// WHY `include!` INSTEAD OF `mod`?
// =============================================================================
//
// Build scripts (`build.rs`) run in a special context where:
// - The normal module system doesn't work fully (limited `mod` declarations)
// - The `OUT_DIR` environment variable is available for generated output
// - All code must be self-contained or use `include!`
//
// We use `include!` to bring in the modular code while keeping the benefits
// of separate files for maintainability and documentation.
//
// =============================================================================
// AVOIDING BUILD LOOPS
// =============================================================================
//
// To prevent infinite rebuild loops:
// 1. Only specific files trigger rebuilds (via `cargo:rerun-if-changed`)
// 2. The build script writes to BOTH `OUT_DIR` and `src/` for:
//    - Cargo/Pest (expects grammar in OUT_DIR for compilation)
//    - IDE support and version control (src/*.pest)
// 3. Generated content is replaced between markers, preserving hand-written code
//
// =============================================================================
// HOW TO MODIFY
// =============================================================================
//
// - **Add a new component**: Edit `build/components.rs`, add to `define_components()`
// - **Add a new attribute**: Add AttributeDef to the component's `attributes` vec
// - **Add a new event**: Add EventDef to the component's `events` vec
// - **Deprecate something**: Use `.deprecated()` builder method
// - **Add allowed values**: Use `AttributeDef::with_values()` instead of `::new()`
//
// After changes, run `cargo build` and the grammar will be regenerated.
//
// =============================================================================

// =============================================================================
// DEPENDENCIES
// =============================================================================
// All crates used by the included modules must be imported here FIRST

use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{read_to_string, File},
    io::Write,
    path::Path,
};

use heck::{
    ToKebabCase,
    ToLowerCamelCase,
    ToShoutyKebabCase,
    ToShoutySnakeCase,
    ToSnakeCase,
    ToTrainCase,
    ToUpperCamelCase,
};
use maplit::hashmap;

// =============================================================================
// MODULAR INCLUDES
// =============================================================================
//
// The build system is split into separate files for maintainability.
// The include order matters - dependencies must come before dependents:
//
// 1. data_structures.rs - Core types (no dependencies)
// 2. utils.rs           - String utilities (no custom dependencies)
// 3. components.rs      - Component definitions (uses data_structures)
// 4. grammar.rs         - Grammar generation (uses data_structures, utils)
// 5. documentation.rs   - Doc generation (uses data_structures)

// Core type definitions - ComponentDef, AttributeDef, EventDef, DeprecationInfo
include!("build/data_structures.rs");

// String manipulation utilities - generate_all_variants, replace_section
include!("build/utils.rs");

// Component definitions - define_components(), type value constants
include!("build/components.rs");

// Grammar generation - generate_page_grammar_file, generate_grammar_file
include!("build/grammar.rs");

// Documentation generation - generate_component_documentation, generate_cheat_sheet
include!("build/documentation.rs");

// =============================================================================
// MAIN BUILD FUNCTION
// =============================================================================

/// Main entry point for the build script.
///
/// This function:
/// 1. Loads component definitions from `build/components.rs`
/// 2. Generates Pest grammar files with component whitelisting
/// 3. Generates Markdown documentation for all components
/// 4. Configures Cargo rebuild triggers
fn main() {
    // Get Cargo's output directory for generated files
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");

    // =========================================================================
    // STEP 1: Configure rebuild triggers FIRST
    // =========================================================================
    // Tell Cargo to rerun this script ONLY when these specific files change.
    // This is critical for preventing infinite rebuild loops.

    // Build script itself
    println!("cargo:rerun-if-changed=build.rs");

    // Build system modules
    println!("cargo:rerun-if-changed=build/mod.rs");
    println!("cargo:rerun-if-changed=build/data_structures.rs");
    println!("cargo:rerun-if-changed=build/components.rs");
    println!("cargo:rerun-if-changed=build/grammar.rs");
    println!("cargo:rerun-if-changed=build/documentation.rs");
    println!("cargo:rerun-if-changed=build/utils.rs");

    // Source grammar files (hand-written portions)
    // Note: We write TO these files, but we also need to trigger on their
    // hand-written content changes
    println!("cargo:rerun-if-changed=src/page.pest");
    println!("cargo:rerun-if-changed=src/metadata.pest");

    // =========================================================================
    // STEP 2: Load component definitions
    // =========================================================================
    // The component definitions are the single source of truth for:
    // - Which components exist in the DSL
    // - What attributes each component accepts
    // - What events each component can emit
    // - Type constraints and allowed values
    // - Deprecation information

    let components = define_components();

    // =========================================================================
    // STEP 3: Generate page grammar
    // =========================================================================
    // The page grammar (page.pest) is the main grammar file that handles:
    // - Template blocks with JSX-like syntax
    // - Component tags with attribute/event whitelisting
    // - Case-insensitive keyword matching
    // - Strongly-typed attribute values

    let page_keywords = generate_page_grammar_keywords();
    generate_page_grammar_file(&out_dir, "page.pest", &page_keywords, &components);

    // =========================================================================
    // STEP 4: Generate metadata grammar
    // =========================================================================
    // The metadata grammar handles frontmatter parsing for DSL files.
    // This is a simpler grammar for metadata directives like @page, @layout, etc.

    let metadata_keywords = hashmap! {
        // Page metadata directives
        "MetadataKeywords" => vec![
            "title", "description", "version", "author", "layout", "route",
        ],
    };
    generate_grammar_file(&out_dir, "metadata.pest", &metadata_keywords);

    // =========================================================================
    // STEP 5: Generate component documentation
    // =========================================================================
    // Creates human-readable documentation from component definitions:
    // - COMPONENTS_REFERENCE.md - Full documentation with all attributes/events
    // - COMPONENTS_CHEATSHEET.md - Quick reference card

    generate_component_documentation(&components);
    generate_cheat_sheet(&components);

    // =========================================================================
    // BUILD COMPLETE
    // =========================================================================
    // At this point:
    // - page.pest and metadata.pest have been generated in OUT_DIR and src/
    // - COMPONENTS_REFERENCE.md and COMPONENTS_CHEATSHEET.md exist in crate root
    // - Pest will pick up the grammar from OUT_DIR for parsing
}
