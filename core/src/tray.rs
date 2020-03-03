use crate::module::*;

/// Tray of modules.
///
/// This is the primary way to configure module connections
///
/// # Examples
///
/// Run an empty tray 10 times:
///
/// ```
/// #[tokio::main]
/// async fn main() {
///   use core::{Tray, InfiniteSource};
///   let tray = Tray::new(InfiniteSource::new());
///   tray.run_bounded(10).await;
/// }
/// ```
///
/// Add a function to the tray:
///
/// ```
/// use core::{Tray, InfiniteSource, Frame, SimpleModule};
///
/// fn print(f: Frame) -> Frame {
///   println!("got frame");
///   f
/// }
///
/// #[tokio::main]
/// async fn main() {
///   let mut tray = Tray::new(InfiniteSource::new());
///   tray.add_fn(print);
///   tray.run_bounded(10).await;
/// }
/// ```
///
/// Add a module to the tray:
///
/// ```
/// use core::{Tray, InfiniteSource, Frame, Module};
///
/// struct MyMod { }
/// impl Module for MyMod {
///   fn process(&self, f: Frame) -> Frame {
///     println!("got frame");
///     f
///   }
/// }
///
/// #[tokio::main]
/// async fn main() {
///   let mut tray = Tray::new(InfiniteSource::new());
///   tray.add(MyMod{});
///   tray.run_bounded(10).await;
/// }
/// ```
pub struct Tray {
    start_module: Box<dyn StartModule>,
    modules: Vec<Box<dyn Module>>,
}

impl Tray {
    /// Create a new empty Tray.
    pub fn new<S: 'static>(s: S) -> Tray
    where
        S: StartModule,
    {
        Tray{start_module: Box::new(s), modules: Vec::new()}
    }

    /// Add a module to the Tray.
    ///
    /// # Arguments
    /// * `m` - module to add
    pub fn add<M: 'static>(&mut self, m: M) -> ()
    where
        M: Module,
    {
        self.modules.push(Box::new(m));
    }

    /// Add a function module to the Tray.
    ///
    /// # Arguments
    /// * `m` - funciton module to add
    pub fn add_fn(&mut self, m: FunctionModule) -> ()
    {
        self.add(SimpleModule::new(m))
    }

    /// Run the tray until it ends.
    pub async fn run(&self) -> () {
        self.run_bounded(std::u64::MAX).await;
    }

    /// Run the tray for `num` frames, or until it ends on its own.
    ///
    /// # Arguments
    /// * `num` - number of frames to execute
    pub async fn run_bounded(&self, num:u64) -> () {
        for _ in 0..num {
            match self.start_module.start() {
                Some(mut fr) => {
                    for m in self.modules.iter() {
                        fr = m.process(fr);
                    }
                },
                None => {
                    break;
                }
            }
        }
    }
}
