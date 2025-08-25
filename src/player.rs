// src/player.rs
use raylib::prelude::*;
use std::f32::consts::PI;
use crate::maze::Maze;

pub struct Player {
    pub pos: Vector2,
    pub a: f32,
    pub fov: f32,
}

pub fn process_events(
    player: &mut Player,
    rl: &RaylibHandle,
    maze: &Maze,
    block_size: usize,
    mouse_locked: bool,
) -> bool {
    const MOVE_SPEED: f32 = 4.0;
    const ROT_SPEED: f32 = PI / 100.0;
    let mut moved = false;

    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        player.a += ROT_SPEED;
    }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.a -= ROT_SPEED;
    }
    if mouse_locked {
        let md = rl.get_mouse_delta();
        player.a -= md.x * 0.003;
    }

    let mut dx = 0.0;
    let mut dy = 0.0;
    if rl.is_key_down(KeyboardKey::KEY_W) {
        dx += MOVE_SPEED * player.a.cos();
        dy += MOVE_SPEED * player.a.sin();
    }
    if rl.is_key_down(KeyboardKey::KEY_S) {
        dx -= MOVE_SPEED * player.a.cos();
        dy -= MOVE_SPEED * player.a.sin();
    }
    if rl.is_key_down(KeyboardKey::KEY_A) {
        dx += MOVE_SPEED * (player.a - PI / 2.0).cos();
        dy += MOVE_SPEED * (player.a - PI / 2.0).sin();
    }
    if rl.is_key_down(KeyboardKey::KEY_D) {
        dx += MOVE_SPEED * (player.a + PI / 2.0).cos();
        dy += MOVE_SPEED * (player.a + PI / 2.0).sin();
    }

    let try_x = player.pos.x + dx;
    let try_y = player.pos.y + dy;
    let i = (try_x as usize / block_size).min(maze[0].len() - 1);
    let j = (try_y as usize / block_size).min(maze.len() - 1);

    if maze[j][i] == ' ' || maze[j][i] == 'g' {
        if dx.abs() > 0.001 || dy.abs() > 0.001 {
            moved = true;
        }
        player.pos.x = try_x;
        player.pos.y = try_y;
    }

    moved
}
