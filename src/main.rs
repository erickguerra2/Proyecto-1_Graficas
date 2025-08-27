mod maze;
mod levels;
mod player;
mod caster;
mod minimap;
mod framebuffer;
mod line;

use raylib::prelude::*;
use raylib::ffi;
use std::ffi::CString;

use crate::levels::Levels;
use crate::player::Player;
use crate::caster::render_3d_scene;
use crate::minimap::{draw_minimap, MiniMapCfg};
use crate::maze::Maze;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppState {
    Menu,
    Playing,
    WonLevel,
}

struct ScreamerState {
    pos: Vector2,
    active: bool,
    timer: f32,
    enabled: bool, // solo se activa si el mapa trae 'S'
}

fn main() {
    // ---------- ventana ----------
    let (mut rl, thread) = raylib::init()
        .size(1024, 640)
        .title("Escape Reputation")
        .build();

    rl.set_target_fps(120);
    rl.set_exit_key(Some(KeyboardKey::KEY_ESCAPE));
    rl.disable_cursor(); // lock de mouse desde el inicio

    // ---------- niveles ----------
    let mut levels = Levels::load_from_dir("levels")
        .expect("No se pudieron cargar niveles desde ./levels. Crea la carpeta y pon mapas .txt");
    assert!(levels.len() > 0, "No hay niveles en ./levels");

    // Maze actual y punto de inicio (se asignan al arrancar nivel)
    let mut player = Player::new(Vector2::new(64.0 * 1.5, 64.0 * 1.5));

    // Screamer inicial
    let mut screamer = ScreamerState {
        pos: Vector2::zero(),
        active: false,
        timer: 0.0,
        enabled: false,
    };

    // ---------- assets visuales ----------
    let wall_tex = rl.load_texture(&thread, "assets/wall.png")
        .expect("Falta assets/wall.png");
    let door_tex = rl.load_texture(&thread, "assets/door.png")
        .expect("Falta assets/door.png");

    let menu_bg = load_bg_any(&mut rl, &thread, "menu_bg");
    let win_bg  = load_bg_any(&mut rl, &thread, "win_bg");

    let screamer_tex = rl.load_texture(&thread, "assets/screamer.png")
        .expect("Falta assets/screamer.png");

    unsafe { ffi::InitAudioDevice(); }

    // música en loop
    let music_path = CString::new("sounds/music.ogg").expect("ruta music.ogg inválida");
    let mut music: ffi::Music = unsafe { ffi::LoadMusicStream(music_path.as_ptr()) };
    unsafe { ffi::SetMusicVolume(music, 0.60); }

    // efectos
    let step_path = CString::new("sounds/step.wav").expect("ruta step.wav inválida");
    let step_snd: ffi::Sound = unsafe { ffi::LoadSound(step_path.as_ptr()) };
    unsafe { ffi::SetSoundVolume(step_snd, 0.75); }

    let screamer_path = CString::new("sounds/screamer.wav").expect("ruta screamer.wav inválida");
    let screamer_snd: ffi::Sound = unsafe { ffi::LoadSound(screamer_path.as_ptr()) };

    // pasos por distancia
    let mut step_accum: f32 = 0.0;  // píxeles acumulados caminados
    const STEP_PIXELS: f32 = 34.0;  // distancia entre pasos (~medio tile si tile=64)

    // ---------- estado de app ----------
    let mut state = AppState::Menu;
    let mut menu_sel: usize = 0;

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        let dt = d.get_frame_time();
        d.clear_background(Color::BLACK);

        match state {
            AppState::Menu => {
                // fondo
                let (sw, sh) = (d.get_screen_width() as f32, d.get_screen_height() as f32);
                d.draw_texture_pro(
                    &menu_bg,
                    Rectangle { x: 0.0, y: 0.0, width: menu_bg.width() as f32, height: menu_bg.height() as f32 },
                    Rectangle { x: 0.0, y: 0.0, width: sw, height: sh },
                    Vector2::zero(), 0.0, Color::WHITE
                );

                d.draw_rectangle(40, 40, sw as i32 - 80, sh as i32 - 80, Color::new(0, 0, 0, 120));
                d.draw_text("SELECCIONA UN NIVEL", 70, 60, 28, Color::RAYWHITE);
                d.draw_text("Usa flechitas y ENTER, o presiona 1..9", 70, 92, 18, Color::LIGHTGRAY);

                // números 1..9
                for n in 1..=levels.len().min(9) {
                    if let Some(key) = key_for_digit(n) {
                        if d.is_key_pressed(key) {
                            step_accum = 0.0;
                            start_level(&mut levels, n - 1, &mut player, &mut screamer, &mut music, &mut state);
                        }
                    }
                }
                // flechas + Enter
                if d.is_key_pressed(KeyboardKey::KEY_DOWN) { menu_sel = (menu_sel + 1) % levels.len(); }
                if d.is_key_pressed(KeyboardKey::KEY_UP)   { menu_sel = (menu_sel + levels.len() - 1) % levels.len(); }
                if d.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    step_accum = 0.0;
                    start_level(&mut levels, menu_sel, &mut player, &mut screamer, &mut music, &mut state);
                }

                // lista
                let base_y = 140;
                for i in 0..levels.len() {
                    let y = base_y + (i as i32) * 28;
                    let name = levels.name(i);
                    let line = format!("{} . {}", i + 1, name);
                    let color = if i == menu_sel { Color::YELLOW } else { Color::RAYWHITE };
                    d.draw_text(&line, 80, y, 22, color);
                }
            }

            AppState::Playing => {
                // actualizar música stream
                unsafe { ffi::UpdateMusicStream(music); }

                // ---- UPDATE ----
                let prev_pos = player.pos;
                player.update(&mut d, levels.active(), dt);

                // pasos por distancia recorrida
                let delta = (player.pos - prev_pos).length();
                if delta > 0.0 {
                    step_accum += delta;
                    if step_accum >= STEP_PIXELS {
                        // (opcional) variar ligeramente pitch
                        // unsafe { ffi::SetSoundPitch(step_snd, 0.98 + ((rl.get_time() as f32).sin().abs() * 0.04)); }
                        unsafe { ffi::PlaySound(step_snd); }
                        step_accum -= STEP_PIXELS;
                    }
                }

                // Puerta (victoria) delante + E
                if is_near_door_use(levels.active(), &player) && d.is_key_pressed(KeyboardKey::KEY_E) || (d.is_gamepad_available(0)
                    && d.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN)) {
                    unsafe {
                        ffi::StopMusicStream(music);
                        ffi::StopSound(step_snd);
                    }
                    step_accum = 0.0;
                    state = AppState::WonLevel;
                }

                // Screamer por proximidad + LOS
                update_screamer(&mut d, levels.active(), &player, &mut screamer, &screamer_snd);

                // ---- DRAW 3D ----
                render_3d_scene(&mut d, levels.active(), &player, &wall_tex, &door_tex);

                // ---- UI / Minimapa ----
                draw_minimap(
                    &mut d, levels.active(), &player, None,
                    MiniMapCfg { tile_px: 6, margin: 8, scale: 1.0 }
                );

                d.draw_text(
                    &format!("Nivel {}/{}", levels.index() + 1, levels.len()),
                    10, 10 + (levels.active().height() as i32 * 6) + 16,
                    16, Color::RAYWHITE
                );
                d.draw_text("E o X en el mando: usar puerta/  |   ESC: salir", 10, d.get_screen_height() - 22, 16, Color::RAYWHITE);

                // Screamer overlay
                if screamer.active {
                    let (sw, sh) = (d.get_screen_width(), d.get_screen_height());
                    d.draw_rectangle(0, 0, sw, sh, Color::new(220, 20, 60, 120));
                    let tw = screamer_tex.width() as f32;
                    let th = screamer_tex.height() as f32;
                    let dst = Rectangle {
                        x: (sw as f32 - tw) * 0.5,
                        y: (sh as f32 - th) * 0.5,
                        width: tw,
                        height: th,
                    };
                    d.draw_texture_pro(
                        &screamer_tex,
                        Rectangle { x: 0.0, y: 0.0, width: tw, height: th },
                        dst,
                        Vector2::zero(),
                        0.0,
                        Color::WHITE
                    );
                }
            }

            AppState::WonLevel => {
                // fondo de victoria
                let (sw, sh) = (d.get_screen_width() as f32, d.get_screen_height() as f32);
                d.draw_texture_pro(
                    &win_bg,
                    Rectangle { x: 0.0, y: 0.0, width: win_bg.width() as f32, height: win_bg.height() as f32 },
                    Rectangle { x: 0.0, y: 0.0, width: sw, height: sh },
                    Vector2::zero(), 0.0, Color::WHITE
                );

                unsafe { ffi::StopSound(step_snd); } // por si algo quedó colgado

                d.draw_rectangle(40, 40, sw as i32 - 80, sh as i32 - 80, Color::new(0, 0, 0, 140));
                d.draw_text("¡GANASTE!", 70, 60, 36, Color::LIME);
                d.draw_text("Elige otro nivel con 1-5 |   M: menú", 70, 104, 22, Color::RAYWHITE);

                // lista de niveles
                let base_y = 150;
                for i in 0..levels.len() {
                    let y = base_y + (i as i32) * 28;
                    let line = format!("{} . {}", i + 1, levels.name(i));
                    d.draw_text(&line, 80, y, 22, Color::YELLOW);
                }

                // números 1..9 para cargar
                for n in 1..=levels.len().min(9) {
                    if let Some(key) = key_for_digit(n) {
                        if d.is_key_pressed(key) {
                            step_accum = 0.0;
                            start_level(&mut levels, n - 1, &mut player, &mut screamer, &mut music, &mut state);
                        }
                    }
                }
                // volver al menú
                if d.is_key_pressed(KeyboardKey::KEY_M) {
                    state = AppState::Menu;
                    menu_sel = levels.index();
                }
            }
        }
        let sw = d.get_screen_width();
        d.draw_fps(sw - 100, 10);
        let dt_ms = d.get_frame_time() * 1000.0;
        d.draw_text(&format!("{:.1} ms", dt_ms), sw - 100, 30, 18, Color::LIGHTGRAY);

    }

    // ---------- cerrar audio ----------
    unsafe {
        ffi::UnloadSound(step_snd);
        ffi::UnloadSound(screamer_snd);
        ffi::UnloadMusicStream(music);
        ffi::CloseAudioDevice();
    }
}

