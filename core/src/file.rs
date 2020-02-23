use crate::frame::Frame;

/// A convenience for reading and writing files of
/// [`Frames`](struct.Frame.html).
///
/// # Example
///
/// ```
/// use core::{Frame, File, FileMode};
///
/// // set up an Frame
/// let mut frame = Frame::new();
/// frame.set("foo", 123u8);
///
/// // get a tempfile
/// let dir = tempfile::tempdir().unwrap();
/// let path = dir.path().join("bar");
/// {
///   // open an File and write the frame
///   let mut file = File::new(path.to_str().unwrap(), FileMode::Write);
///   file.write_frame(&frame);
/// }
/// assert_eq!(path.is_file(), true);
/// {
///   // open the file and read the frame
///   let mut file = File::new(path.to_str().unwrap(), FileMode::Read);
///   match file.read_frame().unwrap() {
///     Some(frame2) => {
///       // compare to original frame
///       let val:&u8 = frame2.get("foo").unwrap();
///       assert_eq!(*val, 123u8);
///     },
///     None => panic!("no frame"),
///   };
/// }
/// ```
pub struct File {
    reader: Option<std::io::BufReader<std::fs::File>>,
    writer: Option<std::io::BufWriter<std::fs::File>>,
}

/// Different ways to open an [`File`](struct.File.html).
pub enum FileMode {
    Read,
    Write,
    Append,
}

impl File {
    /// Create a new File from a filename and mode.
    ///
    /// # Arguments
    /// * `filename` - name of file to open
    /// * `mode` - [`FileMode`](enum.FileMode.html) to open the file in.
    ///
    /// # Panics
    /// * if the file cannot be opened
    pub fn new<S: AsRef<str>>(filename: S, mode: FileMode) -> File
    where
        S: std::fmt::Display
    {
        let fname = filename.as_ref();
        let file = match mode {
            FileMode::Read => std::fs::OpenOptions::new().read(true).open(fname),
            FileMode::Write => std::fs::OpenOptions::new().create(true).truncate(true).write(true).open(fname),
            FileMode::Append => std::fs::OpenOptions::new().create(true).append(true).open(fname),
        };
        match file {
            Ok(f) => match mode {
                FileMode::Read => File{reader: Some(std::io::BufReader::new(f)), writer: None},
                _ => File{reader: None, writer: Some(std::io::BufWriter::new(f))},
            },
            Err(e) => panic!("cannot open file {}: {:?}", filename, e),
        }
    }

    /// Read a frame from the file.
    ///
    /// The correct way to end a file is an EOF at a frame boundary,
    /// which will return a `None` value. All other errors are propagated
    /// up.
    ///
    /// # Returns
    /// * Either an [`Frame`](struct.Frame.html) or `None`.
    ///
    /// # Errors
    /// * Any io errors that occur.
    ///
    /// # Panics
    /// * if we are trying to read a write-only file
    pub fn read_frame(&mut self) -> std::io::Result<Option<Frame>> {
        match &mut self.reader {
            Some(r) => {
                let mut frame = Frame::new();
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

    /// Write a frame to the file.
    ///
    /// # Arguments
    /// * `frame` - the [`Frame`](struct.Frame.html) to write
    ///
    /// # Errors
    /// * Any io errors that occur.
    ///
    /// # Panics
    /// * if we are trying to write a read-only file
    pub fn write_frame(&mut self, frame: &Frame) -> std::io::Result<()> {
        match &mut self.writer {
            Some(w) => frame.write_to_stream(w),
            None => panic!("trying to write to a read-only file"),
        }
    }
}
