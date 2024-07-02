use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read};
use std::path::Path;

#[cfg(test)]
mod codingame_tests {
    use super::*;
    {% for test in tests %}
    #[test]
    fn {{ test.safe_label }}() {
        //{{ test.label }}
        test_from_folder("{{ test.safe_label }}");
    }
    {% endfor %}
}

fn test_from_folder(folder : &str) {
    let test_data_path = format!("tests/data/{}", folder);
    let input_path = format!("{test_data_path}/input.txt");
    let output_path = format!("{test_data_path}/output.txt");

    assert!(Path::new(&test_data_path).exists(), "{}", format!("Test folder {} missing", test_data_path));
    assert!(Path::new(&input_path).exists(), "{}", format!("Test folder {} missing", input_path));
    assert!(Path::new(&output_path).exists(), "{}", format!("Test folder {} missing", output_path));

    // Load the input data from the file
    let mut input_file = File::open(input_path).expect("Failed to open input file");
    let mut input_data = String::new();

    input_file.read_to_string(&mut input_data).expect("Failed to read input file");

    // Use a Cursor to simulate input
    let mut input = Cursor::new(input_data);
    let mut output = Vec::new();

    // Call the function with the test input and output
    assert!({{ safe_name }}::main(&mut input, &mut output).is_ok(), "Library returned an error");

    // Convert the output to a String
    let output_string = String::from_utf8(output).expect("Failed to convert output to String");

    // Read the lines from output.txt
    let file = File::open(output_path).expect("Failed to open output file");
    let file_reader = BufReader::new(file);

    // Iterate over the lines of the file and compare with the output
    let mut lines_from_file = file_reader.lines();
    for line in output_string.lines() {
        let expected_line = lines_from_file.next().expect("Not enough lines in output.txt");
        assert_eq!(line, expected_line.expect("Error reading line from output.txt"));
    }

    // Ensure there are no extra lines in output.txt
    assert!(lines_from_file.next().is_none());
}