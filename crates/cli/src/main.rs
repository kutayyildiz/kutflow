mod actors;
mod protocol;
mod stdio;

fn main() {
    if let Err(error) = stdio::run() {
        eprintln!("kutflow-cli: {error}");
        std::process::exit(1);
    }
}
