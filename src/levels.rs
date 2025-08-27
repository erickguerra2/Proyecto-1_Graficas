use std::fs;
use std::path::{Path, PathBuf};

use crate::maze::Maze;

pub struct Levels {
    maps: Vec<Maze>,
    names: Vec<String>,
    current: usize,
}

impl Levels {
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> std::io::Result<Self> {
        let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_file())
            .filter(|p| p.extension().map(|e| e.eq_ignore_ascii_case("txt")).unwrap_or(false))
            .collect();

        // 01.txt, 02.txt, ...
        entries.sort();

        let mut maps = Vec::new();
        let mut names = Vec::new();

        for path in entries {
            let text = fs::read_to_string(&path)?;
            let maze = Maze::from_str_map(&text, 64); // tamaño de celda en pixeles
            maps.push(maze);

            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("nivel")
                .to_string();
            names.push(name);
        }

        Ok(Self { maps, names, current: 0 })
    }

    /// Nivel activo (solo lectura)
    pub fn active(&self) -> &Maze {
        &self.maps[self.current]
    }

    /// Índice del nivel activo
    pub fn index(&self) -> usize { self.current }

    /// Cantidad de niveles
    pub fn len(&self) -> usize { self.maps.len() }

    /// Avanza al siguiente nivel. Retorna true si avanzó; false si ya no hay más.
    pub fn next(&mut self) -> bool {
        if self.current + 1 < self.maps.len() {
            self.current += 1;
            true
        } else {
            false
        }
    }

    /// Fija el nivel actual por índice (se acota al rango válido)
    pub fn set_current(&mut self, idx: usize) {
        if self.maps.is_empty() {
            self.current = 0;
        } else {
            self.current = idx.min(self.maps.len() - 1);
        }
    }

    /// Nombre legible del nivel `idx` (tomado del nombre del archivo sin extensión)
    pub fn name(&self, idx: usize) -> &str {
        self.names.get(idx).map(|s| s.as_str()).unwrap_or("nivel")
    }
}
