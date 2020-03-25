pub fn parse_command(command: &str) -> Vec<String> {
    if command.is_empty() {
        return vec![]
    }

    let mut result = Vec::<String>::new();
    // let mut last_quote_index: Option<usize> = None;
    let mut quote_opened = false;
    let mut take_from: usize = 0;
    for (i, character) in command.char_indices() {
        let is_quote = character == '"';
        let is_space = character == ' ';

        // if last_space_index.unwrap_or(i)

        if is_quote {
            quote_opened = !quote_opened;
        }

        if is_space && !quote_opened {
            if take_from != i {
                result.push(command[take_from..i].to_string());
            }
            take_from = i + 1
        }
    }
    if take_from < command.chars().count() - 1{
        result.push(command[take_from..].to_string());
    }
    result
}


#[cfg(test)]
mod tests {
    use crate::command_utils::parse_command;
    use std::ops::Deref;

    fn test_parse_command(expected_output: &[&str]) {
        let input = expected_output.join(" ");

        let result = parse_command(&input);
        assert_eq!(
            expected_output.len(),
            result.len()
        );
        for (i, expected_output_item) in expected_output.iter().enumerate() {
            assert_eq!(expected_output_item.deref().trim(), result[i].as_str())
        }
    }

    #[test]
    fn test_single_arg_command() {
        test_parse_command(
            &["one"]
        );
    }

    #[test]
    fn test_single_arg_command_with_leading_and_trailing_spaces() {
        test_parse_command(
            &[" one "]
        );
    }


    #[test]
    fn test_single_multi_word_arg_command() {
        test_parse_command(
            &["\"this is a single argument\""]
        );
    }

    #[test]
    fn test_multi_word_multi_args_command() {
        test_parse_command(
            &[
                "\"this is a single argument\"",
                "\"this is an another single argument\""]
        );
    }

    #[test]
    fn test_two_word_command() {
        test_parse_command(
            &["one two"]
        );
    }
}
