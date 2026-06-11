use std::fs;
use std::collections::HashSet;

fn get_forward_difference(prev_data: &str, curr_data: &str) -> String {
    let prev_items: Vec<&str> = prev_data.split('~').collect();
    let curr_items: Vec<&str> = curr_data.split('~').collect();
    
    let mut diff = Vec::new();
    for item in curr_items {
        // (Faded)
        if !prev_items.contains(&item) {
            diff.push(item);
        }
    }
    diff.join("~")
}

fn process_diff_files(prev_file: &str, curr_file: &str) -> Result<String, std::io::Error> {
    // (Faded)
    let prev_content = fs::read_to_string(prev_file)?;
    let curr_content = fs::read_to_string(curr_file)?;
    Ok(get_forward_difference(&prev_content, &curr_content))
}

fn get_full_difference(prev_data: &str, curr_data: &str) -> String {
    let prev_items: Vec<&str> = prev_data.split('~').filter(|s| !s.is_empty()).collect();
    let curr_items: Vec<&str> = curr_data.split('~').filter(|s| !s.is_empty()).collect();
    
    let mut diff = Vec::new();
    
    // Identify added items
    for item in &curr_items {
        if !prev_items.contains(item) {
            diff.push(format!("+{}", item));
        }
    }
    
    // Identify removed items
    for item in &prev_items {
        if !curr_items.contains(item) {
            diff.push(format!("-{}", item));
        }
    }
    
    diff.sort();
    diff.join("~")
}

fn process_full_diff_files(prev_file: &str, curr_file: &str) -> Result<String, std::io::Error> {
    let prev_content = fs::read_to_string(prev_file)?;
    let curr_content = fs::read_to_string(curr_file)?;
    Ok(get_full_difference(&prev_content, &curr_content))
}

fn get_positional_difference(prev_data: &str, curr_data: &str) -> String {
    let prev_items: Vec<&str> = prev_data.split('~').filter(|s| !s.is_empty()).collect();
    let curr_items: Vec<&str> = curr_data.split('~').filter(|s| !s.is_empty()).collect();
    
    let mut diff = Vec::new();
    
    // Additions and moves
    for (curr_idx, item) in curr_items.iter().enumerate() {
        match prev_items.iter().position(|&x| x == *item) {
            None => {
                diff.push(format!("+{}@{}", item, curr_idx));
            }
            Some(prev_idx) => {
                if prev_idx != curr_idx {
                    diff.push(format!(">{}@{}->{}", item, prev_idx, curr_idx));
                }
            }
        }
    }
    
    // Removals
    for (prev_idx, item) in prev_items.iter().enumerate() {
        if !curr_items.contains(item) {
            diff.push(format!("-{}@{}", item, prev_idx));
        }
    }
    
    diff.sort();
    diff.join("~")
}

fn process_positional_diff_files(prev_file: &str, curr_file: &str) -> Result<String, std::io::Error> {
    let prev_content = fs::read_to_string(prev_file)?;
    let curr_content = fs::read_to_string(curr_file)?;
    Ok(get_positional_difference(&prev_content, &curr_content))
}

// (Open)
fn compile_unique_dataset(files: &[&str], output_file: &str) -> Result<(), std::io::Error> {
    let mut uniques = HashSet::new();
    for file in files {
        let content = fs::read_to_string(file)?;
        for item in content.split('~') {
            if !item.is_empty() {
                uniques.insert(item.to_string());
            }
        }
    }
    
    let mut sorted_items: Vec<String> = uniques.into_iter().collect();
    sorted_items.sort();
    
    let joined = sorted_items.join("~");
    fs::write(output_file, joined)?;
    Ok(())
}

fn main() {
    // Faded setup
    fs::write("prev.txt", "Red~Orange~Yellow~Green~Blue").unwrap();
    fs::write("curr.txt", "Red~Orange~Yellow~Green~Blue~Purple").unwrap();
    
    match process_diff_files("prev.txt", "curr.txt") {
        Ok(diff) => println!("New items: {}", diff),
        Err(e) => println!("Error: {}", e),
    }
    
    let _ = fs::remove_file("prev.txt");
    let _ = fs::remove_file("curr.txt");

    // (Open - Step 3)
    fs::write("dataset1.txt", "Red~Orange~Yellow").unwrap();
    fs::write("dataset2.txt", "Yellow~Green~Blue").unwrap();
    fs::write("dataset3.txt", "Green~Blue~Purple").unwrap();
    
    match compile_unique_dataset(&["dataset1.txt", "dataset2.txt", "dataset3.txt"], "combined.txt") {
        Ok(_) => {
            let combined = fs::read_to_string("combined.txt").unwrap();
            println!("Combined uniques: {}", combined);
        }
        Err(e) => println!("Failed to compile dataset: {}", e),
    }
    
    let _ = fs::remove_file("dataset1.txt");
    let _ = fs::remove_file("dataset2.txt");
    let _ = fs::remove_file("dataset3.txt");
    let _ = fs::remove_file("combined.txt");

    // (Faded - Step 4)
    fs::write("prev_diff.txt", "Red~Orange~Yellow~Green").unwrap();
    fs::write("curr_diff.txt", "Red~Yellow~Green~Purple").unwrap();
    
    match process_full_diff_files("prev_diff.txt", "curr_diff.txt") {
        Ok(diff) => println!("Full diff: {}", diff),
        Err(e) => println!("Error: {}", e),
    }
    
    // (Open - Step 5)
    match process_positional_diff_files("prev_diff.txt", "curr_diff.txt") {
        Ok(diff) => println!("Positional diff: {}", diff),
        Err(e) => println!("Error: {}", e),
    }
    
    let _ = fs::remove_file("prev_diff.txt");
    let _ = fs::remove_file("curr_diff.txt");
}
