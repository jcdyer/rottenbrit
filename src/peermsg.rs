fn push_u32(&mut vec, value: u32) {
    // Redo with byteorder crate
    vec.push((piece >> 24) as u8);
    vec.push((piece >> 16) as u8);
    vec.push((piece >> 8) as u8);
    vec.push(piece as u8);
}

// This should take a Peer and a Torrent object, and calculate the &[u8]s from them.
fn _peer_handshake(info_hash: &[u8], peer_id: &[u8]) -> Vec<u8> {
    let mut handshake = Vec::with_capacity(68);
    handshake.extend(b"\x13BitTorrent protocol");
    handshake.extend(&[0; 8]);
    handshake.extend(info_hash);
    handshake.extend(peer_id);
    handshake
}

fn keepalive() -> Vec<u8> {
    vec![0]
}

fn choke() -> Vec<u8> {
    let msg = Vec::with_capacity(5);
    push_u32(&mut msg, 1); // length
    msg.push(0); // choke id
    msg
}

fn unchoke() -> Vec<u8> {
    let msg = Vec::with_capacity(5);
    push_u32(&mut msg, 1); // length
    msg.push(1); // unchoke id
    msg
}

fn interested() -> Vec<u8> {
    let msg = Vec::with_capacity(5);
    push_u32(&mut msg, 1); // length
    msg.push(2); // interested id
    msg
}

fn not_interested() -> Vec<u8> {
    let msg = Vec::with_capacity(5);
    push_u32(&mut msg, 1); // length
    msg.push(3); // not interested id
    msg
}

fn have(piece: u32) -> Vec<u8> {
    let mut msg = Vec::with_capacity(6);
    push_u32(&mut msg, 5); // length
    msg.push(4); // have msg_id
    push_u32(&mut msg, piece);
    msg
}

fn bitfield() -> Vec<u8> {
    unimplemented!()
}

fn request(piece: u32, begin: u32, length: u32) -> Vec<u8> {
    let mut msg = Vec::with_capacity(17);
    push_u32(&mut msg, 13); // length
    msg.push(6); // request message id
    push_u32(&mut msg, piece);
    push_u32(&mut msg, begin);
    push_u32(&mut msg, length);
    msg
}

fn piece(piece: u32, begin: u32, data: &[u8]) -> Vec<u8> {
    let mut msg = Vec::with_capacity(13 + data.len());
    push_u32(&mut msg, 9 + data.len()); // length
    msg.push(7) // piece message id
    push_u32(&mut msg, piece);
    push_u32(&mut msg, begin);
    msg.extend(data);
    msg
}

fn cancel(piece: u32, begin: u32, length: u32) -> Vec<u8> {
    let mut msg = Vec::with_capacity(17);
    push_u32(&mut msg, 13); // length
    msg.push(8); // cancel msg_id
    push_u32(&mut msg, piece);
    push_u32(&mut msg, begin);
    push_u32(&mut msg, length);
    msg
}
