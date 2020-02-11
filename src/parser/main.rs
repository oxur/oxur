use oxur::parser::sexp;

fn main() {
    let expression_1 = "((if (= (+ 3 (/ 9 3))
         (* 2 3))
            *
            /)
        456 123)";
    println!("expression: {:?}", sexp::parse_expr(expression_1));
    println!(
        "\"{}\"\nevaled gives us: {:?}",
        expression_1,
        sexp::eval_from_str(expression_1)
    );
    // let expr_2 = r#"(mod thing1
    //     (pub fn hi ()
    //         (println! "hey")))

    //     (thing::hi)"#;
    // println!("expression: {:?}", sexp::parse_expr(expr_2));
}
