#[macro_use]
extern crate clap;
extern crate dirs;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
use clap::{App, AppSettings, Arg, SubCommand};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

type KV = HashMap<String, String>;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum OpType {
    Get,
    Set,
    Del,
}

#[derive(Serialize, Deserialize, Debug)]
struct Hook {
    name: String,
    cmd_name: String,
    run_on: OpType,
    key: String,
}

#[derive(Serialize, Deserialize, Default)]
struct KVStore {
    kvs: KV,
    cmds: KV,
    hooks: Vec<Hook>,
}

impl FromStr for OpType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "get" => Ok(OpType::Get),
            "set" => Ok(OpType::Set),
            "del" => Ok(OpType::Del),
            _ => Err("No match found!"),
        }
    }
}

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

fn write_file(m: &KVStore) {
    let mut file = get_file();
    file.set_len(0).unwrap();
    let s = serde_json::to_string_pretty(m).unwrap();
    file.write_all(s.as_bytes()).unwrap();
}

fn run_command(cmd_name: &str, cmd: &str) {
    let shell = match env::var("SHELL") {
        Ok(s) => s,
        Err(_) => "bash".to_owned(),
    };
    if let Err(e) = Command::new(shell).arg("-c").arg(&cmd).spawn() {
        let err_msg = format!(
            "Error! Failed to run '{}' with error:\n {:?}",
            cmd_name,
            e.description()
        );
        print_err(&err_msg[..]);
    }
}

fn run_hooks(key_name: &str, current_op: &OpType) {
    let kvstore = get_store();
    let hooks_to_run = kvstore
        .hooks
        .iter()
        .filter(|&x| x.run_on == *current_op && x.key == key_name);
    for hook in hooks_to_run {
        match get_key(&hook.cmd_name[..], &kvstore.cmds) {
            Some(cmd) => run_command(&hook.cmd_name, &cmd),
            None => println!("Error! Bad hook! Hook {:?} has no cmd!", hook.name),
        }
    }
}

fn get_store() -> KVStore {
    match serde_json::from_reader(get_file()) {
        Ok(s) => s,
        Err(_) => Default::default(),
    }
}

fn rm_hook(name: &str) {
    let mut kvstore = get_store();
    match kvstore.hooks.iter().position(|x| x.name == name) {
        Some(pos) => {
            kvstore.hooks.remove(pos);
        }
        None => {
            let err_msg = format!("Error! Hook {} does not exist!", name);
            print_err(&err_msg[..]);
        }
    }
    write_file(&kvstore);
}

fn add_hook(name: String, cmd_name: String, run_on: OpType, key: String) {
    let mut kvstore = get_store();
    if kvstore.hooks.iter().filter(|&x| x.name == name).count() > 0 {
        let err_msg = format!(
            "Error! {} already exists. To delete it try\n kv cmd del-hook {}",
            name, name
        );
        print_err(&err_msg[..]);
    }
    let new_hook = Hook {
        name,
        cmd_name,
        run_on,
        key,
    };

    kvstore.hooks.push(new_hook);
    write_file(&kvstore)
}

fn get_key(s: &str, map: &KV) -> Option<String> {
    map.get(&s.to_owned()).cloned()
}

fn set_key(k: &str, v: &str, map: &mut KV) {
    map.insert(k.to_owned(), v.to_owned());
}

