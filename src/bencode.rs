use std::collections::BTreeMap;

#[derive(Debug)]
pub enum Bencode {
    Integer(i64),
    String(Vec<u8>),
    List(Vec<Bencode>),
    Dictionary(BTreeMap<String, Bencode>),
}
