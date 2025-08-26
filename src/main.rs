#![allow(dead_code)]

mod framebuffer;
mod maze;
mod caster;
mod player;
mod line;

use raylib::prelude::*;
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
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_W, WINDOW_H)
        .title("Ray Caster — Entrega")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut state = GameState::Menu;
    let level_files = vec!["levels/maze1.txt", "levels/maze2.txt"];
    let mut current_level = 0usize;

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
    let wall_tex = rl.load_texture(&thread, "assets/wall.png").unwrap();
    let menu_bg = rl.load_texture(&thread, "assets/menu_bg.png").unwrap();
    let win_bg = rl.load_texture(&thread, "assets/win_bg.png").unwrap();
    let screamer_tex = rl.load_texture(&thread, "assets/screamer.png").unwrap();

    while !rl.window_should_close() {
        let now = Instant::now();
        let _dt = now.duration_since(last_frame);
        last_frame = now;

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
                let key_enter = rl.is_key_pressed(KeyboardKey::KEY_ENTER);
                let key_l = rl.is_key_pressed(KeyboardKey::KEY_L);
                let key_escape = rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);

                let mut d = rl.begin_drawing(&thread);
                d.clear_background(Color::BLACK);
                d.draw_texture(&menu_bg, 0, 0, Color::WHITE);
                d.draw_text("RAY CASTER", 100, 100, 80, Color::YELLOW);
                d.draw_text("ENTER: Jugar", 100, 300, 30, Color::WHITE);
                d.draw_text("L: Seleccionar nivel", 100, 350, 30, Color::WHITE);
                d.draw_text("ESC: Salir", 100, 400, 30, Color::WHITE);

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
                let key_one = rl.is_key_pressed(KeyboardKey::KEY_ONE);
                let key_two = rl.is_key_pressed(KeyboardKey::KEY_TWO);
                let key_enter = rl.is_key_pressed(KeyboardKey::KEY_ENTER);
                let key_escape = rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);

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
                let _moved = process_events(&mut player, &rl, &maze, BLOCK_SIZE, mouse_locked);

                // calcular slices de pared
                let slices = crate::caster::compute_wall_slices(
                    &maze,
                    &player,
                    BLOCK_SIZE,
                    WINDOW_W,
                    WINDOW_H,
                );

                let fps = rl.get_fps();
                let mut d = rl.begin_drawing(&thread);

                // --- techo y suelo en color sólido
                d.draw_rectangle(0, 0, WINDOW_W, WINDOW_H/2, Color::BLACK);       // techo
                d.draw_rectangle(0, WINDOW_H/2, WINDOW_W, WINDOW_H/2, Color::DARKBROWN); // suelo


                // --- paredes con textura
                for slice in slices {
                    let src = Rectangle {
                        x: slice.tex_x * wall_tex.width() as f32,
                        y: 0.0,
                        width: 1.0,
                        height: wall_tex.height() as f32,
                    };
                    let dst = Rectangle {
                        x: slice.screen_x as f32,
                        y: slice.top as f32,
                        width: 1.0,
                        height: (slice.bottom - slice.top) as f32,
                    };
                    d.draw_texture_pro(
                        &wall_tex,
                        src,
                        dst,
                        Vector2::zero(),
                        0.0,
                        Color::WHITE,
                    );
                }

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

                // victoria
                if maze[j][i] == 'g' {
                    state = GameState::Win;
                }
            }
            GameState::Win => {
                let key_enter = rl.is_key_pressed(KeyboardKey::KEY_ENTER);
                let key_l = rl.is_key_pressed(KeyboardKey::KEY_L);
                let key_escape = rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);

                let mut d = rl.begin_drawing(&thread);
                d.clear_background(Color::DARKGREEN);
                d.draw_texture_pro(
                    &win_bg,
                    Rectangle { x: 0.0, y: 0.0, width: win_bg.width() as f32, height: win_bg.height() as f32 },
                    Rectangle { x: 0.0, y: 0.0, width: WINDOW_W as f32, height: WINDOW_H as f32 },
                    Vector2::zero(),
                    0.0,
                    Color::WHITE,
                );
                d.draw_text("¡Nivel completado!", 80, 100, 70, Color::WHITE);
                d.draw_text("ENTER: Rejugar | L: Niveles | ESC: Menú", 80, 300, 30, Color::WHITE);

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
