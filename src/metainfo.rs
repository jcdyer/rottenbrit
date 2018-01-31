use std::borrow::Cow;
use std::io;
use std::iter::Peekable;
use std::str;

use serde_bencode::de::from_bytes;
use serde_bytes;
use sha1::Sha1;

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Sha1Hash(Vec<u8>);

impl Sha1Hash {
    pub fn new(input: Vec<u8>) -> Option<Sha1Hash> {
        if input.len() == 20 {
            Some(Sha1Hash(input))
        } else {
            None
        }
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.0
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MetaInfo<'a> {
    #[serde(borrow)]
    pub announce: Cow<'a, str>,
    pub info: Info<'a>,

    // Optional data
    #[serde(borrow)]
    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<Cow<'a, str>>>>,
    #[serde(borrow)]
    #[serde(rename = "url-list")]
    pub url_list: Option<Vec<Cow<'a, str>>>,
    #[serde(borrow)]
    #[serde(rename = "created by")]
    pub created_by: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub comment: Option<Cow<'a, str>>,
    #[serde(rename = "creation date")]
    pub creation_date: Option<i64>,
}

pub fn get_info_hash(source: Vec<u8>) -> io::Result<Sha1> {

    value_in_dict(source, b"info").map(|bytes| {
        let mut sha = Sha1::new();
        sha.update(&bytes);
        sha
    })
}

pub fn value_in_dict(source: Vec<u8>, key: &[u8]) -> io::Result<Vec<u8>> {
    let mut iter = source.into_iter().peekable();
    if iter.next() != Some(b'd') {
        return Err(io::Error::new(io::ErrorKind::Other, "Not a dict"));
    }
    let mut found = read_bytes(&mut iter)?;
    //let key = key.to_owned();
    while found.get(2) != Some(&key.to_owned()) {
        read_element(&mut iter)?;
        if iter.peek() == Some(&b'e') {
            return Err(io::Error::new(
                io::ErrorKind::Other, "Key not found"
            ))
        }
        found = read_bytes(&mut iter)?;
    }
    let chunks = read_element(&mut iter)?;
    let capacity = chunks.iter().fold(0, |sum, chunk| sum + chunk.len());
    let mut out = Vec::with_capacity(capacity);
    for chunk in chunks {
        out.extend(chunk);
    }
    Ok(out)
}

fn is_digit(n: u8) -> bool {
    n >= b'0' && n <= b'9'
}


pub fn read_element<I: Iterator<Item = u8>>(source: &mut Peekable<I>) -> io::Result<Vec<Vec<u8>>> {
    let mut reads = Vec::with_capacity(16);
    let initial = *source
        .peek()
        .ok_or(io::Error::new(io::ErrorKind::Other, "source was empty"))?;
    let result = match initial {
        b'd' => read_dict(source),
        b'l' => read_list(source),
        b'i' => read_integer(source),
        b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' => read_bytes(source),
        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            "Invalid start character",
        )),
    }?;
    reads.extend(result);
    Ok(reads)
}

pub fn read_integer<I: Iterator<Item = u8>>(source: &mut Peekable<I>) -> io::Result<Vec<Vec<u8>>> {
    let mut buf = Vec::with_capacity(16);
    let i = source.next().and_then(|c| if c != b'i' { None } else { Some(c) })
        .ok_or(io::Error::new(io::ErrorKind::Other, "integer didn't start with i"))?;
    buf.push(i);
    while let Some(byte) = source.next() {
        buf.push(byte);
        if byte == b'e' {
            return Ok(vec![buf]);
        }
        if byte < b'0' || byte > b'9' {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Non-numeric {} found in expected integer", byte),
            ));
        }
    }
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Integer is never closed",
    ))
}

pub fn read_bytes<I: Iterator<Item = u8>>(source: &mut Peekable<I>) -> io::Result<Vec<Vec<u8>>> {
    //! Redo this to use Peekable<I>
    let mut lengthbuf = Vec::with_capacity(10);
    while source
        .peek()
        .ok_or(io::Error::new(io::ErrorKind::Other, "No next"))? != &b':'
    {
        let next = source.next().unwrap(); // Infallible due to peek() above.
        if is_digit(next) {
            lengthbuf.push(next);
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "Not a number"));
        }
    }
    let colon = vec![
        source
            .next()
            .ok_or(io::Error::new(io::ErrorKind::Other, "No next"))?,
    ];
    // Safe due to is_digit() check above
    let bytes_length = unsafe { str::from_utf8_unchecked(&lengthbuf) }
        .parse()
        .unwrap();
    let mut data = Vec::with_capacity(bytes_length);
    for _ in 0..bytes_length {
        data.push(source
            .next()
            .ok_or(io::Error::new(io::ErrorKind::Other, "No next"))?);
    }
    Ok(vec![lengthbuf, colon, data])
}

