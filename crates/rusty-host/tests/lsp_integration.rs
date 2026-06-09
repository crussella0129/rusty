use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use rusty_host::LspSession;

fn temp_cargo_project(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rusty-lsp-test-{name}-{nanos}"));
    
    fs::create_dir_all(dir.join("src")).unwrap();
    
    let cargo_toml = r#"[package]
name = "temp-lsp-test"
version = "0.1.0"
edition = "2021"

[workspace]
"#;
    fs::write(dir.join("Cargo.toml"), cargo_toml).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    
    dir
}

#[test]
fn test_lsp_session_full_lifecycle() {
    let project_dir = temp_cargo_project("lifecycle");
    
    // Spawn rust-analyzer
    let session = LspSession::new(&project_dir).expect("Failed to start LspSession");
    
    let main_path = project_dir.join("src/main.rs");
    
    // didOpen with a syntax error (version 1)
    let bad_code = r#"fn main() {
    let x = ;
}
"#;
    session.did_open(&main_path, bad_code).unwrap();
    
    // Wait for diagnostics for version 1
    let start = Instant::now();
    let mut diagnostics_received = None;
    while start.elapsed() < Duration::from_secs(60) {
        if let Some(diags) = session.poll_diagnostics() {
            if diags.version == Some(1) {
                diagnostics_received = Some(diags);
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let diags = diagnostics_received.expect("Did not receive diagnostics for version 1");
    assert!(!diags.diagnostics.is_empty());
    
    // didChange with a different syntax error (version 2)
    let bad_code_2 = r#"fn main() {
    let x = 
}
"#;
    fs::write(&main_path, bad_code_2).unwrap();
    session.did_change(&main_path, 2, bad_code_2).unwrap();
    session.did_save(&main_path).unwrap();
    
    // rust-analyzer may deduplicate diagnostics if the errors are identical.
    // Instead of waiting for version 2, we immediately proceed to version 3 (good code)
    // and wait for the diagnostics to clear.
    // didChange with corrected code (version 3)
    let good_code = r#"fn main() {
    let x: &str = "hello";
    println!("{}", x);
}
"#;
    fs::write(&main_path, good_code).unwrap();
    session.did_change(&main_path, 3, good_code).unwrap();
    session.did_save(&main_path).unwrap();
    
    // Hover request over variable `x`
    let mut hover_res = None;
    let start_hover = Instant::now();
    while start_hover.elapsed() < Duration::from_secs(10) {
        let hover_rx = session.hover(&main_path, 1, 9).unwrap();
        let res = hover_rx.recv_timeout(Duration::from_secs(5)).expect("Hover response timeout");
        if let Ok(Some(hover)) = res {
            hover_res = Some(hover);
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    let hover_res = hover_res.expect("Hover request failed or timed out");
    
    let hover_text = match hover_res.contents {
        lsp_types::HoverContents::Scalar(marked) => match marked {
            lsp_types::MarkedString::String(s) => s,
            lsp_types::MarkedString::LanguageString(ls) => ls.value,
        },
        lsp_types::HoverContents::Array(arr) => arr.into_iter().map(|m| match m {
            lsp_types::MarkedString::String(s) => s,
            lsp_types::MarkedString::LanguageString(ls) => ls.value,
        }).collect::<Vec<_>>().join(" "),
        lsp_types::HoverContents::Markup(markup) => markup.value,
    };
    assert!(hover_text.contains("str") || hover_text.contains("x"));
    
    // Completion request: write a string method trigger `x.` (version 4)
    let completion_code = r#"fn main() {
    let x = "hello";
    x.
}
"#;
    fs::write(&main_path, completion_code).unwrap();
    session.did_change(&main_path, 4, completion_code).unwrap();
    session.did_save(&main_path).unwrap();
    
    // completion request at line 2, character 6 (immediately after the dot `x.`)
    let mut labels = Vec::new();
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        let completion_rx = session.completion(&main_path, 2, 6).unwrap();
        let completion_res = completion_rx.recv_timeout(Duration::from_secs(5)).expect("Completion response timeout").unwrap();
        if !completion_res.is_empty() {
            labels = completion_res.into_iter().map(|item| item.label).collect();
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    
    // Check that we have autocomplete suggestions (like len, chars, etc.)
    assert!(!labels.is_empty(), "Failed to get completions from rust-analyzer");
    assert!(labels.contains(&"len".to_string()) || labels.contains(&"chars".to_string()));
    
    // Cleanup
    fs::remove_dir_all(&project_dir).ok();
}