fn del_key(k: &str, map: &mut KV) -> Option<String> {
    map.remove(&k.to_owned())
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
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .about("Get key from storage")
        .subcommand(
            SubCommand::with_name("cmd")
                .about("Add, Run, and hook commands")
                .subcommand(
                    SubCommand::with_name("run")
                        .about("Run commands <cmd-name>")
                        .arg(Arg::with_name("cmd-name").takes_value(true).required(true)),
                )
                .subcommand(
                    SubCommand::with_name("add")
                        .about("Add command with name <cmd-name>, and value <cmd-value>")
                        .arg(Arg::with_name("cmd-name").takes_value(true).required(true))
                        .arg(Arg::with_name("cmd-value").takes_value(true).required(true)),
                )
            .subcommand(
                SubCommand::with_name("add-hook")
                    .about("Add hook with name <hook-name> to run <cmd-name> when [key] are --<trigger>=<get,set,del>")
                    .arg(Arg::with_name("hook-name").takes_value(true).required(true))
                    .arg(Arg::with_name("cmd-name").takes_value(true).required(true))
                    .arg(Arg::with_name("trigger").takes_value(false).required(true).possible_values(&["get", "set", "del"]))
                    .arg(Arg::with_name("key").takes_value(true).required(true))
            )
            .subcommand(
                SubCommand::with_name("del-hook")
                    .about("Remove hook with name <hook-name>")
                    .arg(Arg::with_name("hook-name").takes_value(true).required(true))
            )
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get key from storage")
                .help(
                    r#"kv get <key>

Get the value of <key> from storage

Example:
~> kv set my-key my-key-value
~> kv get my-key
my-key-value
"#,
                )
                .arg(
                    Arg::with_name("key")
                        .help("key to get from storage")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("del")
                .help(
                    r#"kv del <key>

Delete <key> in storage (and its value)

Example:
~> kv set my-key my-key-value
~> kv del my-key
~> kv get my-key

~>
"#,
                )
                .about("Delete key and value from storage")
                .arg(
                    Arg::with_name("key")
                        .help("key to delete from storage")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("set key to value in storage")
                .help(
                    r#"kv set <key> <val>

Set <key> to <val> in storage.

Example:
~> kv set my-key my-key-value
~> kv get my-key
my-key-value
"#,
                )
                .arg(
                    Arg::with_name("key")
                        .help("key to set in storage")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("val")
                        .help("<val> you wish to set <key> to.")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .get_matches();
    let mut kvstore = get_store();
    if let Some(get) = matches.subcommand_matches("get") {
        let key = get.value_of("key").unwrap();
        let value = get_key(key, &kvstore.kvs);
        print_res(value);
        run_hooks(key, &OpType::Get);
    }
    if let Some(set) = matches.subcommand_matches("set") {
        let key = set.value_of("key").unwrap();
        let value = set.value_of("val").unwrap();
        set_key(key, value, &mut kvstore.kvs);
        write_file(&kvstore);
        run_hooks(key, &OpType::Set);
    }
    if let Some(del) = matches.subcommand_matches("del") {
        let key = del.value_of("key").unwrap();
        let value = del_key(key, &mut kvstore.kvs);
        write_file(&kvstore);
        print_res(value);
        run_hooks(key, &OpType::Del);
    }
    if let Some(cmd) = matches.subcommand_matches("cmd") {
        if let Some(m_run) = cmd.subcommand_matches("run") {
            let cmd_name = m_run.value_of("cmd-name").unwrap();
            let cmd_value = get_key(cmd_name, &kvstore.cmds);
            match cmd_value {
                Some(v) => run_command(cmd_name, &v),
                None => println!("Error! Command {} does not exist!", cmd_name),
            }
        }
        if let Some(m_add) = cmd.subcommand_matches("add") {
            let cmd_name = m_add.value_of("cmd-name").unwrap();
            let cmd_value = m_add.value_of("cmd-value").unwrap();
            set_key(cmd_name, cmd_value, &mut kvstore.cmds);
            write_file(&kvstore);
        }
        if let Some(m_del_hook) = cmd.subcommand_matches("del-hook") {
            let hook_name = m_del_hook.value_of("hook-name").unwrap();
            rm_hook(hook_name);
        }

        if let Some(m_add_hook) = cmd.subcommand_matches("add-hook") {
            let hook_name = m_add_hook.value_of("hook-name").unwrap();
            let cmd_name = m_add_hook.value_of("cmd-name").unwrap();
            let trigger_op = value_t!(m_add_hook, "trigger", OpType).unwrap();
            let key = m_add_hook.value_of("key").unwrap();
            add_hook(
                hook_name.to_owned(),
                cmd_name.to_owned(),
                trigger_op,
                key.to_owned(),
            )
        }
    }
}
