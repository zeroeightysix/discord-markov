use clap::Clap;
use serde_json::{Map, Value};
use std::io::{BufReader, BufRead};
use std::collections::HashMap;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use markov::Chain;

#[derive(Clap)]
#[clap(version = "1.0", author = "Ridan Vandenbergh")]
struct Opts {
    /// Whether or not to search for index.json in this directory, and to load it
    #[clap(long)]
    no_index: bool,
    /// Whether or not to print non-critical information
    #[clap(long)]
    verbose: bool,
    /// The file to read from and produce markov chains. If empty, this program will run in 'producing' mode, where it will try to find all messages and print them to stdout.
    input: Option<String>,
    #[clap(default_value = "10")]
    amount: usize
}

#[derive(Debug, Deserialize)]
struct MessageRecord {
    #[serde(rename = "ID")]
    id: u64,
    #[serde(rename = "Timestamp")]
    date: String,
    #[serde(rename = "Contents")]
    message: String,
    #[serde(rename = "Attachments")]
    attachments: String,
}

fn main() {
    let path = std::env::current_dir().unwrap();
    let opts = Opts::parse();
    let verbose = opts.verbose;

    if let Some(input) = opts.input {
        let file = File::open(input).expect("Couldn't open input file");
        let reader = BufReader::new(file);
        let mut chain: Chain<String> = Chain::new();
        let mut lines_read = 0;

        for line in reader.lines() {
            lines_read += 1;
            chain.feed_str(line.expect("Couldn't read line").as_str());
        }

        if verbose {
            println!("Producing {} markov chains from {} lines of input", opts.amount, lines_read);
        }

        for _ in 0..opts.amount {
            println!("{}", chain.generate_str());
        }
    } else {
        let index: Option<HashMap<String, String>> = if !opts.no_index {
            let index_json = path.join("index.json");
            if index_json.exists() {
                let file = File::open(index_json).expect("Couldn't open index.json file.");
                Some(match serde_json::from_reader::<BufReader<File>, Map<String, Value>>(BufReader::new(file)) {
                    Ok(map) => map,
                    Err(e) => {
                        println!("Couldn't read `index.json`: {}", e);
                        return;
                    }
                }.into_iter().map(|(key, value)| {
                    let value = match value {
                        Value::String(str) => str,
                        Value::Null => String::from("null in index"),
                        _ => panic!("Non-null and non-string value found in the index: I don't know what to do with this.")
                    };
                    (key, value)
                }).collect())
            } else {
                println!("Looks like there's no index.json in the current working directory.");
                println!("Please run this program from the `messages` folder in the discord data folder, or pass --no_index if you're sure you want to proceed.");
                return;
            }
        } else {
            if verbose {
                println!("Proceeding without index.");
            }
            None
        };

        for entry in path.read_dir().unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                let messages_csv = &path.join("messages.csv");
                if messages_csv.exists() {
                    let dirname = &path.file_name().unwrap().to_os_string().into_string().unwrap();
                    let file = File::open(messages_csv).expect(format!("Couldn't open messages.csv in {}", dirname).as_str());
                    match extract_messages(&file) {
                        Ok(messages) => {
                            if verbose {
                                if let Some(index) = &index {
                                    let indexed = index.get(dirname).map(|v| v.as_str()).unwrap_or(&"no index entry");
                                    println!("Read {} messages in `{}`", messages.len(), indexed);
                                }
                            } else {
                                for m in messages {
                                    println!("{}", m)
                                }
                            }
                        }
                        Err(e) => panic!("Couldn't extract messages: {}", e)
                    }
                } else if verbose {
                    println!("Directory {} didn't contain a messages.csv", path.into_os_string().into_string().unwrap())
                }
            }
        }
    }
}

fn extract_messages(file: &File) -> Result<Vec<String>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_reader(BufReader::new(file));
    let mut messages = Vec::new();
    for result in rdr.deserialize() {
        let record: MessageRecord = result?;
        messages.push(record.message);
    };

    Ok(messages)
}
