//pub mod operations;
//pub mod stack_lang;
mod stack_lang_2;

use std::env::args;
use std::fs;
use std::io::BufReader;
use std::io::Read;

fn main() {
    println!("{:?}", args().collect::<String>());
    if args().len() == 2 {
        let arg = args().collect::<Vec<String>>();
        let fd = fs::File::open(arg[1].as_str());
        if let Ok(fd) = fd {
            app(BufReader::new(fd));
        } else {
            println!("File not found!");
        }
        return;
    }

    app(BufReader::new(std::io::stdin()))
}

fn app<T: Read>(mut r: BufReader<T>) {
    let mut i = 1usize;
    let mut lang = stack_lang_2::ExprLang::new();
    loop {
        let expr = input::input(&mut r, format!("Eval[{}]-->", i).as_str()).unwrap();
        println!("Expr[~]: {}", expr);
        match lang.parse_syntax_stack(&expr) {
            Err(e) => eprint!("{:?}", e),
            _ => {}
        }
        match lang.eval() {
            Ok(ans) => println!("==> {}", ans),
            Err(e) => eprintln!("{:?}", e),
        }
        lang.reset();
        i += 1;
    }
}

mod input {
    use std::io::BufRead;
    use std::io::BufReader;
    use std::io::Read;

    pub fn input<T: Read>(r: &mut BufReader<T>, s: &str) -> std::io::Result<String> {
        println!("{}", s);
        let mut buf = String::with_capacity(1024);
        r.read_line(&mut buf).expect("Unable to read expr line!");
        Ok(buf.trim().to_owned())
    }
}
