use raylib::prelude::*;

#[derive(Clone)]
pub struct Maze {
    grid: Vec<Vec<char>>,
    block_size: u32,
    w: usize,
    h: usize,
}

impl Maze {
    pub fn new(grid: Vec<Vec<char>>, block_size: u32) -> Self {
        let h = grid.len();
        let w = if h > 0 { grid[0].len() } else { 0 };
        Self { grid, block_size, w, h }
    }

    /// Crea un Maze a partir del texto de un .txt (múltiples líneas)
    pub fn from_str_map(text: &str, block_size: u32) -> Self {
        let mut rows: Vec<Vec<char>> = Vec::new();
        for line in text.lines() {
            let row: Vec<char> = line.chars().collect();
            if !row.is_empty() {
                rows.push(row);
            }
        }
        // Normalizar anchura con padding de espacios
        let max_w = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        for r in &mut rows {
            if r.len() < max_w {
                r.extend(std::iter::repeat(' ').take(max_w - r.len()));
            }
        }
        Self::new(rows, block_size)
    }

    pub fn width(&self) -> usize { self.w }
    pub fn height(&self) -> usize { self.h }
    pub fn block_size(&self) -> u32 { self.block_size }

    pub fn cell(&self, i: isize, j: isize) -> char {
        if i < 0 || j < 0 { return '#'; }
        let (i, j) = (i as usize, j as usize);
        if j >= self.h || i >= self.w { return '#'; }
        self.grid[j][i]
    }

    pub fn cell_i32(&self, i: i32, j: i32) -> char {
        self.cell(i as isize, j as isize)
    }

    pub fn is_blocking_at(&self, i: isize, j: isize) -> bool {
        match self.cell(i, j) {
            '#' => true,
            'D' => true, // la puerta bloquea hasta que el jugador la “use”
            _ => false,
        }
    }

    pub fn is_door_at(&self, i: isize, j: isize) -> bool {
        self.cell(i, j) == 'D'
    }

    /// Busca la primera ocurrencia de un char y devuelve (i,j) en celdas
    pub fn find_char(&self, ch: char) -> Option<(isize, isize)> {
        for j in 0..self.h {
            for i in 0..self.w {
                if self.grid[j][i] == ch {
                    return Some((i as isize, j as isize));
                }
            }
        }
        None
    }

    /// Centro (x,y) del mundo (en píxeles) de una celda (i,j)
    pub fn cell_center_world(&self, cell: (isize, isize)) -> Vector2 {
        let bs = self.block_size as f32;
        Vector2::new(
            (cell.0 as f32 + 0.5) * bs,
            (cell.1 as f32 + 0.5) * bs
        )
    }

    pub fn world_to_cell(&self, p: Vector2) -> (isize, isize) {
        let bs = self.block_size as f32;
        ((p.x/bs) as isize, (p.y/bs) as isize)
    }
}
