use oxur_ast::sexp::print_sexp;
use oxur_ast::sexp::Parser;

fn main() {
    let input = r#"(Crate :items ())"#;

    match Parser::parse_str(input) {
        Ok(sexp) => {
            println!("✓ Parsed successfully!");
            println!("S-expression:");
            println!("{}", print_sexp(&sexp));
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }

    // Test parsing a more complex S-expression
    let complex_input = r#"
    (Item
      :vis (Inherited)
      :ident (Ident :name "main")
      :kind (Fn
              :defaultness Final
              :sig (FnSig :decl (FnDecl :inputs () :output (Default)))
              :body nil))
    "#;

    match Parser::parse_str(complex_input) {
        Ok(sexp) => {
            println!("\n✓ Parsed complex S-expression successfully!");
            println!("Result:");
            println!("{}", print_sexp(&sexp));
        }
        Err(e) => {
            eprintln!("\nParse error: {}", e);
        }
    }
}
