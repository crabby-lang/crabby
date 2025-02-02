pub struct Light {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}

impl Light {
    pub fn new(position: [f32; 3], color: [f32; 3], intensity: f32) -> Self {
        Self { position, color, intensity }
    }
}
