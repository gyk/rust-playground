use std::io::stdin;

fn main() {
    println!("========");
    for arg in std::env::args() {
        println!("{}", arg);
    }
    println!("========");
    let mut s = String::new();
    stdin().read_line(&mut s).expect("Did not enter a correct string");
}
