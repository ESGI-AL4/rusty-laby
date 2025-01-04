use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Invalid usage: encoder <encode|decode> <string>");
        return;
    }

    let operation = &args[1];
    let input = &args[2];

    let result = match operation.as_str() {
        "encode" => encode(input),
        "decode" => decode(input),
        _ => "Invalid operation".to_string(),
    };

    println!("Result: {}", result);
}

fn encode(input: &str) -> String {
    input.chars().map(|c| ((c as u8) + 1) as char).collect()
}

fn decode(input: &str) -> String {
    input.chars().map(|c| ((c as u8) - 1) as char).collect()
}
