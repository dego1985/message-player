#![allow(non_local_definitions)]
use std::thread;
use std::time::Instant;

use clap::{arg, command, value_parser, ArgAction};
use std::path::PathBuf;
use zenoh::config::Config;
use zenoh::prelude::sync::*;

use hdf5::types::VarLenArray;
use hdf5::{File, H5Type};

#[derive(H5Type, Clone, PartialEq, Debug)] // register with HDF5
#[repr(C)]
struct Record {
    timestamp_micro: u64,  // タイムスタンプ
    data: VarLenArray<u8>, // 可変長バイナリデータ
}

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(
            arg!([INPUT_FILE] "some regular input")
                .value_parser(value_parser!(PathBuf))
                .required(true),
        )
        .arg(arg!(-k --key_expr "lists key_expr").action(ArgAction::Append))
        .get_matches();

    let input = matches.get_one::<PathBuf>("INPUT_FILE").unwrap();
    let key_exprs: Vec<String> = matches
        .get_many::<String>("key_expr")
        .unwrap_or_default()
        .cloned() // 文字列の所有権を取得
        .collect();

    // initiate logging
    env_logger::init();

    let config = Config::default();
    let session = zenoh::open(config).res().unwrap().into_arc();

    let start = Instant::now();

    let mut handles = vec![];

    for key_expr in key_exprs.iter().cloned() {
        let zpub = session.declare_publisher(key_expr.clone()).res().unwrap();
        let input = input.clone();

        let handle = thread::spawn(move || {
            let file = File::open(input).unwrap();
            let dataset = file.dataset(key_expr.as_str()).unwrap(); // open the dataset
            let data: Vec<Record> = dataset.read_raw().unwrap(); // read the dataset

            for record in data.iter() {
                let mut elapsed = start.elapsed().as_micros() as u64;
                while record.timestamp_micro > elapsed {
                    std::thread::sleep(std::time::Duration::from_micros(10));
                    elapsed = start.elapsed().as_micros() as u64;
                }
                // println!("elapsed: {}", elapsed);
                // println!("timestamp: {}", record.timestamp_micro);
                // println!("data: {:?}", record.data);

                zpub.put(record.data.to_vec()).res().unwrap();
            }
        });
        handles.push(handle);
    }

    // 全てのスレッドの終了を待機
    for handle in handles {
        handle.join().unwrap();
    }
}
