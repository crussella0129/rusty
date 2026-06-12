#[derive(Debug, PartialEq)]
struct LogEntry<'a> {
    level: &'a str,
    message: &'a str,
}

fn parse_log<'a>(line: &'a str) -> Option<LogEntry<'a>> {
    let mut parts = line.splitn(2, ':');
    let level = parts.next()?.trim();
    let message = parts.next()?.trim();
    Some(LogEntry { level, message })
}

fn find_error_message<'a>(entries: &[LogEntry<'a>]) -> Option<&'a str> {
    entries.iter().find(|e| e.level == "ERROR").map(|e| e.message)
}

fn main() {
    let log_data = "INFO: Booting system\nERROR: Disk failure\nWARN: High memory usage";
    
    // Step 2 validation
    let mut entries = Vec::new();
    for line in log_data.lines() {
        if let Some(entry) = parse_log(line) {
            entries.push(entry);
        }
    }
    println!("Parsed {} entries.", entries.len());

    // Step 3 validation
    if let Some(err) = find_error_message(&entries) {
        println!("Found error: {}", err);
    }
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
