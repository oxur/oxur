use log;
use oxur::parser::sexp;
use twyg;

fn main() {
    let opts = twyg::LoggerOpts {
        colored: true,
        file: "".to_string(),
        level: "debug".to_string(),
        report_caller: true,
    };
    match twyg::setup_logger(&opts) {
        Ok(_) => {}
        Err(error) => panic!("Could not setup logger: {:?}", error),
    };
    let expression_1 = "((if (= (+ 3 (/ 9 3))
         (* 2 3))
            *
            /)
        456 123)";
    log::debug!("expression: {:?}\n", sexp::parse_expr(expression_1));
    log::debug!("\"{}\"", expression_1);
    log::debug!("evaled gives us: {:?}", sexp::eval_from_str(expression_1));
    // let expr_2 = r#"(mod thing1
    //     (pub fn hi ()
    //         (println! "hey")))

    //     (thing::hi)"#;
    // println!("expression: {:?}", sexp::parse_expr(expr_2));
}
