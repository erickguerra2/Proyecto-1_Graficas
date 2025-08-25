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

    // --- assets
    let floor_tex = rl.load_texture(&thread, "assets/floor.png").unwrap();
    let ceil_tex = rl.load_texture(&thread, "assets/ceiling.png").unwrap();
    let menu_bg = rl.load_texture(&thread, "assets/menu_bg.png").unwrap();
    let win_bg = rl.load_texture(&thread, "assets/win_bg.png").unwrap();
    let screamer_tex = rl.load_texture(&thread, "assets/screamer.png").unwrap();

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
                // --- inputs
                let key_enter = rl.is_key_pressed(KeyboardKey::KEY_ENTER);
                let key_l = rl.is_key_pressed(KeyboardKey::KEY_L);
                let key_escape = rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);

                // --- dibujo
                let mut d = rl.begin_drawing(&thread);
                d.clear_background(Color::BLACK);
                d.draw_texture(&menu_bg, 0, 0, Color::WHITE);
                d.draw_text("RAY CASTER", 100, 100, 80, Color::YELLOW);
                d.draw_text("ENTER: Jugar", 100, 300, 30, Color::WHITE);
                d.draw_text("L: Seleccionar nivel", 100, 350, 30, Color::WHITE);
                d.draw_text("ESC: Salir", 100, 400, 30, Color::WHITE);
                drop(d);

                // --- lógica
                if key_enter {
                    state = GameState::Playing;
                }
                if key_l {
                    state = GameState::LevelSelect;
                }
                if key_escape {
                    break;
                }
            }
            GameState::LevelSelect => {
                // --- inputs
                let key_one = rl.is_key_pressed(KeyboardKey::KEY_ONE);
                let key_two = rl.is_key_pressed(KeyboardKey::KEY_TWO);
                let key_enter = rl.is_key_pressed(KeyboardKey::KEY_ENTER);
                let key_escape = rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);

                // --- dibujo
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

                // --- lógica
                if key_one {
                    current_level = 0;
                    maze = load_maze(level_files[current_level]);
                }
                if key_two {
                    current_level = 1;
                    maze = load_maze(level_files[current_level]);
                }
                if key_enter {
                    state = GameState::Playing;
                }
                if key_escape {
                    state = GameState::Menu;
                }
            }
            GameState::Playing => {
                framebuffer.clear();
                let _moved = process_events(&mut player, &rl, &maze, BLOCK_SIZE, mouse_locked);

                // render walls
                crate::caster::render_world(
                    &mut framebuffer,
                    &maze,
                    &player,
                    BLOCK_SIZE,
                );

                // --- inputs
                let fps = rl.get_fps();

                // --- dibujo
                if let Ok(texture) = rl.load_texture_from_image(&thread, &framebuffer.color_buffer) {
                    let mut d = rl.begin_drawing(&thread);

                    // --- techo y suelo
                    d.draw_texture(&ceil_tex, 0, 0, Color::WHITE);
                    d.draw_texture(&floor_tex, 0, WINDOW_H/2, Color::WHITE);

                    // --- laberinto (raycasting framebuffer)
                    d.draw_texture(&texture, 0, 0, Color::WHITE);

                    // --- screamer (si está en celda 's')
                    let i = (player.pos.x as usize / BLOCK_SIZE).min(maze[0].len() - 1);
                    let j = (player.pos.y as usize / BLOCK_SIZE).min(maze.len() - 1);
                    if maze[j][i] == 's' {
                        d.draw_texture_ex(
                            &screamer_tex,
                            Vector2::new((WINDOW_W/2 - 200) as f32, (WINDOW_H/2 - 200) as f32),
                            0.0,
                            2.0,
                            Color::WHITE,
                        );
                    }

                    // --- HUD
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
                // --- inputs
                let key_enter = rl.is_key_pressed(KeyboardKey::KEY_ENTER);
                let key_l = rl.is_key_pressed(KeyboardKey::KEY_L);
                let key_escape = rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);

                // --- dibujo
                let mut d = rl.begin_drawing(&thread);
                d.clear_background(Color::DARKGREEN);
                d.draw_texture(&win_bg, 0, 0, Color::WHITE);
                d.draw_text("¡Nivel completado!", 80, 100, 70, Color::WHITE);
                d.draw_text("ENTER: Rejugar | L: Niveles | ESC: Menú", 80, 300, 30, Color::WHITE);
                drop(d);

                // --- lógica
                if key_enter {
                    state = GameState::Playing;
                }
                if key_l {
                    state = GameState::LevelSelect;
                }
                if key_escape {
                    state = GameState::Menu;
                }
            }
        }

        std::thread::sleep(target_frame);
    }
}