// ---- helpers ----

fn load_bg_any(rl: &mut RaylibHandle, thread: &RaylibThread, base: &str) -> Texture2D {
    let candidates = [
        format!("assets/{}.png", base),
        format!("assets/{}.jpg", base),
        format!("assets/{}.jpeg", base),
        format!("assets/{}.bmp", base),
    ];
    for path in &candidates {
        if let Ok(tex) = rl.load_texture(thread, path) {
            return tex;
        }
    }
    rl.load_texture(thread, &format!("assets/{}.png", base))
        .expect(&format!("Falta assets/{}.png (o .jpg/.jpeg/.bmp)", base))
}

fn key_for_digit(n: usize) -> Option<KeyboardKey> {
    use KeyboardKey::*;
    Some(match n {
        1 => KEY_ONE, 2 => KEY_TWO, 3 => KEY_THREE,
        4 => KEY_FOUR, 5 => KEY_FIVE, 6 => KEY_SIX,
        7 => KEY_SEVEN, 8 => KEY_EIGHT, 9 => KEY_NINE,
        _ => return None,
    })
}

fn start_level(
    levels: &mut Levels,
    lvl_index: usize,
    player: &mut Player,
    screamer: &mut ScreamerState,
    music: &mut ffi::Music,
    state: &mut AppState,
) {
    levels.set_current(lvl_index);
    let maze = levels.active();

    *player = spawn_player_from_maze(maze);

    // screamer: solo si hay 'S'
    if let Some(p) = find_screamer_pos(maze) {
        screamer.pos = p;
        screamer.enabled = true;
    } else {
        screamer.enabled = false;
    }
    screamer.active = false;
    screamer.timer = 0.0;

    // arranca música del nivel
    unsafe { ffi::PlayMusicStream(*music); }

    *state = AppState::Playing;
}

