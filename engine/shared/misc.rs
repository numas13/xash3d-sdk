use core::{ffi::c_int, mem};

use xash3d_ffi::common::wrect_s;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(x: i32, y: i32) -> Point {
        Self { x, y }
    }

    pub const fn to_rect(self, size: Size) -> Rect {
        Rect::new(self.x, self.y, size.width, size.height)
    }
}

impl From<Rect> for Point {
    fn from(rect: Rect) -> Self {
        rect.into_point()
    }
}

impl From<Point> for (i32, i32) {
    fn from(point: Point) -> Self {
        (point.x, point.y)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(width: u32, height: u32) -> Size {
        Self { width, height }
    }

    pub const fn to_rect(self, point: Point) -> Rect {
        Rect::new(point.x, point.y, self.width, self.height)
    }
}

impl From<Rect> for Size {
    fn from(rect: Rect) -> Self {
        rect.into_size()
    }
}

impl From<Size> for (u32, u32) {
    fn from(size: Size) -> Self {
        (size.width, size.height)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const ZERO: Self = Self::new(0, 0, 0, 0);

    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn from_point_size(point: Point, size: Size) -> Self {
        Self {
            x: point.x,
            y: point.y,
            width: size.width,
            height: size.height,
        }
    }

    pub const fn left(&self) -> i32 {
        self.x
    }

    pub const fn top(&self) -> i32 {
        self.y
    }

    pub const fn right(&self) -> i32 {
        if self.width > i32::MAX as u32 {
            panic!("rect width must be less than i32::MAX");
        }
        self.x + self.width as i32
    }

    pub const fn bottom(&self) -> i32 {
        if self.height > i32::MAX as u32 {
            panic!("rect height must be less than i32::MAX");
        }
        self.y + self.height as i32
    }

    pub const fn into_point(self) -> Point {
        Point::new(self.x, self.y)
    }

    pub const fn into_size(self) -> Size {
        Size::new(self.width, self.height)
    }
}

impl From<wrect_s> for Rect {
    fn from(rect: wrect_s) -> Self {
        Self {
            x: rect.left,
            y: rect.top,
            width: rect.right.saturating_sub(rect.left).max(0) as u32,
            height: rect.bottom.saturating_sub(rect.top).max(0) as u32,
        }
    }
}

impl From<Rect> for wrect_s {
    fn from(rect: Rect) -> Self {
        Self {
            left: rect.left(),
            right: rect.right(),
            top: rect.top(),
            bottom: rect.bottom(),
        }
    }
}

impl From<Size> for Rect {
    fn from(size: Size) -> Self {
        Self::new(0, 0, size.width, size.height)
    }
}

impl From<(Point, Size)> for Rect {
    fn from((point, size): (Point, Size)) -> Self {
        Self::from_point_size(point, size)
    }
}

impl From<Rect> for (Point, Size) {
    fn from(rect: Rect) -> Self {
        (rect.into_point(), rect.into_size())
    }
}

impl From<(i32, i32, u32, u32)> for Rect {
    fn from((x, y, width, height): (i32, i32, u32, u32)) -> Self {
        Self::new(x, y, width, height)
    }
}

#[deprecated]
pub trait WRectExt {
    fn default() -> Self;

    fn width(&self) -> c_int;

    fn height(&self) -> c_int;

    fn size(&self) -> (c_int, c_int) {
        (self.width(), self.height())
    }
}

#[allow(deprecated)]
impl WRectExt for wrect_s {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }

    fn width(&self) -> c_int {
        self.right - self.left
    }

    fn height(&self) -> c_int {
        self.bottom - self.top
    }
}
