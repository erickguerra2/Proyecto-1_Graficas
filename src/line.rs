use raylib::prelude::*;

pub fn draw_line_fast(d: &mut RaylibDrawHandle, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    d.draw_line(x0, y0, x1, y1, color);
}
