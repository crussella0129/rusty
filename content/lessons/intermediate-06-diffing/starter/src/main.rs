#[derive(Debug, PartialEq)]
enum Diff {
    Keep(char),
    Insert(char),
    Delete(char),
}

fn lcs_matrix(a: &str, b: &str) -> Vec<Vec<usize>> {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut matrix = vec![vec![0; b_chars.len() + 1]; a_chars.len() + 1];

    for i in 1..=a_chars.len() {
        for j in 1..=b_chars.len() {
            if a_chars[i - 1] == b_chars[j - 1] {
                // TODO: Step 2: Characters match
                // matrix[i][j] = ...
            } else {
                // TODO: Step 2: Characters do not match
                // matrix[i][j] = ...
            }
        }
    }
    matrix
}

// TODO: Step 3: Implement compute_diff
// fn compute_diff(a: &str, b: &str) -> Vec<Diff> { ... }

fn main() {
    /* Uncomment after completing Step 3
    let a = "kitten";
    let b = "sitting";
    let diffs = compute_diff(a, b);
    for d in diffs {
        println!("{:?}", d);
    }
    */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcs_matrix() {
        let m = lcs_matrix("abc", "ac");
        assert_eq!(m[3][2], 2);
    }

    /* Uncomment after completing Step 3
    #[test]
    fn test_compute_diff() {
        let diffs = compute_diff("ab", "ac");
        assert_eq!(
            diffs,
            vec![
                Diff::Keep('a'),
                Diff::Delete('b'),
                Diff::Insert('c')
            ]
        );
    }
    */
}
