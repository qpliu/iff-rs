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

    if let Chunk::Envelope{ envelope_id, id, chunks } = Chunk::new(&vec).unwrap() {
        assert_eq!(envelope_id, iff::FORM);
        assert_eq!(id, expected_id);
        assert_eq!(chunks.len(), expected_chunks.len());
        for item in chunks.iter().zip(expected_chunks) {
            if let (&Chunk::Data{ id, data },&(expected_id,expected_len)) = item {
                assert_eq!(id, expected_id);
                assert_eq!(data.len(), expected_len);
            } else {
                assert!(false);
            }
        }
    } else {
        assert!(false);
    }
}

#[test]
fn test_read() {
    test_read_file("Advent.save", TypeID::from(b"IFZS"), &[
            (TypeID::from(b"IFhd"), 128),
            (TypeID::from(b"CMem"), 1229),
            (TypeID::from(b"Stks"), 380),
            ]);
    test_read_file("Alabaster.gblorb", TypeID::from(b"IFRS"), &[
            (TypeID::from(b"RIdx"), 316),
            (TypeID::from(b"GLUL"), 1660416),
            (TypeID::from(b"IFmd"), 2765),
            (TypeID::from(b"Fspc"), 4),
            (TypeID::from(b"JPEG"), 24039),
            (TypeID::from(b"PNG "), 37632),
            (TypeID::from(b"PNG "), 26019),
            (TypeID::from(b"PNG "), 19463),
            (TypeID::from(b"PNG "), 26615),
            (TypeID::from(b"PNG "), 74396),
            (TypeID::from(b"PNG "), 9946),
            (TypeID::from(b"PNG "), 23815),
            (TypeID::from(b"PNG "), 6413),
            (TypeID::from(b"PNG "), 13333),
            (TypeID::from(b"PNG "), 38434),
            (TypeID::from(b"PNG "), 20583),
            (TypeID::from(b"PNG "), 103116),
            (TypeID::from(b"PNG "), 62015),
            (TypeID::from(b"PNG "), 143187),
            (TypeID::from(b"PNG "), 261962),
            (TypeID::from(b"PNG "), 189566),
            (TypeID::from(b"PNG "), 150125),
            (TypeID::from(b"PNG "), 3038),
            (TypeID::from(b"PNG "), 39861),
            (TypeID::from(b"PNG "), 995),
            (TypeID::from(b"PNG "), 8124),
            (TypeID::from(b"PNG "), 108334),
            (TypeID::from(b"PNG "), 31170),
            (TypeID::from(b"PNG "), 46181),
            ]);
    test_read_file("BeingSteve.zblorb", TypeID::from(b"IFRS"), &[
            (TypeID::from(b"RIdx"), 16),
            (TypeID::from(b"ZCOD"), 123392),
            (TypeID::from(b"IFmd"), 1365),
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
