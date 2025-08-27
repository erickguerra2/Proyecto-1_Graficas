use raylib::prelude::*;

pub struct FrameBuffer;

impl FrameBuffer {
    pub fn new() -> Self { Self }
    pub fn clear(&mut self, _d: &mut RaylibDrawHandle, _color: Color) {}
    pub fn draw_vertical_line(_d: &mut RaylibDrawHandle, _x: i32, _y0: i32, _y1: i32, _color: Color) {}
}
