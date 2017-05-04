extern crate iff;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use iff::{Chunk,TypeID};

fn testdata(file: &'static str) -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    while path.file_name().unwrap() != "target" {
        path.pop();
    }
    path.pop();
    path.push("tests");
    path.push("testdata");
    path.push(file);
    path
}

fn test_read_file(file: &'static str, expected_id: TypeID, expected_chunks: &[(TypeID,usize)]) {
    let mut vec = Vec::new();
    File::open(testdata(file)).unwrap().read_to_end(&mut vec).unwrap();

    let chunk = Chunk::new(&vec).unwrap();
    assert!(chunk.has_envelope_type(iff::FORM, expected_id));
    let data_chunks = chunk.data_chunks();
    assert_eq!(data_chunks.len(), expected_chunks.len());
    for (&(id,data),&(expected_id,expected_len)) in data_chunks.iter().zip(expected_chunks) {
        assert_eq!(id,expected_id);
        assert_eq!(data.len(), expected_len);
    }
}

#[allow(non_upper_case_globals)]
mod typeids {
    use iff::TypeID;
    pub const IFZS: TypeID = TypeID([b'I',b'F',b'Z',b'S']);
    pub const IFhd: TypeID = TypeID([b'I',b'F',b'h',b'd']);
    pub const CMem: TypeID = TypeID([b'C',b'M',b'e',b'm']);
    pub const Stks: TypeID = TypeID([b'S',b't',b'k',b's']);
    pub const IFRS: TypeID = TypeID([b'I',b'F',b'R',b'S']);
    pub const RIdx: TypeID = TypeID([b'R',b'I',b'd',b'x']);
    pub const GLUL: TypeID = TypeID([b'G',b'L',b'U',b'L']);
    pub const Fspc: TypeID = TypeID([b'F',b's',b'p',b'c']);
    pub const JPEG: TypeID = TypeID([b'J',b'P',b'E',b'G']);
    pub const PNG:  TypeID = TypeID([b'P',b'N',b'G',b' ']);
    pub const ZCOD: TypeID = TypeID([b'Z',b'C',b'O',b'D']);
    pub const IFmd: TypeID = TypeID([b'I',b'F',b'm',b'd']);
}

#[test]
fn test_read() {
    test_read_file("Advent.save", typeids::IFZS, &[
            (typeids::IFhd, 128),
            (typeids::CMem, 1229),
            (typeids::Stks, 380),
            ]);
    test_read_file("Alabaster.gblorb", typeids::IFRS, &[
            (typeids::RIdx, 316),
            (typeids::GLUL, 1660416),
            (typeids::IFmd, 2765),
            (typeids::Fspc, 4),
            (typeids::JPEG, 24039),
            (typeids::PNG, 37632),
            (typeids::PNG, 26019),
            (typeids::PNG, 19463),
            (typeids::PNG, 26615),
            (typeids::PNG, 74396),
            (typeids::PNG, 9946),
            (typeids::PNG, 23815),
            (typeids::PNG, 6413),
            (typeids::PNG, 13333),
            (typeids::PNG, 38434),
            (typeids::PNG, 20583),
            (typeids::PNG, 103116),
            (typeids::PNG, 62015),
            (typeids::PNG, 143187),
            (typeids::PNG, 261962),
            (typeids::PNG, 189566),
            (typeids::PNG, 150125),
            (typeids::PNG, 3038),
            (typeids::PNG, 39861),
            (typeids::PNG, 995),
            (typeids::PNG, 8124),
            (typeids::PNG, 108334),
            (typeids::PNG, 31170),
            (typeids::PNG, 46181),
            ]);
    test_read_file("BeingSteve.zblorb", typeids::IFRS, &[
            (typeids::RIdx, 16),
            (typeids::ZCOD, 123392),
            (typeids::IFmd, 1365),
            ]);
}

#[test]
fn test_write() {
    let mut vec = Vec::new();
    File::open(testdata("Advent.save")).unwrap().read_to_end(&mut vec).unwrap();

    let new_data = [1,2,3];
    let mut chunk = Chunk::new(&vec).unwrap();
    let mut out = Vec::new();
    chunk.write(&mut out).unwrap();
    assert_eq!(vec, out);

    out.clear();
    chunk.append_data(TypeID::from(b"TEST"), &new_data);
    chunk.write(&mut out).unwrap();

    let mut expected = vec.clone();
    expected.extend(&[b'T',b'E',b'S',b'T',0,0,0,3,1,2,3,0]);
    let size = expected.len() - 8;
    expected[4] = (size >> 24) as u8;
    expected[5] = (size >> 16) as u8;
    expected[6] = (size >> 8) as u8;
    expected[7] = size as u8;
    assert_eq!(expected, out);
}

#[test]
fn test_create() {
    let data = [1,2,3];
    let mut chunk = Chunk::create(iff::FORM, TypeID::from(b"TEST"));
    chunk.append_data(TypeID::from(b"TEST"), &data);

    let mut out = Vec::new();
    chunk.write(&mut out).unwrap();
    assert_eq!(out, vec![
            b'F',b'O',b'R',b'M',0,0,0,16,
            b'T',b'E',b'S',b'T',
            b'T',b'E',b'S',b'T',0,0,0,3,1,2,3,0]);
}

#[test]
fn mutable_type_id() {
    let mut id = TypeID::from(b"TEST");
    assert_eq!(id, TypeID::from(b"TEST"));
    id.0[0] = b'R'; // Not really useful, but the real benefit is being able
    const REST : TypeID = TypeID([b'R',b'E',b'S',b'T']); // to define consts.
    assert_eq!(id, REST);
}
