use raylib::prelude::*;
use crate::{maze::Maze, player::Player};

pub struct MiniMapCfg {
    pub tile_px: i32,
    pub margin: i32,
    pub scale: f32,
}

pub fn draw_minimap(
    d: &mut RaylibDrawHandle,
    maze: &Maze,
    player: &Player,
    _enemies: Option<&[()]>,
    cfg: MiniMapCfg,
) {
    let tile = cfg.tile_px;
    let ox = cfg.margin;
    let oy = cfg.margin;
    let w = (maze.width() as i32) * tile;
    let h = (maze.height() as i32) * tile;

    d.draw_rectangle(ox - 4, oy - 4, w + 8, h + 8, Color::new(0, 0, 0, 160));
    d.draw_rectangle_lines(ox - 4, oy - 4, w + 8, h + 8, Color::WHITE);

    for j in 0..maze.height() {
        for i in 0..maze.width() {
            let c = maze.cell_i32(i as i32, j as i32);
            let color = match c {
                '#' => Color::DARKGRAY,
                'D' => Color::GOLD,
                'P' => Color::DARKBLUE,
                'S' => Color::MAROON,
                _ => Color::BLACK,
            };
            d.draw_rectangle(ox + (i as i32) * tile, oy + (j as i32) * tile, tile, tile, color);
        }
    }

    // jugador
    let bs = maze.block_size() as f32;
    let px = ox as f32 + (player.pos.x / bs) * tile as f32;
    let py = oy as f32 + (player.pos.y / bs) * tile as f32;
    d.draw_circle(px as i32, py as i32, (tile as f32) * 0.35, Color::SKYBLUE);

    // FOV líneas
    let len = (tile as f32) * 1.5;
    let a0 = player.a - 0.25;
    let a1 = player.a + 0.25;
    let (x0, y0) = (px + len * a0.cos(), py + len * a0.sin());
    let (x1, y1) = (px + len * a1.cos(), py + len * a1.sin());
    d.draw_line(px as i32, py as i32, x0 as i32, y0 as i32, Color::RAYWHITE);
    d.draw_line(px as i32, py as i32, x1 as i32, y1 as i32, Color::RAYWHITE);
}
