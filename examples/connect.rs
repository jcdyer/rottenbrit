#![feature(ip_constructors)]

extern crate clap;
extern crate rottenbrit;

use std::net::Ipv6Addr;

use rottenbrit::serve;

fn main() {
    let opts = clap::App::new("RottenBrit")
        .version("0.1")
        .author("J. Cliff Dyer <jcd@sdf.org>")
        .about("Rotten Brit Bittorrent client")
        .arg(clap::Arg::with_name("port")
             .short("p")
             .long("port")
             .value_name("PORT")
             .help("Sets the port number to listen on")
             .takes_value(true))
        .get_matches();

    let port = opts.value_of("port")
        .map(|x| x.parse::<u16>().expect("u16"))
        .expect("port");

    println!("listening on [::1]:{}", port);
    serve((Ipv6Addr::unspecified(), port)).expect("served");
}
