#![allow(dead_code)]

mod framebuffer;
mod maze;
mod caster;
mod player;
mod line;

use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::maze::load_maze;
use crate::player::{process_events, Player};

use std::f32::consts::PI;
use std::time::{Duration, Instant};

const WINDOW_W: i32 = 1280;
const WINDOW_H: i32 = 800;
const BLOCK_SIZE: usize = 100;

#[derive(Clone, Copy, PartialEq, Eq)]
enum GameState {
    Menu,
    LevelSelect,
    Playing,
    Win,
}

fn main() {
    // --- ventana
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_W, WINDOW_H)
        .title("Ray Caster — Entrega")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    // --- estados
    let mut state = GameState::Menu;
    let level_files = vec!["levels/maze1.txt", "levels/maze2.txt"];
    let mut current_level = 0usize;

    let mut framebuffer = Framebuffer::new(WINDOW_W as u32, WINDOW_H as u32);
    framebuffer.set_background_color(Color::new(50, 50, 100, 255));

    let mut maze = load_maze(level_files[current_level]);
    let mut player = Player {
        pos: Vector2::new(150.0, 150.0),
        a: PI / 3.0,
        fov: PI / 3.0,
    };

    let mut last_frame = Instant::now();
    let target_frame = Duration::from_millis(16);

    let mut mouse_locked = false;
    rl.set_mouse_cursor(raylib::consts::MouseCursor::MOUSE_CURSOR_ARROW);

    while !rl.window_should_close() {
        let now = Instant::now();
        let _dt = now.duration_since(last_frame);
        last_frame = now;

        // toggle mouse lock
        if rl.is_key_pressed(KeyboardKey::KEY_M) {
            mouse_locked = !mouse_locked;
            if mouse_locked {
                rl.disable_cursor();
            } else {
                rl.enable_cursor();
            }
        }

        match state {
            GameState::Menu => {
                let mut d = rl.begin_drawing(&thread);
                d.clear_background(Color::DARKBLUE);
                d.draw_text("RAY CASTER", 40, 60, 60, Color::WHITE);
                d.draw_text("ENTER: Jugar", 40, 140, 30, Color::RAYWHITE);
                d.draw_text("L: Seleccionar nivel", 40, 180, 30, Color::RAYWHITE);
                d.draw_text("ESC: Salir", 40, 220, 30, Color::RAYWHITE);
                drop(d);

                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    state = GameState::Playing;
                }
                if rl.is_key_pressed(KeyboardKey::KEY_L) {
                    state = GameState::LevelSelect;
                }
            }
            GameState::LevelSelect => {
                let mut d = rl.begin_drawing(&thread);
                d.clear_background(Color::BLACK);
                d.draw_text("Selecciona nivel (1 o 2)", 40, 60, 40, Color::RAYWHITE);
                for (i, name) in level_files.iter().enumerate() {
                    d.draw_text(
                        &format!("{} - {}", i + 1, name),
                        60,
                        120 + (i as i32) * 30,
                        24,
                        if i == current_level { Color::YELLOW } else { Color::RAYWHITE },
                    );
                }
                drop(d);

                if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
                    current_level = 0;
                    maze = load_maze(level_files[current_level]);
                }
                if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
                    current_level = 1;
                    maze = load_maze(level_files[current_level]);
                }
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    state = GameState::Playing;
                }
                if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    state = GameState::Menu;
                }
            }
            GameState::Playing => {
                framebuffer.clear();
                let _moved = process_events(&mut player, &rl, &maze, BLOCK_SIZE, mouse_locked);

                // render walls
                crate::caster::render_world(&mut framebuffer, &maze, &player, BLOCK_SIZE);

                // minimapa
                crate::caster::render_minimap(&mut framebuffer, &maze, &player, BLOCK_SIZE);

                // HUD
                let fps = rl.get_fps();
                if let Ok(texture) = rl.load_texture_from_image(&thread, &framebuffer.color_buffer)
                {
                    let mut d = rl.begin_drawing(&thread);
                    d.clear_background(Color::BLACK);
                    d.draw_texture(&texture, 0, 0, Color::WHITE);
                    d.draw_text(&format!("FPS: {}", fps), 10, 10, 20, Color::RAYWHITE);
                }

                // victoria
                let i = (player.pos.x as usize / BLOCK_SIZE).min(maze[0].len() - 1);
                let j = (player.pos.y as usize / BLOCK_SIZE).min(maze.len() - 1);
                if maze[j][i] == 'g' {
                    state = GameState::Win;
                }
            }
            GameState::Win => {
                let mut d = rl.begin_drawing(&thread);
                d.clear_background(Color::DARKGREEN);
                d.draw_text("¡Nivel completado!", 40, 60, 60, Color::WHITE);
                d.draw_text("ENTER: Rejugar | L: Niveles | ESC: Menú", 40, 140, 24, Color::RAYWHITE);
                drop(d);

                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    state = GameState::Playing;
                }
                if rl.is_key_pressed(KeyboardKey::KEY_L) {
                    state = GameState::LevelSelect;
                }
                if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    state = GameState::Menu;
                }
            }
        }

        std::thread::sleep(target_frame);
    }
}
