fn parse_scores(csv_data: &str) -> Vec<u32> {
    csv_data
        .lines()
        .skip(1) // Skip header
        .filter_map(|line| {
            let mut parts = line.split(',');
            let _name = parts.next()?;
            let score_str = parts.next()?;
            let score: u32 = score_str.parse().ok()?;
            if score > 90 {
                Some(score)
            } else {
                None
            }
        })
        .collect()
}

fn total_high_scores(csv_data: &str) -> u32 {
    let scores = parse_scores(csv_data);
    scores.into_iter().fold(0, |acc, s| acc + s)
}

fn main() {
    let dataset = "Name,Score\nAlice,85\nBob,92\nCharlie,95\nDiana,88";
    
    // Step 2 validation
    let high_scores = parse_scores(dataset);
    println!("High scores: {:?}", high_scores);

    // Step 3 validation
    let total = total_high_scores(dataset);
    println!("Total of high scores: {}", total);
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
