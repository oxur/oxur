use oxur::ast::dump;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

fn main() {
    if let Err(error) = try_main() {
        let _ = writeln!(io::stderr(), "{}", error);
        process::exit(1);
    }
}

fn try_main() -> Result<(), dump::Error> {
    let mut args = env::args_os();
    let _ = args.next(); // executable name

    let filepath = match (args.next(), args.next()) {
        (Some(arg), None) => PathBuf::from(arg),
        _ => return Err(dump::Error::IncorrectUsage),
    };

    let code = fs::read_to_string(&filepath).map_err(dump::Error::ReadFile)?;
    let syntax = syn::parse_file(&code).map_err({
        |error| dump::Error::ParseFile {
            error,
            filepath,
            source_code: code,
        }
    })?;
    println!("{:#?}", syntax);

    Ok(())
}
