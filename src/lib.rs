#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_bytes;

extern crate serde_bencode;

pub mod metainfo {
    use serde_bytes;
    use std::borrow::Cow;

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
        pub announce: Cow<'a, str>,
        #[serde(rename = "created by")]
        pub created_by: Option<Cow<'a, str>>,
        //#[serde(rename = "url-list")]
        //url_list: Option<Vec<Cow<'a, str>>>,
        pub comment: Option<Cow<'a, str>>,
        #[serde(rename = "creation date")]
        pub creation_date: Option<i64>,
        pub info: MiInfo<'a>,
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    #[serde(untagged)]
    pub enum Info<'a> {
        MiInfo(MiInfo<'a>),
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

    #[allow(dead_code)]
    fn pieces_from_bytes<'de, D>(deserializer: D) -> Result<Vec<Sha1Hash>, D::Error>
    where D: ::serde::de::Deserializer<'de> {
        println!("HI");
        let b: serde_bytes::ByteBuf = ::serde::de::Deserialize::deserialize(deserializer)?;
        println!("Got a b");
        b.chunks(20)
            .map(|x| Sha1Hash::new(x.iter().map(|x| *x).collect()))
            .map(|x| x.ok_or_else(|| ::serde::de::Error::custom("oops")))
            .collect()
    }

    #[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
    pub struct MiMultiInfo {
        pub name: String,
        #[serde(rename = "piece length")]
        pub piece_length: u64,
        // #[serde()] pub pieces: Vec<Sha1Hash>,
        pub files: Vec<MiFileData>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    pub struct MiFileData {
        length: u64,
        path: String,
    }
}

#[cfg(test)]
mod tests {
    use super::metainfo::*;
    use std::fs::File;
    use std::io::*;
    use std::str;

    use serde_bencode::value::Value;
    use serde_bencode::de::from_bytes;

    #[test]
    fn into_metainfo() {
        let mut b = vec![];
        let mut f = File::open("archlinux-2017.12.01-x86_64.iso.torrent").unwrap();
        f.read_to_end(&mut b).expect("read");
        let mi: MetaInfo = from_bytes(&b).expect("deser");
        println!("piece length: {}", mi.info.piece_length);
        assert!(false);
    }

    #[test]
    fn error_if_pieces_not_multiples_of_20_chars() {
        let mut b = vec![];
        let mut f = File::open("archerror.torrent").unwrap();
        f.read_to_end(&mut b).expect("read");
        if let Ok(mi) = from_bytes::<MetaInfo>(&b) {
            panic!("Unexpected success {:?}", mi);
        }
    }
    #[test]
    fn examine_value() {
        let mut b = vec![];
        let mut f = File::open("archlinux-2017.12.01-x86_64.iso.torrent").unwrap();
        f.read_to_end(&mut b).unwrap();
        let v: Value = from_bytes(&b).unwrap();
        if let &Value::Dict(ref d) = &v {
            let ks: Vec<&str> = d.keys().map(|k| str::from_utf8(&k).unwrap()).collect();
            println!("Keys: {:?}", ks);
            for field in ks {
                println!("{}: {}", field, match d[field.as_bytes()] {
                    Value::Dict(_) => "dict",
                    Value::Int(_) => "int",
                    Value::List(_) => "list",
                    Value::Bytes(_) => "bytes",
                });
            }
            for str_field in vec![&b"announce"[..], &b"comment"[..], &b"created by"[..]] {
                if let &Value::Bytes(ref value) = &d[&str_field[..]] {
                    println!("  {}: {} ", str::from_utf8(&str_field).unwrap(), str::from_utf8(&value).unwrap());
                }
            }
        }
        assert!(false, "Success!");
    }
    #[test]
    fn examine_info_value() {
        let mut b = vec![];
        let mut f = File::open("archlinux-2017.12.01-x86_64.iso.torrent").unwrap();
        f.read_to_end(&mut b).unwrap();
        let v: Value = from_bytes(&b).unwrap();
        if let &Value::Dict(ref d) = &v {
            if let &Value::Dict(ref id) = &d[&b"info"[..]] {
                let ks: Vec<&str> = id.keys().map(|k| str::from_utf8(&k).unwrap()).collect();
                println!("Keys: {:?}", ks);
                for field in ks {
                    println!("{}: {}", field, match id[field.as_bytes()] {
                        Value::Dict(_) => "dict",
                        Value::Int(_) => "int",
                        Value::List(_) => "list",
                        Value::Bytes(_) => "bytes",
                    });
                }
                for str_field in vec![&b"name"[..]] {
                    if let &Value::Bytes(ref value) = &id[&str_field[..]] {
                        println!("  {}: {} ", str::from_utf8(&str_field).expect("field"), str::from_utf8(&value).expect("value"));
                    }
                }

            }
            /*

            }
            if let &Value::Bytes(ref comment) = &d[&b"comment"[..]] {
                println!("  comment: {} ", str::from_utf8(&comment).unwrap());
            }
            */
        }
        assert!(false, "Success!");
    }
}
