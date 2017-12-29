#[macro_use] extern crate serde_derive;

extern crate mio;
extern crate serde;
extern crate serde_bencode;
extern crate serde_bytes;

pub mod metainfo;
pub mod peermsg;

use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::net::SocketAddr;

use mio::*;
use mio::net::{TcpListener, TcpStream};

// Setup some tokens to allow us to identify which event is
// for which socket.
const LISTENER: Token = Token(0);

enum State {
    New,
    Connected,
}

struct Piece;

struct Torrent<'a> {
    metainfo: metainfo::MetaInfo<'a>,
    pieces: Vec<Piece>,
    peers: Vec<Peer>
}

struct Peer {
    peer_id: Vec<u8>,
    choked: bool,
    interested: bool,
    bitfield: Vec<u8>,
}

pub fn serve<T: Into<SocketAddr>>(addr: T) -> Result<(), Box<Error>> {
    let addr = addr.into();

    // Setup the server socket
    let listener = TcpListener::bind(&addr).unwrap();

    // Create a poll instance
    let poll = Poll::new().unwrap();

    // Start listening for incoming connections
    poll.register(&listener, LISTENER, Ready::readable(),
                PollOpt::edge()).unwrap();

    // Create storage for events
    let mut events = Events::with_capacity(1024);
    let mut next_token_index = 1;
    let mut sockets = HashMap::new();
    let mut buf = [0; 1024];

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                LISTENER => {
                    loop {
                        match listener.accept() {
                            Ok((socket, _)) => {
                                let token = Token(next_token_index);
                                poll.register(
                                    &socket,
                                    token,
                                    Ready::readable(),
                                    PollOpt::edge()
                                )?;
                                next_token_index += 1;
                                sockets.insert(token, socket);
                            }
                            Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                                break;
                            }
                            err => panic!("err={:?}", err),
                        }
                    }
                }
                token => {
                    // Loop to drain the event buffer, because we are edge polling
                    loop {
                        match sockets.get_mut(&token).unwrap().read(&mut buf) {
                            Ok(0) => {
                                // Socket is closed; remove it from the hashmap
                                sockets.remove(&token);
                                break;
                            }
                            Ok(n) => {
                                if buf[n-1] == b'\n' {
                                    println!("got a newline");
                                    &buf[..(n - 1)].reverse();
                                } else {
                                    println!("got {}", buf[n-1]);
                                    &buf[..n].reverse();
                                }
                                sockets.get_mut(&token).unwrap().write(&buf[..n])?;
                                // handle data
                            }
                            Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                                // Socket is no longer ready; stop reading
                                break;
                            }
                            err => panic!("err={:?}", err),
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
