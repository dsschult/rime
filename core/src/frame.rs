use std::any::Any;
use std::sync::Arc;
use std::collections::HashMap;
use bincode::{serialize_into, deserialize_from};

/// Base trait for any serializable object in an Frame.
#[typetag::serde]
pub trait Serializeable {
    /// Convert to an `Any` reference.
    fn as_any(&self) -> &dyn Any;
    /// Convert to a mutable `Any` reference.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[typetag::serde]
impl Serializeable for String {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[typetag::serde]
impl Serializeable for u8 {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

type FrameValue = Box<dyn Serializeable>;

/// Frame - a bag of holding for event data.
///
/// The primary interface is similar to a map, but with all values
/// guaranteed to be serializable. The key must be a `String`,
/// while the value is any type implementing the `Serializable`
/// trait.
///
/// One magic feature is the ability to stack frames together,
/// such that read-only access is granted to any object higher up in
/// the stack. This makes a great way to share common data between
/// two Frames, even across threads.
///
/// # Examples
///
/// Get and set values on an Frame:
///
/// ```
/// use core::Frame;
/// 
/// let mut x = Frame::new();
/// x.set("foo", String::from("Bar"));
/// let v: &mut String = x.get_mut("foo").unwrap();
/// assert_eq!(*v, "Bar");
/// *v = String::from("baz");
/// 
/// let w: &String = x.get("foo").unwrap();
/// assert_eq!(*w, "baz");
/// ```
///
/// Serialize and deserialize an Frame:
///
/// ```
/// use core::Frame;
/// 
/// let mut x = Frame::new();
/// x.set("foo", String::from("Bar"));
/// 
/// let mut file = Vec::new();
/// x.write_to_stream(&mut file).unwrap();
/// 
/// let mut y = Frame::new();
/// y.read_from_stream(&mut file.as_slice()).unwrap();
/// 
/// let v: &String = y.get("foo").unwrap();
/// assert_eq!(*v, "Bar");
/// ```
pub struct Frame {
    data: HashMap<String, FrameValue>,
    parent: Option<Arc<Frame>>,
}

impl Frame {
    /// Create a new Frame with default values.
    pub fn new() -> Frame {
        Frame{data: HashMap::new(), parent: None}
    }

    /// Create a new Frame with default values, but with a parent frame.
    ///
    /// # Arguments
    /// * `parent` - a read-only parent frame 
    pub fn new_with_parent(parent: Arc<Frame>) -> Frame {
        Frame{data: HashMap::new(), parent: Some(parent)}
    }

    /// Get a read-only reference to a value at a specific key.
    ///
    /// This function will also check the parent frame (recursively)
    /// for a match, and return that value.
    ///
    /// Note: the return type must be known ahead of time.
    ///
    /// # Arguments
    /// * `key` - the key to lookup
    ///
    /// # Errors
    /// * A type error, if the key exists but the value cannot be
    ///   coerced to the return type.
    /// * A not-found error if the key cannot be found in the frame
    ///   any parent frames.
    pub fn get<S: AsRef<str>, T: 'static>(&self, key: S) -> Result<&T, String>
    where
        S: std::fmt::Display,
        T: Serializeable
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

    /// A mutable version of [`get`](#method.get), which only looks at the current frame.
    ///
    /// Since the parent is read-only, it is not considered for key matches.
    pub fn get_mut<S: AsRef<str>, T: 'static>(&mut self, key: S) -> Result<&mut T, String>
    where
        S: std::fmt::Display,
        T: Serializeable
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

    /// Set a value at a specified key.
    ///
    /// The value is cloned into a `Box` in order to add it to the frame.
    ///
    /// # Arguments
    /// * `key` - the key to set
    /// * `value` - the value to set (must be `Serializable`)
    pub fn set<S: Into<String>, T: 'static>(&mut self, key: S, value: T) -> ()
    where
        T: Serializeable + Clone
    {
        self.data.insert(key.into(), Box::new(value.clone()));
    }

    /// Write the frame to a stream.
    ///
    /// Serialize the frame into a stream using
    /// [`bincode`](https://crates.io/crates/bincode).
    ///
    /// # Arguments
    /// * `destination` - the stream to write to
    pub fn write_to_stream<W>(&self, destination: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write
    {
        match serialize_into(destination, &self.data) {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
        }
    }

    /// Read a frame from the stream.
    ///
    /// Deserialize a frame from a stream using
    /// [`bincode`](https://crates.io/crates/bincode).
    ///
    /// # Arguments
    /// * `source` - the stream to read from
    ///
    /// # Errors
    /// * A serialization error if the frame cannot be deserialized for
    ///   any reason.
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
