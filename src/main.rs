extern crate clap;
extern crate dirs;
extern crate serde_json;

use clap::{App, Arg, SubCommand};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
// const FILE_LOC: &str = "/home/david/test.json";

type KVStore = HashMap<String, String>;

fn get_file_loc() -> PathBuf {
    match dirs::home_dir() {
        Some(home) => Path::new(&home).join("test.json"),
        None => print_err(&"Cannot find the user's home!".to_owned()),
    }
}

fn get_file() -> std::fs::File {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(false)
        .open(get_file_loc())
        .unwrap()
}

fn write_kv_file(m: &KVStore) {
    let mut file = get_file();
    file.set_len(0).unwrap();
    let s = serde_json::to_string_pretty(m).unwrap();
    file.write_all(s.as_bytes()).unwrap();
}

fn get_kv_store() -> KVStore {
    match serde_json::from_reader(get_file()) {
        Ok(s) => s,
        Err(_) => HashMap::new(),
    }
}

fn get_key(s: &str) -> Option<String> {
    let map: KVStore = serde_json::from_reader(get_file()).expect("Bad json file!");
    map.get(&s.to_owned()).cloned()
}

fn set_key(k: &str, v: &str) -> Option<String> {
    let mut map = get_kv_store();
    map.insert(k.to_owned(), v.to_owned());
    write_kv_file(&map);
    Some("".to_owned())
}

fn del_key(k: &str) -> Option<String> {
    let mut map = get_kv_store();
    // map.insert("3".to_owned(), "4".to_owned());
    map.remove(&k.to_owned());
    write_kv_file(&map);
    Some("".to_owned())
}

fn print_res(s: Option<String>) {
    match s {
        Some(s) => println!("{}", s),
        None => println!(),
    }
}

fn print_err(s: &str) -> ! {
    println!("{}", s);
    std::process::exit(1);
}

/// Fooar
fn main() {
    let matches = App::new("kv")
        .version("0.1")
        .author("David B")
        .about("Simple key, value storage")
        .subcommand(
            SubCommand::with_name("get")
                .about("Get key from storage")
                .arg(
                    Arg::with_name("key")
                        .help("key to get from storage")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("del")
                .about("Delete key and value from storage")
                .arg(
                    Arg::with_name("key")
                        .help("key to del from storage")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("set key to value in storage")
                .arg(
                    Arg::with_name("key")
                        .help("key to set in storage")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("val")
                        .help("key to set in storage")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .get_matches();
    if let Some(get) = matches.subcommand_matches("get") {
        let key = get.value_of("key").unwrap();
        let value = get_key(key);
        print_res(value);
    }
    if let Some(set) = matches.subcommand_matches("set") {
        let key = set.value_of("key").unwrap();
        let value = set.value_of("val").unwrap();
        set_key(key, value);
    }
    if let Some(del) = matches.subcommand_matches("del") {
        let key = del.value_of("key").unwrap();
        let value = del_key(key);
        print_res(value);
    }
}
