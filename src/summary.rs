use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Center {
    pub lat: f32,
    pub lon: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub title: String,
    pub description: String,
    pub bounds: Bounds,
    pub center: Center,
}
