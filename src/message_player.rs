#![allow(non_local_definitions)]
use std::thread;

use clap::{arg, command, value_parser, ArgAction};
use message_player::node;
use std::path::PathBuf;
use zenoh::config::Config;
use zenoh::prelude::sync::*;

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(
            arg!([INPUT_FILE] "some regular input")
                .value_parser(value_parser!(PathBuf))
                .required(true),
        )
        .arg(arg!(-k --key_expr "lists key_expr").action(ArgAction::Append))
        .get_matches();

    let input: &PathBuf = matches.get_one::<PathBuf>("INPUT_FILE").unwrap();
    let key_exprs: Vec<String> = matches
        .get_many::<String>("key_expr")
        .unwrap_or_default()
        .cloned() // 文字列の所有権を取得
        .collect();

    // initiate logging
    env_logger::init();

    let config = Config::default();
    let session: std::sync::Arc<Session> = zenoh::open(config).res().unwrap().into_arc();

    node(session, input.clone(), key_exprs);
    loop {
        thread::sleep(std::time::Duration::from_millis(10));
    }
}
