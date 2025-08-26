use raylib::prelude::*;
use crate::maze::Maze;
use crate::player::Player;

pub struct WallSlice {
    pub screen_x: i32,
    pub top: i32,
    pub bottom: i32,
    pub tex_x: f32,
}

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
}

pub fn cast_ray(
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
) -> Intersect {
    let mut d = 0.0f32;
    loop {
        let cos = d * a.cos();
        let sin = d * a.sin();
        let x = (player.pos.x + cos) as usize;
        let y = (player.pos.y + sin) as usize;
        let i = (x / block_size).min(maze[0].len() - 1);
        let j = (y / block_size).min(maze.len() - 1);
        if maze[j][i] != ' ' {
            return Intersect {
                distance: d.max(0.0001),
                impact: maze[j][i],
            };
        }
        d += 2.0;
    }
}

/// Calcula todas las columnas de pared visibles
pub fn compute_wall_slices(
    maze: &Maze,
    player: &Player,
    block_size: usize,
    screen_w: i32,
    screen_h: i32,
) -> Vec<WallSlice> {
    let mut slices = Vec::new();
    let hh = screen_h as f32 / 2.0;

    for i in 0..screen_w {
        let t = i as f32 / screen_w as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * t);
        let intersect = cast_ray(maze, player, a, block_size);

        let corrected = intersect.distance * (player.a - a).cos().abs().max(0.0001);
        let dpp = 150.0;
        let h = (hh / corrected) * dpp;
        let top = (hh - h / 2.0).max(0.0) as i32;
        let bottom = (hh + h / 2.0).min(screen_h as f32 - 1.0) as i32;

        let hit_x = player.pos.x + corrected * a.cos();
        let hit_y = player.pos.y + corrected * a.sin();

        let tex_x = if (hit_x as i32 % block_size as i32) < (hit_y as i32 % block_size as i32) {
            (hit_x as i32 % block_size as i32) as f32
        } else {
            (hit_y as i32 % block_size as i32) as f32
        };
        let tex_x = tex_x / block_size as f32;

        slices.push(WallSlice {
            screen_x: i,
            top,
            bottom,
            tex_x,
        });
    }
    slices
}
