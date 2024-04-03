use ben::encode::BenEncoder;
use clap::{Arg, ArgAction, Command};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
};

macro_rules! log {
    ($($arg:tt)*) => {{
        if let Ok(log_level) = std::env::var("RUST_LOG") {
            if log_level.to_lowercase() == "trace" {
                eprint!($($arg)*);
            }
        }
    }}
}

macro_rules! logln {
    ($($arg:tt)*) => {{
        if let Ok(log_level) = std::env::var("RUST_LOG") {
            if log_level.to_lowercase() == "trace" {
                eprintln!($($arg)*);
            }
        }
    }}
}

fn read_shape<R: BufRead>(
    reader: R,
    region: &str,
    precinct: &str,
) -> HashMap<String, HashMap<String, usize>> {
    logln!("Reading dual-graph shapefile");
    let data: serde_json::Value = serde_json::from_reader(reader).expect("Error while reading");

    // Note: This hashmap is going to rely on the way that the msms structures
    // the data and will assume the heirarchical structure therein
    let mut map: HashMap<String, HashMap<String, usize>> = HashMap::new();
    if let Some(nodes) = data["nodes"].as_array() {
        for value in nodes {
            if let (Some(reg), Some(prec), Some(id)) = (
                value[region].as_str(),
                value[precinct].as_str(),
                value["id"].as_u64(),
            ) {
                let entry = map.entry(reg.to_string()).or_insert_with(HashMap::new);
                entry.insert(prec.to_string(), id as usize);
            }
        }
    }
    map
}

fn canonicalize_jsonl<S: BufRead, R: BufRead, W: Write>(
    shapefile_reader: S,
    reader: R,
    mut writer: W,
    mut logger: W,
    region: &str,
    precinct: &str,
) {
    let map = read_shape(shapefile_reader, region, precinct);

    let n_precincts = map.iter().map(|(_, v)| v.len()).sum::<usize>();

    for (i, line) in reader.lines().enumerate() {
        if i == 0 {
            continue;
        }
        if i < 3 {
            writeln!(logger, "{}", line.unwrap()).expect("Error writing to file");
            continue;
        }

        let mut assignments = vec![0 as usize; n_precincts];

        let line = line.unwrap();
        let data: serde_json::Value = serde_json::from_str(&line).expect("Error while reading");

        log!("Processing sample {}\r", i - 2);
        if let Some(districts) = data["districting"].as_array() {
            for item in districts.iter() {
                if let Value::Object(dst) = item {
                    for (key, value) in dst {
                        let new_key = key
                            .trim_start_matches("[")
                            .trim_end_matches("]")
                            .split("\", \"")
                            .map(|s| s.replace("\"", ""))
                            .collect::<Vec<String>>();
                        match new_key.len() {
                            1 => {
                                for (_, v) in map.get(&new_key[0]).unwrap() {
                                    let id = v;
                                    assignments[*id] = value.as_u64().unwrap() as usize;
                                }
                            }
                            2 => {
                                let id = map.get(&new_key[0]).unwrap().get(&new_key[1]).unwrap();
                                assignments[*id] = value.as_u64().unwrap() as usize;
                            }
                            _ => {
                                logln!("Error: {:?}", key);
                            }
                        }
                    }
                }
            }
            let json_line = json!({"sample": i-2, "assignment": assignments});
            writeln!(writer, "{}", json_line.to_string()).expect("Error writing to file");
        }
    }
    logln!();
    logln!("Done!");
}

fn canonicalize_jsonl_ben<S: BufRead, R: BufRead, W: Write>(
    shapefile_reader: S,
    reader: R,
    writer: W,
    mut settings_log: W,
    region: &str,
    precinct: &str,
) {
    let map = read_shape(shapefile_reader, region, precinct);

    let n_precincts = map.iter().map(|(_, v)| v.len()).sum::<usize>();

    let mut ben_encoder = BenEncoder::new(writer);

    for (i, line) in reader.lines().enumerate() {
        if i == 0 {
            continue;
        }
        if i < 3 {
            writeln!(settings_log, "{}", line.unwrap()).expect("Error writing to file");
            continue;
        }

        let mut assignments = vec![0 as u16; n_precincts];

        let line = line.unwrap();
        let data: serde_json::Value = serde_json::from_str(&line).expect("Error while reading");

        log!("Processing sample {}\r", i - 2);
        if let Some(districts) = data["districting"].as_array() {
            for item in districts.iter() {
                if let Value::Object(dst) = item {
                    for (key, value) in dst {
                        let new_key = key
                            .trim_start_matches("[")
                            .trim_end_matches("]")
                            .split("\", \"")
                            .map(|s| s.replace("\"", ""))
                            .collect::<Vec<String>>();
                        match new_key.len() {
                            1 => {
                                for (_, v) in map.get(&new_key[0]).unwrap() {
                                    let id = v;
                                    assignments[*id] = value.as_u64().unwrap() as u16;
                                }
                            }
                            2 => {
                                let id = map.get(&new_key[0]).unwrap().get(&new_key[1]).unwrap();
                                assignments[*id] = value.as_u64().unwrap() as u16;
                            }
                            _ => {
                                logln!("Error: {:?}", key);
                            }
                        }
                    }
                }
            }
            ben_encoder.write_assignment(assignments);
        }
    }
    logln!();
    logln!("Done!");
}

