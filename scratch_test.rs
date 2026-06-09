use rusty_host::LspSession;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    let dir = PathBuf::from("c:/Users/charl/rusty/scratch_test_lsp");
    fs::create_dir_all(dir.join("src")).unwrap();
    let cargo_toml = r#"[package]
name = "temp-lsp-test"
version = "0.1.0"
edition = "2021"

[workspace]
"#;
    fs::write(dir.join("Cargo.toml"), cargo_toml).unwrap();
    let bad_code = r#"fn main() {
    let x = ;
}
"#;
    fs::write(dir.join("src/main.rs"), bad_code).unwrap();

    let session = LspSession::new(&dir).unwrap();
    
    // didOpen
    session.did_open(&dir.join("src/main.rs"), bad_code).unwrap();

    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(15) {
        if let Some(diags) = session.poll_diagnostics() {
            println!("Got diags: {:?}", diags);
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    
    fs::remove_dir_all(&dir).unwrap();
}
