use std::fs;
use serde_json::{from_str, json, to_string_pretty, Map, Value};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Watch for file changes and merge them automatically
    #[arg(short, long, default_value = "false")]
    watch: bool
}

// #[derive(Serialize, Deserialize)]
// enum LanguageValue {
//     String(String),
//     Object(Map<String, Value>)
// }

fn main() {
    let args = Args::parse();

    println!("Test: {}", args.watch);
    let files = vec!["test.json"];
    let mut merged_data: Map<String, Value> = Map::new();

    for file in files {
        let contents = fs::read_to_string(file).expect("Unable to read file");
        let data: Value = from_str(&contents).expect("Unable to parse JSON");

        let file_name = file.split('.').next().unwrap();
        merged_data.insert(file_name.to_string(), data);
    }
    let merged_json = json!(merged_data);
    let merged_json_str = to_string_pretty(&merged_json).expect("Unable to serialize JSON");

    fs::write("output.json", merged_json_str).expect("Unable to write file");


}

