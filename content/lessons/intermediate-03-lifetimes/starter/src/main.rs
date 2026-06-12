#[derive(Debug, PartialEq)]
struct LogEntry {
    // TODO: Add lifetime parameter <'a> to LogEntry and change these fields to &'a str
    level: String,
    message: String,
}

// TODO: Update the signature to `fn parse_log<'a>(line: &'a str) -> Option<LogEntry<'a>>`
fn parse_log(line: &str) -> Option<LogEntry> {
    // TODO: implement zero-copy parsing using line.splitn(2, ':')
    None
}

// TODO: Implement `find_error_message<'a>(entries: &[LogEntry<'a>]) -> Option<&'a str>`

fn main() {
    let log_data = "INFO: Booting system\nERROR: Disk failure\nWARN: High memory usage";
    
    // Step 2 validation
    /* Uncomment this block after completing Step 2
    let mut entries = Vec::new();
    for line in log_data.lines() {
        if let Some(entry) = parse_log(line) {
            entries.push(entry);
        }
    }
    println!("Parsed {} entries.", entries.len());
    */

    // Step 3 validation
    /* Uncomment this block after completing Step 3
    if let Some(err) = find_error_message(&entries) {
        println!("Found error: {}", err);
    }
    */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log() {
        let entry = parse_log("DEBUG: Testing");
        assert!(entry.is_some());
        let e = entry.unwrap();
        assert_eq!(e.level, "DEBUG");
        assert_eq!(e.message, "Testing");
    }
}