fn spawn_player_from_maze(maze: &Maze) -> Player {
    if let Some(cell) = maze.find_char('P') {
        let p = maze.cell_center_world(cell);
        Player::new(p)
    } else {
        let bs = maze.block_size() as f32;
        Player::new(Vector2::new(bs * 1.5, bs * 1.5))
    }
}

fn find_screamer_pos(maze: &Maze) -> Option<Vector2> {
    maze.find_char('S').map(|c| maze.cell_center_world(c))
}

fn is_near_door_use(maze: &Maze, player: &Player) -> bool {
    let bs = maze.block_size() as f32;
    let front = Vector2 {
        x: player.pos.x + player.a.cos() * bs * 0.5,
        y: player.pos.y + player.a.sin() * bs * 0.5,
    };
    let (i, j) = ((front.x / bs) as isize, (front.y / bs) as isize);
    maze.is_door_at(i, j)
}

fn update_screamer(
    d: &mut RaylibDrawHandle,
    maze: &Maze,
    player: &Player,
    screamer: &mut ScreamerState,
    snd: &ffi::Sound,
) {
    if !screamer.enabled {
        return; // deshabilitado si el mapa no tiene 'S'
    }

    if screamer.active {
        screamer.timer -= d.get_frame_time();
        if screamer.timer <= 0.0 {
            screamer.active = false;
            screamer.timer = 0.0;
        }
        return;
    }

    // proximidad + LOS
    let trigger_dist = maze.block_size() as f32 * 1.2;
    let dist = (player.pos - screamer.pos).length();
    if dist <= trigger_dist && has_los(maze, player.pos, screamer.pos) {
        screamer.active = true;
        screamer.timer = 1.2;
        unsafe { ffi::PlaySound(*snd); }
    }
}

// Línea de vista simple
fn has_los(maze: &Maze, a: Vector2, b: Vector2) -> bool {
    let bs = maze.block_size() as f32;
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let steps = (dx.abs().max(dy.abs()) / (bs * 0.5)).max(1.0);
    let sx = dx / steps;
    let sy = dy / steps;

    let mut x = a.x;
    let mut y = a.y;
    for _ in 0..steps as i32 {
        let i = (x / bs) as isize;
        let j = (y / bs) as isize;
        if maze.is_blocking_at(i, j) { return false; }
        x += sx; y += sy;
    }
    true
}
