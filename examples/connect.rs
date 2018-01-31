#![feature(ip_constructors)]

extern crate clap;
extern crate reqwest;
extern crate rottenbrit;
extern crate url;

use std::io::Read;

use url::percent_encoding::{percent_encode as pe, DEFAULT_ENCODE_SET};

use rottenbrit::metainfo::{MetaInfo, get_info_hash};


fn percent_encode(input: &[u8]) -> String {
    pe(input, DEFAULT_ENCODE_SET).collect()
}


struct TrackerRequest {
    info_hash: Vec<u8>,
    peer_id: Vec<u8>,
    ip: Option<String>,
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
    event: Option<String>,
}


fn get_tracker(info_hash: Vec<u8>, url: &str, size: u64) -> Result<Vec<u8>, Box<::std::error::Error>> {
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
    println!("Reqwesting URL: {}", url);
    Ok(reqwest::get(&url).and_then(|mut response| {
        println!("Got a {} response", response.status());
        let mut buf = Vec::new();
        response.read_to_end(&mut buf).expect("Reading response");
        Ok(buf)
    })?)
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
    let mut f = std::fs::File::open(opts.value_of("torrent").unwrap())
        .expect("open torrent");
    f.read_to_end(&mut tordata).expect("read torrent");
    let info_hash = get_info_hash(tordata.clone()).expect("info hash");
    let mi = MetaInfo::from_bytes(&tordata).expect("parsing torrent file");
    println!("Got torrent: {:?}", &mi.announce);
    let bytes = get_tracker(info_hash.digest().bytes().to_vec(), &mi.announce, mi.info.length());
    println!("{}", ::std::string::String::from_utf8_lossy(&bytes.unwrap()));
}
