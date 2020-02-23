use crate::frame::Frame;

/// Module function type.
///
/// Any function that takes a mutable Frame
/// and returns a Frame.
pub type FunctionModule = fn(_: Frame) -> Frame;

/// Module trait.
///
/// A trait with a `process` function that takes a Frame
/// and returns a Frame.
pub trait Module {
    fn process(&self, _: Frame) -> Frame;
}

/// A simple module that converts from a function to
/// the `Module` trait.
pub struct SimpleModule {
    func: FunctionModule,
}

impl SimpleModule {
    pub fn new(f: FunctionModule) -> SimpleModule {
        SimpleModule{func: f}
    }
}

impl Module for SimpleModule
{
    fn process(&self, f: Frame) -> Frame {
        (self.func)(f)
    }
}

impl From<FunctionModule> for SimpleModule
{
    fn from(f: FunctionModule) -> Self {
        SimpleModule{func: f}
    }
}


/// Start module trait.
///
/// A trait with a `start` function that takes nothing
/// and returns a Frame or None.
pub trait StartModule {
    fn start(&self) -> Option<Frame>;
}

/// Infinite source module.
///
/// A start module that takes no input, and produces empty frames.
pub struct InfiniteSource {}

impl InfiniteSource {
    pub fn new() -> InfiniteSource {
        InfiniteSource{}
    }
}
impl StartModule for InfiniteSource {
    fn start(&self) -> Option<Frame> {
        Some(Frame::new())
    }
}
