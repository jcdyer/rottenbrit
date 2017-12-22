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
    #[serde(borrow)] pub announce: Cow<'a, str>,
    #[serde(borrow)]
    #[serde(rename = "announce-list")]
    pub announce_list: Vec<Cow<'a, str>>,
    #[serde(borrow)]
    #[serde(rename = "created by")]
    pub created_by: Option<Cow<'a, str>>,
    #[serde(borrow)]
    #[serde(rename = "url-list")]
    pub url_list: Option<Vec<Cow<'a, str>>>,
    #[serde(borrow)] pub comment: Option<Cow<'a, str>>,
    #[serde(rename = "creation date")] pub creation_date: Option<i64>,
    pub info: MiInfo<'a>,
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
    #[serde(rename = "piece length")] pub piece_length: u64,
    #[serde(deserialize_with = "pieces_from_bytes")] pub pieces: Vec<Sha1Hash>,
    pub length: u64,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MiMultiInfo<'a> {
    pub name: Cow<'a, str>,
    #[serde(rename = "piece length")] pub piece_length: u64,
    #[serde(deserialize_with = "pieces_from_bytes")] pub pieces: Vec<Sha1Hash>,
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
    path: Cow<'a, str>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::*;
    use std::collections::HashMap;

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
                    let blen = b.len();
                    let s = String::from_utf8(b).unwrap_or(format!("<bin:{}>", blen));
                    StrValue::Str(s)
                },
                Value::Int(i) => StrValue::Int(i),
            }
        }
    }
    #[test]
    fn into_metainfo() {
        let mut b = vec![];
        let mut f = File::open("data/archlinux-2017.12.01-x86_64.iso.torrent").unwrap();
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
        println!("{:?}", mi);
        assert_eq!(mi.announce, "http://tracker.archlinux.org:6969/announce")
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

    #[test]
    fn examine_strvalue() {
        let mut b = vec![];
        let filename = "data/These Systems Are Failing.torrent";
        let mut f = File::open(&filename).unwrap();
        f.read_to_end(&mut b).unwrap();
        let v: Value = from_bytes(&b).unwrap();
        let str_value: StrValue = v.into();
        println!("{:?}", str_value);
        assert!(false, "Success!");

    }
}
