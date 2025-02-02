use crate::graphics::Graphics;

pub struct Renderer {
    pub graphics: Graphics,
}

impl Renderer {
    pub fn new(graphics: Graphics) -> Self {
        Self { graphics }
    }

    pub fn draw(&self) {
        println!("Drawing Frame...");
    }
}
