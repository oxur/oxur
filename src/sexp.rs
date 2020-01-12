use lexpr::sexp;

fn main() {
    let address = sexp!(((name . "Jane Doe") (street . "4026 Poe Lane")));
    println!("{:#?}", address);
    let program = sexp!(fn main () (println! "Hello, world!"));
    println!("{:#?}", program);
}