fn main() {
    let args = Command::new("canonicalize_jsonl")
        .version("0.1.0")
        .about(concat!(
            "Allows for the conversion of a JSONL file stored in the ",
            "multi-scale map sampler output format into an assignment-sample JSONL ",
            "file or into a BEN file."
        ))
        .arg(
            Arg::new("dual_graph_json")
                .short('g')
                .long("graph-json")
                .help("Path to the dual-graph json file")
                .required(true),
        )
        .arg(
            Arg::new("input_jsonl")
                .short('i')
                .long("input-jsonl")
                .help("Path to the input jsonl file")
                .required(false),
        )
        .arg(
            Arg::new("output_file")
                .short('o')
                .long("output-file")
                .help("Path to the output jsonl file")
                .required(false),
        )
        .arg(
            Arg::new("region")
                .short('r')
                .long("region")
                .help("Region name")
                .required(true),
        )
        .arg(
            Arg::new("subregion")
                .short('s')
                .long("subregion")
                .help("Subregion name")
                .required(true),
        )
        .arg(
            Arg::new("ben")
                .short('b')
                .long("ben")
                .help("Ben")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Verbose output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("overwrite")
                .short('w')
                .long("overwrite")
                .help("Overwrite output file")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    if *args.get_one::<bool>("verbose").unwrap_or(&false) {
        env::set_var("RUST_LOG", "trace");
    }

    let reader = match args.get_one("input_jsonl").map(String::as_str) {
        Some(file_name) => {
            let file = File::open(file_name).expect("Error opening file");
            Box::new(BufReader::new(file)) as Box<dyn BufRead>
        }
        None => Box::new(io::stdin().lock()) as Box<dyn BufRead>,
    };

    let shapefile_json = File::open(args.get_one("dual_graph_json").map(String::as_str).unwrap())
        .expect("Error opening file");
    let shapefile_reader = BufReader::new(shapefile_json);

    let (writer, settings_log) = match args.get_one("output_file").map(String::as_str) {
        Some(file_name) => {
            let path = std::path::Path::new(file_name);
            if path.exists() && !*args.get_one::<bool>("overwrite").unwrap_or(&false) {
                eprint!(
                    "File {:?} already exists. Would you like to overwrite? y/[n]: ",
                    path
                );
                let mut response = String::new();
                io::stdin()
                    .read_line(&mut response)
                    .expect("Error reading response");

                if response.trim() != "y" {
                    std::process::exit(0);
                }
            }
            let file = File::create(file_name).expect("Error creating file");
            let settings_file =
                File::create(file_name.to_owned() + ".msms_settings").expect("Error creating file");
            (
                Box::new(BufWriter::new(file)) as Box<dyn Write>,
                Box::new(BufWriter::new(settings_file)) as Box<dyn Write>,
            )
        }
        None => (
            Box::new(io::stdout()) as Box<dyn Write>,
            Box::new(io::stderr()) as Box<dyn Write>,
        ),
    };

    if *args.get_one::<bool>("ben").unwrap_or(&false) {
        canonicalize_jsonl_ben(
            shapefile_reader,
            reader,
            writer,
            settings_log,
            args.get_one("region").map(String::as_str).unwrap(),
            args.get_one("subregion").map(String::as_str).unwrap(),
        );
    } else {
        canonicalize_jsonl(
            shapefile_reader,
            reader,
            writer,
            settings_log,
            args.get_one("region").map(String::as_str).unwrap(),
            args.get_one("subregion").map(String::as_str).unwrap(),
        );
    }
}
