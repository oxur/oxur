use oxur_ast::builder::AstBuilder;
use oxur_ast::sexp::Parser;

fn main() {
    let input = r#"
    (Crate
      :attrs ()
      :items ()
      :spans (ModSpans :inner-span (Span :lo 0 :hi 0))
      :id 0)
    "#;

    println!("Building simple Crate AST from S-expression...\n");

    match Parser::parse_str(input) {
        Ok(sexp) => {
            println!("✓ Parsed S-expression");

            let mut builder = AstBuilder::new();
            match builder.build_crate(&sexp) {
                Ok(crate_ast) => {
                    println!("✓ Successfully built Crate AST!");
                    println!("  Items: {}", crate_ast.items.len());
                    println!("  ID: {:?}", crate_ast.id);
                }
                Err(e) => {
                    eprintln!("Builder error: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}
