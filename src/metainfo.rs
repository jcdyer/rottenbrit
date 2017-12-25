use std::borrow::Cow;
use serde_bencode::de::from_bytes;
use serde_bytes;

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
        assert_eq!(mi.announce, "http://tracker.bundles.bittorrent.com/announce")
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
