fn parse_scores(csv_data: &str) -> Vec<u32> {
    // csv_data
    //     // TODO: get an iterator over lines
    //     // TODO: skip the header row
    //     // .filter_map(|line| { ... })
    //     // TODO: collect into a Vec
    
    vec![] // Placeholder to make it compile
}

// TODO: Implement `total_high_scores(csv_data: &str) -> u32`

fn main() {
    let dataset = "Name,Score\nAlice,85\nBob,92\nCharlie,95\nDiana,88";
    
    // Step 2 validation
    let high_scores = parse_scores(dataset);
    println!("High scores: {:?}", high_scores);

    // Step 3 validation
    // let total = total_high_scores(dataset);
    // println!("Total of high scores: {}", total);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scores() {
        let data = "Name,Score\nAlice,85\nBob,92\nCharlie,95\nDiana,88";
        assert_eq!(parse_scores(data), vec![92, 95]);
    }
}
