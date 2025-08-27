use raylib::prelude::*;
use raylib::consts::{GamepadAxis, GamepadButton};
use crate::maze::Maze;

pub struct Player {
    pub pos: Vector2,
    pub a: f32,        // ángulo (radianes)
    walk_speed: f32,
}

impl Player {
    pub fn new(pos: Vector2) -> Self {
        Self {
            pos,
            a: 0.0,
            walk_speed: 170.0,
        }
    }

    pub fn update(&mut self, d: &mut RaylibDrawHandle, maze: &Maze, dt: f32) {
        // ---- rotación con mouse ----
        let md = d.get_mouse_delta();
        self.a += md.x * 0.0032; // sensibilidad mouse

        // ---- rotación con gamepad (stick derecho + D-Pad) ----
        if d.is_gamepad_available(0) {
            // derecha/izquierda del stick derecho
            let rx = deadzone(d.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_RIGHT_X), 0.18);
            // velocidad de giro por segundo (rad/s) escalada por el eje
            let turn_speed_gamepad = 3.6;
            self.a += rx * turn_speed_gamepad * dt;

            // giro fino con D-Pad
            let turn_speed_dpad = 2.6;
            if d.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT) {
                self.a -= turn_speed_dpad * dt;
            }
            if d.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT) {
                self.a += turn_speed_dpad * dt;
            }
        }

        // ---- rotación con teclado (← / →) ----
        let turn_speed_keys: f32 = 2.8; // rad/s
        if d.is_key_down(KeyboardKey::KEY_LEFT)  { self.a -= turn_speed_keys * dt; }
        if d.is_key_down(KeyboardKey::KEY_RIGHT) { self.a += turn_speed_keys * dt; }

        // mantener ángulo acotado
        if self.a > std::f32::consts::PI { self.a -= 2.0 * std::f32::consts::PI; }
        if self.a < -std::f32::consts::PI { self.a += 2.0 * std::f32::consts::PI; }

        // ---- movimiento (teclado + gamepad) ----
        // dir.x = forward/back ; dir.y = strafe
        let mut dir = Vector2::zero();

        // teclado
        if d.is_key_down(KeyboardKey::KEY_W) { dir.x += 1.0; }
        if d.is_key_down(KeyboardKey::KEY_S) { dir.x -= 1.0; }
        if d.is_key_down(KeyboardKey::KEY_D) { dir.y += 1.0; }
        if d.is_key_down(KeyboardKey::KEY_A) { dir.y -= 1.0; }

        // gamepad stick izquierdo
        if d.is_gamepad_available(0) {
            let lx = deadzone(d.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_X), 0.18);
            let ly = deadzone(d.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_Y), 0.18);
            // En la mayoría de mandos, arriba = -1, abajo = +1
            let forward = -ly;
            let strafe  =  lx;
            dir.x += forward;
            dir.y += strafe;
        }

        // velocidad base + sprint con L1 (opcional)
        let mut speed = self.walk_speed;
        if d.is_gamepad_available(0) && d.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_1) {
            speed *= 1.35;
        }
        if d.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
            speed *= 1.35;
        }

        // transformar a espacio mundo según el ángulo
        let mut mv = Vector2::zero();
        if dir.x != 0.0 {
            mv.x += self.a.cos() * dir.x;
            mv.y += self.a.sin() * dir.x;
        }
        if dir.y != 0.0 {
            // strafe (a + 90°)
            mv.x += (self.a + std::f32::consts::FRAC_PI_2).cos() * dir.y;
            mv.y += (self.a + std::f32::consts::FRAC_PI_2).sin() * dir.y;
        }

        if mv.length() > 0.0 {
            mv = norm(mv) * (speed * dt);
            self.try_move(maze, mv);
        }
    }

    fn try_move(&mut self, maze: &Maze, delta: Vector2) {
        let bs = maze.block_size() as f32;
        let r = 12.0; // radio de colisión
        let nx = self.pos.x + delta.x;
        let ny = self.pos.y + delta.y;

        // Colisiones separadas por eje (deslizamiento)
        if !collides(maze, nx, self.pos.y, r, bs) {
            self.pos.x = nx;
        }
        if !collides(maze, self.pos.x, ny, r, bs) {
            self.pos.y = ny;
        }
    }
}

fn collides(maze: &Maze, x: f32, y: f32, r: f32, bs: f32) -> bool {
    let tests = [
        (x - r, y - r),
        (x + r, y - r),
        (x - r, y + r),
        (x + r, y + r),
    ];
    for (tx, ty) in tests {
        let i = (tx/bs) as isize;
        let j = (ty/bs) as isize;
        if maze.is_blocking_at(i, j) { return true; }
    }
    false
}

// Normaliza un Vector2 sin depender de métodos de versión
fn norm(v: Vector2) -> Vector2 {
    let len = v.length();
    if len > 0.0 {
        Vector2::new(v.x/len, v.y/len)
    } else {
        v
    }
}

// Deadzone simétrica para sticks analógicos
fn deadzone(v: f32, dz: f32) -> f32 {
    if v.abs() < dz { 0.0 } else { v }
}
