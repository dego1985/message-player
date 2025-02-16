#![allow(non_local_definitions)]
use hdf5::types::VarLenArray;
use hdf5::{File, H5Type};

use std::path::PathBuf;
use std::thread;
use std::time::Instant;
use zenoh::prelude::sync::*;

#[derive(H5Type, Clone, PartialEq, Debug)] // register with HDF5
#[repr(C)]
struct Record {
    timestamp_micro: u64,  // タイムスタンプ
    data: VarLenArray<u8>, // 可変長バイナリデータ
}

pub fn node(session: std::sync::Arc<Session>, input: PathBuf, key_exprs: Vec<String>) {
    thread::spawn(move || {
        let start = Instant::now();

        let mut handles = vec![];

        for key_expr in key_exprs.iter().cloned() {
            let zpub = session.declare_publisher(key_expr.clone()).res().unwrap();
            let input = input.clone();

            let handle = thread::spawn(move || {
                let file = File::open(input).unwrap();
                let dataset = file.dataset(key_expr.as_str()).unwrap(); // open the dataset
                let data: Vec<Record> = dataset.read_raw().unwrap(); // read the dataset

                println!("start key_expr: {}", key_expr);
                for record in data.iter() {
                    let mut elapsed = start.elapsed().as_micros() as u64;
                    while record.timestamp_micro > elapsed {
                        std::thread::sleep(std::time::Duration::from_micros(10));
                        elapsed = start.elapsed().as_micros() as u64;
                    }
                    // println!("elapsed: {}, key_expr: {}", elapsed, key_expr);
                    // println!("timestamp: {}", record.timestamp_micro);
                    // println!("data: {:?}", record.data);

                    zpub.put(record.data.to_vec()).res().unwrap();
                }
                println!("end key_expr: {}", key_expr);
            });
            handles.push(handle);
        }

        // 全てのスレッドの終了を待機
        for handle in handles {
            handle.join().unwrap();
        }
    });
}
