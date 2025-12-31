# Orbis DSL

Multi-grammar pest parser with automatic case-insensitive keyword matching.

## Features

- ✅ **Multiple Independent Grammars**: Each grammar isolated in its own module
- ✅ **Case-Insensitive Keywords**: Supports 7 case formats (snake_case, camelCase, PascalCase, kebab-case, SCREAMING_SNAKE_CASE, SCREAMING-KEBAB-CASE, Train-Case)
- ✅ **Automatic Generation**: Grammar files generated at build time
- ✅ **Version Controlled**: Generated files copied to `src/` for easy inspection
- ✅ **Extensible**: Easy to add new grammars and keywords

## Available Grammars

| Grammar         | Module                 | Purpose                              |
|-----------------|------------------------|--------------------------------------|
| `page.pest`     | `orbis_dsl::page`      | Page definitions and UI elements     |
| `metadata.pest` | `orbis_dsl::metadata`  | Metadata and configuration fields    |

## Quick Start

### 1. Add to Cargo.toml

```toml
[dependencies]
orbis-dsl = { path = "../crates/orbis-dsl" }
pest = "2.8.4"
```

### 2. Use in Your Code

```rust
use orbis_dsl::page::{Parser, Rule};
use pest::Parser as PestParser;

// All of these are equivalent:
let input1 = "longString: String";
let input2 = "long_string: String";
let input3 = "LONG_STRING: String";

// They all parse successfully
let pairs = Parser::parse(Rule::field, input1)?;
```

## Module Structure

```text
orbis_dsl/
├── page           # Page grammar parser
│   ├── Parser     # pest parser (uncomment to enable)
│   └── Rule       # Grammar rules enum
└── metadata       # Metadata grammar parser
    ├── Parser     # pest parser (uncomment to enable)
    └── Rule       # Grammar rules enum
```

## Adding Keywords

Edit [`build.rs`](build.rs) to add keywords to existing grammars:

```rust
// In build.rs main function
page_keywords.insert("identifiers", vec![
    "page", 
    "longString", 
    "userId", 
    "apiVersion",
    "yourNewKeyword"  // Add here
]);
```

Rebuild: `cargo build -p orbis-dsl`

## Adding New Grammars

1. Define grammar in build.rs:

   ```rust
   let mut my_grammar: HashMap<&str, Vec<&str>> = HashMap::new();
   my_grammar.insert("category", vec!["keyword1", "keyword2"]);
   grammars.insert("my_grammar", my_grammar);
   ```

2. Add module in lib.rs:

   ```rust
   pub mod my_grammar {
       const _GRAMMAR: &str = include_str!("my_grammar.pest");
       
       // #[derive(pest_derive::Parser)]
       // #[grammar = "my_grammar.pest"]
       // pub struct Parser;
   }
   ```

3. Rebuild:

   ```bash
   cargo build -p orbis-dsl
   ```

## Supported Case Formats

Each keyword is automatically transformed into:

- **Train-Case**: `Long-String`
- **snake_case**: `long_string`
- **SCREAMING_SNAKE_CASE**: `LONG_STRING`
- **camelCase**: `longString`
- **PascalCase**: `LongString`
- **kebab-case**: `long-string`
- **SCREAMING-KEBAB-CASE**: `LONG-STRING`

## Testing

```bash
cargo test -p orbis-dsl
```

All generated grammars are tested to ensure:

- Files are properly included
- All case variants are present
- Module isolation is maintained

## Build Process

```text
cargo build
    ↓
build.rs executes
    ↓
For each grammar:
  - Generate rules from keywords
  - Write to OUT_DIR/{grammar}.pest
  - Copy to src/{grammar}.pest
    ↓
lib.rs includes via include_str!
    ↓
pest_derive generates parser (when enabled)
```

## License

See workspace [LICENSE](../../LICENSE) for details.
