use std::collections::HashMap;

use erased_serde::Serialize;
use erased_serde::Deserializer;
use bincode::{serialize_into, deserialize_from};

use std::sync::Arc;

use std::any::Any;

use std::sync::RwLock;
//use tokio::sync::RwLock;


#[typetag::serde]
pub trait I3Serializeable {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[typetag::serde]
impl I3Serializeable for String {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

type I3FrameValue = Box<dyn I3Serializeable>;

pub struct I3Frame {
    data: HashMap<String, I3FrameValue>,
    parent: Option<Arc<I3Frame>>,
}

impl I3Frame {
    pub fn new() -> I3Frame {
        I3Frame{data: HashMap::new(), parent: None}
    }
    pub fn new_with_parent(parent: Arc<I3Frame>) -> I3Frame {
        I3Frame{data: HashMap::new(), parent: Some(parent)}
    }

    pub fn get<S: AsRef<str>, T: 'static>(&self, key: S) -> Result<&T, String>
    where
        S: std::fmt::Display,
        T: I3Serializeable
    {
        match self.data.get(key.as_ref()) {
            Some(x) => {
                let y: &dyn Any = x.as_any();
                if let Some(x) = y.downcast_ref::<T>() {
                    Ok(x)
                } else {
                    Err(format!("Key \"{}\" is not of specified type", key))
                }
            },
            None => {
                match &self.parent {
                    Some(frame) => {
                        let x = frame.get(key.as_ref());
                        if x.is_ok() {
                            x
                        } else {
                            Err(format!("No key \"{}\" in frame or parents", key))
                        }
                    },
                    None => Err(format!("No key \"{}\" in frame", key)),
                }
            }
        }
    }

    pub fn get_mut<S: AsRef<str>, T: 'static>(&mut self, key: S) -> Result<&mut T, String>
    where
        S: std::fmt::Display,
        T: I3Serializeable
    {
        match self.data.get_mut(key.as_ref()) {
            Some(x) => {
                let y: &mut dyn Any = x.as_any_mut();
                if let Some(x) = y.downcast_mut::<T>() {
                    Ok(x)
                } else {
                    Err(format!("Key \"{}\" is not of specified type", key))
                }
            },
            None => {
                Err(format!("No key \"{}\" in frame", key))
            }
        }
    }

    pub fn set<S: Into<String>, T: 'static>(&mut self, key: S, v: T) -> ()
    where
        T: I3Serializeable + Clone
    {
        self.data.insert(key.into(), Box::new(v.clone()));
    }

    pub fn write_to_stream<W>(&self, destination: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write
    {
        match serialize_into(destination, &self.data) {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
        }
    }

    pub fn read_from_stream<R>(&mut self, source: &mut R) -> std::io::Result<()>
    where
        R: std::io::Read
    {
        self.data = match deserialize_from(source) {
            Ok(d) => d,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
        };
        Ok(())
    }
}


struct I3File {
    reader: Option<std::io::BufReader<std::fs::File>>,
    writer: Option<std::io::BufWriter<std::fs::File>>,
}

enum FileMode {
    Read,
    Write,
    Append,
}

impl I3File {
    fn new(filename: &String, mode: FileMode) -> I3File {
        let file = match mode {
            FileMode::Read => std::fs::OpenOptions::new().read(true).open(filename),
            FileMode::Write => std::fs::OpenOptions::new().create(true).truncate(true).write(true).open(filename),
            FileMode::Append => std::fs::OpenOptions::new().create(true).append(true).open(filename),
        };
        match file {
            Ok(f) => match mode {
                FileMode::Read => I3File{reader: Some(std::io::BufReader::new(f)), writer: None},
                _ => I3File{reader: None, writer: Some(std::io::BufWriter::new(f))},
            },
            Err(e) => panic!("cannot open file {}: {:?}", filename, e),
        }
    }

    fn read_frame(&mut self) -> std::io::Result<Option<I3Frame>> {
        match &mut self.reader {
            Some(r) => {
                let mut frame = I3Frame::new();
                match frame.read_from_stream(r) {
                    Ok(_) => Ok(Some(frame)),
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::UnexpectedEof => Ok(None),
                        _ => Err(e),
                    },
                }
            },
            None => panic!("trying to read to a write-only file"),
        }
    }

    fn write_frame(&mut self, frame: &I3Frame) -> std::io::Result<()> {
        match &mut self.writer {
            Some(w) => frame.write_to_stream(w),
            None => panic!("trying to write to a read-only file"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_get_set() {
        let mut x = I3Frame::new();
        x.set("foo", String::from("Bar"));
        let v: &mut String = x.get_mut("foo").unwrap();
        assert_eq!(*v, "Bar");
        *v = String::from("baz");
        
        let w: &String = x.get("foo").unwrap();
        assert_eq!(*w, "baz");
    }

    #[test]
    fn frame_serialization() {
        let mut x = I3Frame::new();
        x.set("foo", String::from("Bar"));

        let mut file = Vec::new();
        x.write_to_stream(&mut file).unwrap();

        let mut y = I3Frame::new();
        y.read_from_stream(&mut file.as_slice()).unwrap();

        let v: &String = y.get("foo").unwrap();
        assert_eq!(*v, "Bar");
    }
}
