extern crate clap;
extern crate serde_bencode;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use serde_bencode::de::from_bytes;
use serde_bencode::value::Value;

#[derive(Eq, PartialEq)]
enum StrValue {
    Dict(HashMap<String, StrValue>),
    List(Vec<StrValue>),
    Str(String),
    Int(i64),
}

impl ::std::fmt::Debug for StrValue {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            StrValue::Dict(ref map) =>
                write!(f, "{:?}", map),
            StrValue::List(ref vec) =>
                write!(f, "{:?}", vec),
            StrValue::Str(ref s) =>
                write!(f, "{:?}", s),
            StrValue::Int(ref i) =>
                write!(f, "{:?}", i),
        }

    }
}
impl ::std::convert::From<Value> for StrValue {
    fn from(input: Value) -> StrValue {
        match input {
            Value::Dict(map) => {
                let mut m = HashMap::new();
                for (key, val) in map {
                    let key = String::from_utf8_lossy(&key[..]).into_owned();
                    let val = val.into();
                    m.insert(key, val);
                }
                StrValue::Dict(m)
            }
            Value::List(v) => StrValue::List(v.into_iter().map(|x| x.into()).collect()),
            Value::Bytes(b) => {
                let length = b.len();
                let s = String::from_utf8(b).unwrap_or(format!("<bin:{}>", length));
                StrValue::Str(s)
            },
            Value::Int(i) => StrValue::Int(i),
        }
    }
}

fn main() {
    let mut b = vec![];
    let opts = clap::App::new("bparse")
        .about("Parses bencode files into a readable json-like format")
        .author("J. Cliff Dyer <jcd@sdf.org>")
        .arg(
            clap::Arg::with_name("file")
                .required(true)
                .index(1)
        )
        .get_matches();

    let filename = opts.value_of("file").unwrap();
    // let filename = "data/These Systems Are Failing.torrent";
    //let filename = "data/archlinux-2017.12.01-x86_64.iso.torrent";
    let mut f = File::open(&filename).unwrap();
    f.read_to_end(&mut b).unwrap();
    let v: Value = from_bytes(&b).unwrap();
    let str_value: StrValue = v.into();
    println!("{:?}", str_value);
    // Uncomment the following to see the reserialized file
    //assert!(false, "Success!");

}