pub fn read_list<I: Iterator<Item = u8>>(source: &mut Peekable<I>) -> io::Result<Vec<Vec<u8>>> {
    let mut buf = Vec::with_capacity(1024);
    let l = source
        .next()
        .ok_or(io::Error::new(io::ErrorKind::Other, "No data"))?;
    if l != b'l' {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Didn't find leading `l`",
        ))
    } else {
        buf.push(vec![l]);
        while source
            .peek()
            .ok_or(io::Error::new(io::ErrorKind::Other, "List didn't end"))? != &b'e'
        {
            buf.extend(read_element(source)?);
        }
        Ok(buf)
    }
}

pub fn read_dict<I: Iterator<Item = u8>>(source: &mut Peekable<I>) -> io::Result<Vec<Vec<u8>>> {
    let mut buf = Vec::with_capacity(1024);
    let d = source
        .next()
        .ok_or(io::Error::new(io::ErrorKind::Other, "No data"))?;
    if d != b'd' {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Didn't find leading `d`",
        ))
    } else {
        buf.push(vec![d]);
        while source
            .peek()
            .ok_or(io::Error::new(io::ErrorKind::Other, "Dict didn't end"))? != &b'e'
        {
            buf.extend(read_bytes(source)?);
            buf.extend(read_element(source)?);
        }
        Ok(buf)
    }
}

impl<'a> MetaInfo<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Option<MetaInfo> {
        from_bytes(bytes).ok()
    }

}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum Info<'a> {
    MiInfo(MiInfo<'a>),
    MiMultiInfo(MiMultiInfo<'a>),
}

impl <'a> Info<'a> {
    pub fn length(&self) -> u64 {
        match *self {
            Info::MiInfo(ref info) => info.length,
            Info::MiMultiInfo(ref info) => info.files.iter().fold(0, |sum, filedata| sum + filedata.length),
        }
    }
}
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MiInfo<'a> {
    pub name: Cow<'a, str>,
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    #[serde(deserialize_with = "pieces_from_bytes")]
    pub pieces: Vec<Sha1Hash>,
    pub length: u64,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MiMultiInfo<'a> {
    pub name: Cow<'a, str>,
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    #[serde(deserialize_with = "pieces_from_bytes")]
    pub pieces: Vec<Sha1Hash>,
    pub files: Vec<MiFileData<'a>>,
}

fn pieces_from_bytes<'de, D>(deserializer: D) -> Result<Vec<Sha1Hash>, D::Error>
where
    D: ::serde::de::Deserializer<'de>,
{
    println!("HI");
    let b: serde_bytes::ByteBuf = ::serde::de::Deserialize::deserialize(deserializer)?;
    println!("Got a b");
    b.chunks(20)
        .map(|x| Sha1Hash::new(x.to_vec()))
        .map(|x| x.ok_or_else(|| ::serde::de::Error::custom("oops")))
        .collect()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MiFileData<'a> {
    length: u64,
    path: Vec<Cow<'a, str>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn into_metainfo() {
        let mut b = vec![];
        let filename = "data/archlinux-2017.12.01-x86_64.iso.torrent";
        let mut f = File::open(&filename).unwrap();
        f.read_to_end(&mut b).expect("read");
        let mi = MetaInfo::from_bytes(&b).expect("deserialize");
        assert_eq!(mi.announce, "http://tracker.archlinux.org:6969/announce")
    }

    #[test]
    fn into_moby_metainfo() {
        let mut b = vec![];
        let mut f = File::open("data/These Systems Are Failing.torrent").unwrap();
        f.read_to_end(&mut b).expect("read");
        let mi = MetaInfo::from_bytes(&b).expect("deserialize");
        assert_eq!(
            mi.announce,
            "http://tracker.bundles.bittorrent.com/announce"
        )
    }

    #[test]
    fn error_if_pieces_not_multiples_of_20_chars() {
        let mut b = vec![];
        let mut f = File::open("data/archerror.torrent").unwrap();
        f.read_to_end(&mut b).expect("read");
        if let Some(mi) = MetaInfo::from_bytes(&b) {
            panic!("Unexpected success {:?}", mi);
        }
    }
}
