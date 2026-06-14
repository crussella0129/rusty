// TODO: Add `use serde::Deserialize;` here

// TODO: Add `#[derive(Deserialize, Debug, PartialEq)]`
#[derive(Debug, PartialEq)]
struct PromptHeader {
    name: String,
    role: String,
    // Step 3 validation
    // temperature: Option<f32>,
}

fn parse_header(input: &str) -> Result<PromptHeader, toml::de::Error> {
    // TODO: Implement parsing using `toml::from_str`
    // toml::from_str(input)
    unimplemented!()
}

fn main() {
    let toml = r#"
name = "System Prompt"
role = "system"
"#;
    if let Ok(header) = parse_header(toml) {
        println!("Parsed header: {:?}", header);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let input = r#"
name = "User Prompt"
role = "user"
"#;
        let header = parse_header(input).unwrap();
        assert_eq!(header.name, "User Prompt");
        assert_eq!(header.role, "user");
    }

    // Step 3 validation
    /* Uncomment this block after completing Step 3
    #[test]
    fn test_optional_temperature() {
        let input1 = r#"
name = "Assistant"
role = "assistant"
"#;
        let header1 = parse_header(input1).unwrap();
        assert_eq!(header1.temperature, None);

        let input2 = r#"
name = "Assistant"
role = "assistant"
temperature = 0.8
"#;
        let header2 = parse_header(input2).unwrap();
        assert_eq!(header2.temperature, Some(0.8));
    }
    */
}
