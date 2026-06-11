use std::fs;

fn get_forward_difference(prev_data: &str, curr_data: &str) -> String {
    let prev_items: Vec<&str> = prev_data.split('~').collect();
    let curr_items: Vec<&str> = curr_data.split('~').collect();
    
    let mut diff = Vec::new();
    for item in curr_items {
        // (Faded) If the previous items do not contain this item, we want to collect it.
        // Replace `false` with the correct check using `prev_items.contains(&item)`.
        if /* TODO: check if prev_items does not contain item */ false {
            diff.push(item);
        }
    }
    diff.join("~")
}

fn process_diff_files(prev_file: &str, curr_file: &str) -> Result<String, std::io::Error> {
    // (Faded) Read the contents of the files using `fs::read_to_string` with error propagation.
    // Replace the empty strings below with actual fs calls!
    let prev_content = /* TODO: read prev_file */ String::new();
    let curr_content = /* TODO: read curr_file */ String::new();
    
    Ok(get_forward_difference(&prev_content, &curr_content))
}

// (Open) Implement `compile_unique_dataset` here!
// It should take files: &[&str], output_file: &str, and return Result<(), std::io::Error>.
// Remember to import std::collections::HashSet;

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

    // (Open) Uncomment the lines below once you've implemented `compile_unique_dataset`:
    /*
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
    */
}
