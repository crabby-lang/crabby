
pub struct ComputeShader {
    pub shader_code: Vec<u8>,
}

impl ComputeShader {
    pub fn new(shader_code: Vec<u8>) -> Self {
        Self { shader_code }
    }
}
