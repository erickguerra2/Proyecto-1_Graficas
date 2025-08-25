// src/maze.rs
use std::fs::File;
use std::io::{BufRead, BufReader};

pub type Maze = Vec<Vec<char>>;

pub fn load_maze(filename: &str) -> Maze {
    let file = File::open(filename).expect("No se pudo abrir el nivel");
    let reader = BufReader::new(file);
    reader.lines().map(|l| l.unwrap().chars().collect()).collect()
}
