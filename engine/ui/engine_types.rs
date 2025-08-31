use core::ffi::c_int;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Point {
    pub x: c_int,
    pub y: c_int,
}

impl Point {
    pub const fn new(x: c_int, y: c_int) -> Self {
        Self { x, y }
    }

    pub const fn components(&self) -> (c_int, c_int) {
        (self.x, self.y)
    }
}

impl From<Size> for Point {
    fn from(size: Size) -> Self {
        Self::new(size.width, size.height)
    }
}

impl From<(c_int, c_int)> for Point {
    fn from((x, y): (c_int, c_int)) -> Self {
        Self::new(x, y)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Size {
    pub width: c_int,
    pub height: c_int,
}

impl Size {
    pub const fn new(width: c_int, height: c_int) -> Self {
        Self { width, height }
    }

    pub const fn components(&self) -> (c_int, c_int) {
        (self.width, self.height)
    }
}

impl From<Point> for Size {
    fn from(point: Point) -> Self {
        Self::new(point.x, point.y)
    }
}

impl From<(c_int, c_int)> for Size {
    fn from((w, h): (c_int, c_int)) -> Self {
        Self::new(w, h)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum ActiveMenu {
    Console,
    Game,
    Menu,
}

impl TryFrom<c_int> for ActiveMenu {
    type Error = ();

    fn try_from(value: c_int) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Console),
            1 => Ok(Self::Game),
            2 => Ok(Self::Menu),
            _ => Err(()),
        }
    }
}
