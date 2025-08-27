use raylib::prelude::*;
use crate::maze::Maze;
use crate::player::Player;

struct Hit {
    perp_px: f32,       
    perp_cells: f32,    
    tex_x: f32,
    tile: char,
    side: i32,
    dir: Vector2,
}

pub fn render_3d_scene(
    d: &mut RaylibDrawHandle,
    maze: &Maze,
    player: &Player,
    wall_tex: &Texture2D,
    door_tex: &Texture2D,
) {
    let (sw, sh) = (d.get_screen_width(), d.get_screen_height());
    let (swf, shf) = (sw as f32, sh as f32);

    d.draw_rectangle(0, 0, sw, sh / 2, Color::new(15, 15, 25, 255));
    d.draw_rectangle(0, sh / 2, sw, sh / 2, Color::new(25, 20, 15, 255));

    let fov = std::f32::consts::FRAC_PI_3; // 60°
    for x in 0..sw {
        let cam_x = (2.0 * x as f32 / swf) - 1.0; // -1..+1
        let ray_angle = player.a + cam_x * (fov * 0.5);

        if let Some(mut hit) = cast_ray(maze, player.pos, ray_angle) {
            let bs = maze.block_size() as f32;
            let mut column_h = (shf * bs) / hit.perp_px.max(0.0001);
            column_h = column_h.min(shf * 4.0); // clamp para muy cerca

            let top = ((shf - column_h) * 0.5).max(0.0);
            let bottom = ((shf + column_h) * 0.5).min(shf);

            let tex = if hit.tile == 'D' { door_tex } else { wall_tex };
            let (tw, th) = (tex.width() as f32, tex.height() as f32);

            if (hit.side == 0 && hit.dir.x > 0.0) || (hit.side == 1 && hit.dir.y < 0.0) {
                hit.tex_x = 1.0 - hit.tex_x;
            }

            let src = Rectangle {
                x: (hit.tex_x * (tw - 1.0)).clamp(0.0, tw - 1.0),
                y: 0.0,
                width: 1.0,
                height: th,
            };
            let dst = Rectangle {
                x: x as f32,
                y: top,
                width: 1.0,
                height: bottom - top,
            };

            let k = 0.15;                 
            let min_brightness = 0.25;     
            let shade_base = 1.0 / (1.0 + k * hit.perp_cells);
            let shade = min_brightness + (1.0 - min_brightness) * shade_base;

            let tint = Color::new(
                (255.0 * shade) as u8,
                (255.0 * shade) as u8,
                (255.0 * shade) as u8,
                255,
            );

            d.draw_texture_pro(tex, src, dst, Vector2::zero(), 0.0, tint);
        }
    }
}

fn cast_ray(maze: &Maze, origin_px: Vector2, angle: f32) -> Option<Hit> {
    let bs = maze.block_size() as f32;

    // Posición/dirección en unidades de CELDA
    let pos_x = origin_px.x / bs;
    let pos_y = origin_px.y / bs;
    let dir_x = angle.cos();
    let dir_y = angle.sin();

    // Celda inicial
    let mut map_x = pos_x.floor() as i32;
    let mut map_y = pos_y.floor() as i32;

    // DeltaDist: cuánto recorre el rayo para cruzar una celda en X o Y
    let delta_x = if dir_x == 0.0 { f32::INFINITY } else { (1.0 / dir_x).abs() };
    let delta_y = if dir_y == 0.0 { f32::INFINITY } else { (1.0 / dir_y).abs() };

    // step y sideDist iniciales (en celdas)
    let (step_x, mut side_x) = if dir_x < 0.0 {
        (-1, (pos_x - map_x as f32) * delta_x)
    } else {
        ( 1, ((map_x as f32 + 1.0) - pos_x) * delta_x)
    };
    let (step_y, mut side_y) = if dir_y < 0.0 {
        (-1, (pos_y - map_y as f32) * delta_y)
    } else {
        ( 1, ((map_y as f32 + 1.0) - pos_y) * delta_y)
    };

    // DDA
    let mut side = 0; // 0 = vertical (eje X), 1 = horizontal (eje Y)
    let mut tile = ' ';
    for _ in 0..4096 {
        if side_x < side_y {
            side_x += delta_x;
            map_x += step_x;
            side = 0;
        } else {
            side_y += delta_y;
            map_y += step_y;
            side = 1;
        }
        tile = maze.cell(map_x as isize, map_y as isize);
        if tile == '#' || tile == 'D' {
            break;
        }
    }

    if tile != '#' && tile != 'D' {
        return None;
    }

    // Distancia perpendicular en CELDAS (no a lo largo del rayo)
    let dist_cells = if side == 0 {
        // pared vertical
        (map_x as f32 - pos_x + (1 - step_x) as f32 * 0.5) / dir_x
    } else {
        // pared horizontal
        (map_y as f32 - pos_y + (1 - step_y) as f32 * 0.5) / dir_y
    }.abs();

    // Coordenada de impacto a lo largo de la pared para tex_x
    let mut wall_x = if side == 0 {
        // vertical -> usar Y del impacto
        pos_y + dist_cells * dir_y
    } else {
        // horizontal -> usar X del impacto
        pos_x + dist_cells * dir_x
    };
    wall_x -= wall_x.floor(); // quedarnos con la fracción [0,1)

    Some(Hit {
        perp_px: (dist_cells * bs).max(0.0001),
        perp_cells: dist_cells,
        tex_x: wall_x,
        tile,
        side,
        dir: Vector2::new(dir_x, dir_y),
    })
}
