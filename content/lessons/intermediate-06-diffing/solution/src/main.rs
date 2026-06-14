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
                matrix[i][j] = matrix[i - 1][j - 1] + 1;
            } else {
                matrix[i][j] = std::cmp::max(matrix[i - 1][j], matrix[i][j - 1]);
            }
        }
    }
    matrix
}

fn compute_diff(a: &str, b: &str) -> Vec<Diff> {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let matrix = lcs_matrix(a, b);

    let mut i = a_chars.len();
    let mut j = b_chars.len();
    let mut diffs = Vec::new();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && a_chars[i - 1] == b_chars[j - 1] {
            diffs.push(Diff::Keep(a_chars[i - 1]));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || matrix[i][j - 1] >= matrix[i - 1][j]) {
            diffs.push(Diff::Insert(b_chars[j - 1]));
            j -= 1;
        } else if i > 0 && (j == 0 || matrix[i][j - 1] < matrix[i - 1][j]) {
            diffs.push(Diff::Delete(a_chars[i - 1]));
            i -= 1;
        }
    }

    diffs.reverse();
    diffs
}

fn main() {
    let a = "kitten";
    let b = "sitting";
    let diffs = compute_diff(a, b);
    for d in diffs {
        println!("{:?}", d);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcs_matrix() {
        let m = lcs_matrix("abc", "ac");
        assert_eq!(m[3][2], 2);
    }

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
}
