use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

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
        d += 5.0;
    }
}

pub fn render_world(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player, block_size: usize) {
    let num_rays = framebuffer.width;
    let hh = framebuffer.height as f32 / 2.0;

    for i in 0..num_rays {
        let t = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * t);
        let intersect = cast_ray(maze, player, a, block_size);

        let corrected = intersect.distance * (player.a - a).cos().abs().max(0.0001);
        let dpp = 70.0;
        let h = (hh / corrected) * dpp;
        let top = (hh - h / 2.0).max(0.0) as i32;
        let bottom = (hh + h / 2.0).min(framebuffer.height as f32 - 1.0) as i32;

        let col = match intersect.impact {
            '1' => Color::MAROON,
            '2' => Color::ORANGE,
            '3' => Color::DARKBLUE,
            '+' => Color::BLUEVIOLET,
            '-' => Color::VIOLET,
            '|' => Color::DARKPURPLE,
            'g' => Color::GREEN,
            _ => Color::WHITE,
        };
        framebuffer.set_current_color(col);
        for y in top..=bottom {
            framebuffer.set_pixel(i, y as u32);
        }
    }
}

pub fn render_minimap(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player, block_size: usize) {
    let mini_block = 10usize;
    let offset_x = 10usize;
    let offset_y = 10usize;

    for (j, row) in maze.iter().enumerate() {
        for (i, &cell) in row.iter().enumerate() {
            if cell != ' ' {
                framebuffer.set_current_color(Color::GRAY);
                for x in 0..mini_block {
                    for y in 0..mini_block {
                        framebuffer.set_pixel(
                            (offset_x + i * mini_block + x) as u32,
                            (offset_y + j * mini_block + y) as u32,
                        );
                    }
                }
            }
        }
    }

    // jugador
    framebuffer.set_current_color(Color::YELLOW);
    let px = (player.pos.x as usize / block_size) * mini_block + offset_x + mini_block / 2;
    let py = (player.pos.y as usize / block_size) * mini_block + offset_y + mini_block / 2;
    for x in px.saturating_sub(2)..=px + 2 {
        for y in py.saturating_sub(2)..=py + 2 {
            framebuffer.set_pixel(x as u32, y as u32);
        }
    }
}
