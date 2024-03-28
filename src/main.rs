use clap::{Arg, ArgAction, Command};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
};

fn read_shape<R: BufRead>(
    reader: R,
    region: &str,
    precinct: &str,
) -> HashMap<String, HashMap<String, usize>> {
    eprintln!("Reading dual-graph shapefile");
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

        eprint!("Processing sample {}\r", i - 2);
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
                                eprintln!("Error: {:?}", key);
                            }
                        }
                    }
                }
            }
            let json_line = json!({"sample": i-2, "assignment": assignments});
            writeln!(writer, "{}", json_line.to_string()).expect("Error writing to file");
        }
    }
    eprintln!();
}

fn canonicalize_jsonl_ben<S: BufRead, R: BufRead, W: Write>(
    shapefile_reader: S,
    reader: R,
    mut writer: W,
    mut logger: W,
    region: &str,
    precinct: &str,
) {
    let map = read_shape(shapefile_reader, region, precinct);

    let n_precincts = map.iter().map(|(_, v)| v.len()).sum::<usize>();

    writer
        .write_all(b"STANDARD BEN FILE")
        .expect("Error writing to file");
    for (i, line) in reader.lines().enumerate() {
        if i == 0 {
            continue;
        }
        if i < 3 {
            writeln!(logger, "{}", line.unwrap()).expect("Error writing to file");
            continue;
        }

        let mut assignments = vec![0 as u16; n_precincts];

        let line = line.unwrap();
        let data: serde_json::Value = serde_json::from_str(&line).expect("Error while reading");

        eprint!("Processing sample {}\r", i - 2);
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
                                eprintln!("Error: {:?}", key);
                            }
                        }
                    }
                }
            }
            let assign_ben = ben::encode::encode_ben_vec_from_assign(assignments);
            writer
                .write_all(assign_ben.as_slice())
                .expect("Error writing to file");
        }
    }
    eprintln!();
}

fn main() {
    let args = Command::new("canonicalize_jsonl")
        .version("0.1.0")
        .about("Canonicalize jsonl file")
        .arg(
            Arg::new("shapefile_json")
                .short('s')
                .long("shapefile_json")
                .help("Path to the shapefile json file")
                .required(true),
        )
        .arg(
            Arg::new("input_jsonl")
                .short('i')
                .long("input_jsonl")
                .help("Path to the input jsonl file")
                .required(false),
        )
        .arg(
            Arg::new("output_jsonl")
                .short('o')
                .long("output_jsonl")
                .help("Path to the output jsonl file")
                .required(true),
        )
        .arg(
            Arg::new("region")
                .short('r')
                .long("region")
                .help("Region name")
                .required(true),
        )
        .arg(
            Arg::new("precinct")
                .short('p')
                .long("precinct")
                .help("Precinct name")
                .required(true),
        )
        .arg(
            Arg::new("ben")
                .short('b')
                .long("ben")
                .help("Ben")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let stdin = io::stdin();
    let reader = stdin.lock();

    let shapefile_json = File::open(args.get_one("shapefile_json").map(String::as_str).unwrap())
        .expect("Error opening file");
    let shapefile_reader = BufReader::new(shapefile_json);

    let output_jsonl = File::create(args.get_one("output_jsonl").map(String::as_str).unwrap())
        .expect("Error creating file");
    let writer = BufWriter::new(output_jsonl);

    let logger_file = File::create(
        args.get_one("output_jsonl")
            .map(String::as_str)
            .unwrap()
            .to_owned()
            + ".settings",
    )
    .expect("Error creating file");
    let logger = BufWriter::new(logger_file);

    eprintln!(
        "Ben???? {:?}",
        *args.get_one::<bool>("ben").unwrap_or(&false)
    );

    if *args.get_one::<bool>("ben").unwrap_or(&false) {
        canonicalize_jsonl_ben(
            shapefile_reader,
            reader,
            writer,
            logger,
            args.get_one("region").map(String::as_str).unwrap(),
            args.get_one("precinct").map(String::as_str).unwrap(),
        );
    } else {
        canonicalize_jsonl(
            shapefile_reader,
            reader,
            writer,
            logger,
            args.get_one("region").map(String::as_str).unwrap(),
            args.get_one("precinct").map(String::as_str).unwrap(),
        );
    }
}
