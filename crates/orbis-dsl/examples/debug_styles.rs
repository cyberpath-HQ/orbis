//! Debug styles parsing

use pest::Parser;

use orbis_dsl::page::{Parser as PageParser, Rule};

fn main() {
    let source = r#"styles scoped {
    .dashboard-layout {
        padding: 24px;
        max-width: 1200px;
        margin: 0 auto;
    }
    .metric-card {
        background: var(--card-bg);
        border-radius: var(--radius);
    }
    @media (min-width: 768px) {
        .metric-card {
            padding: 32px;
        }
    }
}"#;
    
    println!("Source:\n{}\n", source);
    println!("=== Raw Parse Tree ===\n");
    
    match PageParser::parse(Rule::styles_block, source) {
        Ok(pairs) => {
            print_pairs(pairs, 0);
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}

fn print_pairs(pairs: pest::iterators::Pairs<'_, Rule>, indent: usize) {
    for pair in pairs {
        let rule = pair.as_rule();
        let text = pair.as_str();
        let text_preview: String = text.chars().take(50).collect();
        let text_preview = text_preview.replace('\n', "\\n");
        
        println!("{:indent$}{:?}: \"{}\"", "", rule, text_preview, indent = indent);
        
        print_pairs(pair.into_inner(), indent + 2);
    }
}
