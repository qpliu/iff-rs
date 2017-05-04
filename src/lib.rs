use std::io::{Error,ErrorKind,Result,Write};

/// The Type ID of a chunk.
#[derive(Clone,Copy,Debug,Eq,Hash,Ord,PartialEq,PartialOrd)]
pub struct TypeID(pub [u8; 4]);

pub const FORM: TypeID = TypeID([b'F',b'O',b'R',b'M']);
pub const CAT:  TypeID = TypeID([b'C',b'A',b'T',b' ']);
pub const LIST: TypeID = TypeID([b'L',b'I',b'S',b'T']);
pub const PROP: TypeID = TypeID([b'P',b'R',b'O',b'P']);

impl TypeID {
    /// Test for Type IDs defined to contain nested chunks.
    pub fn is_envelope(self) -> bool {
        match self {
            FORM | CAT | LIST | PROP => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for TypeID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(),std::fmt::Error> {
        write!(f, "{}{}{}{}", self.0[0] as char, self.0[1] as char, self.0[2] as char, self.0[3] as char)
    }
}

impl<'a> From<&'a [u8; 4]> for TypeID {
    /// Create Type ID from an array.
    ///
    /// #Examples
    /// ```
    /// let id = iff::TypeID::from(b"TEXT");
    /// ```
    fn from(id: &'a [u8; 4]) -> TypeID {
        TypeID([id[0], id[1], id[2], id[3]])
    }
}

#[test]
fn test_type_id() {
    assert_eq!(FORM, TypeID::from(b"FORM"));
    assert!(FORM.is_envelope());
    assert!(TypeID::from(b"FORM").is_envelope());
    assert!(TypeID::from(b"CAT ").is_envelope());
    assert!(TypeID::from(b"LIST").is_envelope());
    assert!(TypeID::from(b"PROP").is_envelope());
    assert!(!TypeID::from(b"TEST").is_envelope());
}

pub enum Chunk<'a> {
    Envelope {
        envelope_id: TypeID,
        id: TypeID,
        chunks: Vec<Chunk<'a>>,
    },
    Data {
        id: TypeID,
        data: &'a [u8],
    },
}

impl<'a> Chunk<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self> {
        Chunk::new_chunk(data, 0, data.len())
    }

    fn new_chunk(data: &'a [u8], index: usize, last_index: usize) -> Result<Self> {
        if index + 8 > last_index {
            return Err(Error::new(ErrorKind::InvalidData, "invalid data"));
        }
        let id = Self::chunk_id(&data, index);
        let size = Self::chunk_size(&data, index+4);
        if index + 8 + size > last_index {
            return Err(Error::new(ErrorKind::InvalidData, "invalid data"));
        }
        if id.is_envelope() {
            if size < 4 {
                return Err(Error::new(ErrorKind::InvalidData, "invalid data"));
            }
            let data_id = Self::chunk_id(&data, index+8);
            let mut i = index + 12;
            let mut chunks = Vec::new();
            while i < index + 8 + size {
                let chunk = Self::new_chunk(&data, i, index+8+size)?;
                i += chunk.size();
                if i % 2 != 0 {
                    i += 1;
                }
                chunks.push(chunk);
            }
            Ok(Chunk::Envelope{envelope_id: id, id: data_id, chunks})
        } else {
            Ok(Chunk::Data{id, data: &data[index+8..index+8+size]})
        }
    }

    fn chunk_id(data: &[u8], index: usize) -> TypeID {
        TypeID([data[index],data[index+1],data[index+2],data[index+3]])
    }

    fn chunk_size(data: &[u8], index: usize) -> usize {
        (data[index] as usize) << 24 | (data[index+1] as usize) << 16
            | (data[index+2] as usize) << 8 | data[index+3] as usize
    }

    pub fn create(envelope_id: TypeID, id: TypeID) -> Self {
        Chunk::Envelope{ envelope_id, id, chunks: Vec::new() }
    }

    pub fn append_data(&mut self, id: TypeID, data: &'a [u8]) {
        if let &mut Chunk::Envelope{ envelope_id:_, id:_, ref mut chunks } = self {
            chunks.push(Chunk::Data{ id, data });
        } else {
            panic!("Cannot add nested chunks to a data chunk");
        }
    }

    pub fn append_chunk(&mut self, chunk: Chunk<'a>) {
        if let &mut Chunk::Envelope{ envelope_id:_, id:_, ref mut chunks } = self {
            chunks.push(chunk);
        } else {
            panic!("Cannot add nested chunks to a data chunk");
        }
    }

    pub fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        match self {
            &Chunk::Envelope{ envelope_id, id, ref chunks } => {
                w.write_all(&envelope_id.0[..])?;
                let size = self.size() - 8;
                w.write_all(&[(size >> 24) as u8, (size >> 16) as u8, (size >> 8) as u8, size as u8])?;
                w.write_all(&id.0[..])?;
                for chunk in chunks {
                    chunk.write(w)?;
                }
                if size % 2 != 0 {
                    w.write_all(&[0u8])?;
                }
            },
            &Chunk::Data{ id, data } => {
                w.write_all(&id.0[..])?;
                let size = self.size() - 8;
                w.write_all(&[(size >> 24) as u8, (size >> 16) as u8, (size >> 8) as u8, size as u8])?;
                w.write_all(data)?;
                if size % 2 != 0 {
                    w.write_all(&[0u8])?;
                }
            }
        }
        Ok(())
    }

    fn size(&self) -> usize {
        match self {
            &Chunk::Envelope{ envelope_id:_, id:_, ref chunks } => {
                let mut size = 12;
                for chunk in chunks {
                    size += chunk.size();
                    if size % 2 != 0 {
                        size += 1;
                    }
                }
                size
            },
            &Chunk::Data{ id:_, data } => 8 + data.len(),
        }
    }

    pub fn has_envelope_type(&self, envelope_type_id: TypeID, type_id: TypeID) -> bool {
        match self {
            &Chunk::Envelope{ envelope_id, id, chunks:_ } =>
                envelope_type_id == envelope_id && type_id == id,
            _ => false,
        }
    }

    pub fn has_data_type(&self, type_id: TypeID) -> bool {
        match self {
            &Chunk::Data{ id, data:_ } => type_id == id,
            _ => false,
        }
    }

    pub fn data_chunks(&self) -> Vec<(TypeID,&'a [u8])> {
        let mut vec = Vec::new();
        match self {
            &Chunk::Envelope{ envelope_id:_, id:_, ref chunks } => {
                for chunk in chunks {
                    match chunk {
                        &Chunk::Data{ id, data } => vec.push((id, data)),
                        _ => (),
                    }
                }
            },
            _ => (),
        }
        vec
    }
}
