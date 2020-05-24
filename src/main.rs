fn main() {
    if let Err(e) = lab_proto_gen::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
