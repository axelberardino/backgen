pub mod cfg;
pub mod chooser;
pub mod color;
pub mod deserializer;
pub mod frame;
pub mod log;
pub mod paint;
pub mod pos;
pub mod salt;
pub mod scene;
pub mod shape;
pub mod svg;
pub mod tesselate;

pub mod prelude {
    use super::*;
    pub use cfg::{Pattern, Tiling};
    pub use chooser::Chooser;
    pub use color::Color;
    pub use frame::Frame;
    pub use pos::{radians, Pos};
    pub use salt::{Salt, SaltItem};

    use std::collections::HashMap;
    pub type ColorList = HashMap<String, Color>;
    pub type ThemeList = HashMap<String, Chooser<ThemeItem>>;

    #[derive(Clone, Debug)]
    pub struct ThemeItem(pub Color, pub Option<usize>, pub Option<usize>, pub Salt);
}
