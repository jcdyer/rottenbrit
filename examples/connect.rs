#![feature(ip_constructors)]

extern crate clap;
extern crate reqwest;
extern crate rottenbrit;
extern crate url;

use std::io::Read;

use url::percent_encoding::{percent_encode as pe, DEFAULT_ENCODE_SET};

use rottenbrit::metainfo::{MetaInfo};

fn percent_encode(input: &[u8]) -> String {
    pe(input, DEFAULT_ENCODE_SET).collect()
}

struct TrackerRequest {
    info_hash: Vec<u8>,
    peer_id: Vec<u8>,
    ip: Option<String>,
    port: u16,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    event: Option<String>,
}

fn get_tracker(info_hash: Vec<u8>, url: &str, size: usize) -> Option<Vec<u8>> {
    let tr = TrackerRequest {
        info_hash,
        peer_id: b"rbxxxyyyyyzzzzz00000".to_vec(),
        ip: None,
        port: 6881,
        uploaded: 0,
        downloaded: 0,
        left: size,
        event: Some("started".to_string()),
    };
    let url = format!(
        "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&event={}",
        url,
        percent_encode(&tr.info_hash),
        percent_encode(&tr.peer_id),
        tr.port,
        tr.uploaded,
        tr.downloaded,
        tr.left,
        tr.event.unwrap(),
    );
    reqwest::get(&url).ok().and_then(|mut response| {
        println!("Got a {} response", response.status());
        let mut buf = Vec::new();
        response.read_to_end(&mut buf).ok()?;
        Some(buf)
    })
}

fn main() {
    let opts = clap::App::new("RottenBrit")
        .version("0.1")
        .author("J. Cliff Dyer <jcd@sdf.org>")
        .about("Rotten Brit Bittorrent client")
        .arg(
            clap::Arg::with_name("torrent")
                .required(true)
                .help("The path to a .torrent file."),
        )
        .get_matches();

    let mut tordata = Vec::new(); // Get file size
    let mut f =
        std::fs::File::open(opts.value_of("torrent").unwrap()).expect("could not open file");
    f.read_to_end(&mut tordata);
    let mi = MetaInfo::from_bytes(&tordata).expect("parsing torrent file");
    println!("Got torrent: {:?}", &mi.announce);
    let bytes = get_tracker(mi.info_hash().to_vec(), &mi.announce, 0);
    println!("{}", ::std::string::String::from_utf8_lossy(&bytes.unwrap()));
}
