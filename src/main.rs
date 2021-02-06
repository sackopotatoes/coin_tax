use std::process;

fn main() {
    let filename = coin_tax::get_filename().unwrap();

    if let Err(e) = coin_tax::run(&filename) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}
