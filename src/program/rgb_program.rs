use std::mem::size_of;

use glium::{program::ComputeShader, uniform, Display, Texture2d, uniforms::{UniformBuffer, ImageUnitAccess, ImageUnitFormat}, texture::{UncompressedFloatFormat, MipmapsOption}, Surface};
use yaml_rust::Yaml;

use super::{Program, EdgeSolution};

pub struct RgbProgram {
    width: u32,
    height: u32,
    convolution_shader: ComputeShader,
    kernel_buf: UniformBuffer<[f32]>,
    kernel_size: usize,
    buffer_texture: Texture2d
}

impl RgbProgram {
    pub fn new(
        width: u32,
        height: u32,
        fun: &str,
        kernel: Vec<Vec<f32>>,
        display: &Display,
        edge_solution: EdgeSolution<(f32, f32, f32)>
    ) -> Self {
        let clamp_src = edge_solution.csample_src();
        let kernel_size = kernel.len();
        let flat_kernel: Vec<f32> = kernel.iter()
            .flatten()
            .copied()
            .collect();

        
        let kernel_buf: UniformBuffer<[f32]> = UniformBuffer::empty_unsized_immutable(display, kernel_size*kernel_size*size_of::<f32>()).unwrap();
        kernel_buf.write(&flat_kernel);

        let buffer_texture = glium::texture::Texture2d::with_format(
            display,
            vec![vec![(0., 0., 0., 1.); width as usize]; height as usize],
            UncompressedFloatFormat::F32F32F32F32,
            MipmapsOption::NoMipmap,
        )
        .unwrap();

        Self {
            width,
            height,
            convolution_shader: glium::program::ComputeShader::from_source(
                display,
                &convolution_shader_src(fun, &clamp_src, kernel_size * kernel_size)
            )
            .unwrap(),
            kernel_buf,
            kernel_size,
            buffer_texture
        }
    }
}

impl Program for RgbProgram {
    fn step(&self, board: &mut Texture2d) {
        
        board.as_surface().fill(&self.buffer_texture.as_surface(), glium::uniforms::MagnifySamplerFilter::Nearest);

        let image_unit = board
            .image_unit(ImageUnitFormat::RGBA32F)
            .unwrap()
            .set_access(ImageUnitAccess::Write);
        let image_buffer = self.buffer_texture
            .image_unit(ImageUnitFormat::RGBA32F)
            .unwrap()
            .set_access(ImageUnitAccess::Read);
        
        self.convolution_shader.execute(
            uniform! { 
                uWidth: self.width, 
                uHeight: self.height, 
                uKernelSize: self.kernel_size as i32, 
                uKernel: &self.kernel_buf, 
                uTextureWrite: image_unit,
                uTexture: image_buffer
            }, 
            self.width/16 + if self.width % 16 > 0 {1} else {0}, 
            self.height/16 + if self.height % 16 > 0 {1} else {0}, 
            1
        );
    }

    fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn from_yaml(doc: &Yaml, display: &Display) -> Self {
        let edge = {
            if let Some(edge) = doc["edge"].as_str() {
                if edge == "wrap" {
                    EdgeSolution::Wrap
                } else if edge == "clamp" {
                    EdgeSolution::Clamp
                } else {
                    println!("Invalid edge value!");
                    EdgeSolution::Clamp
                }
            } else if let Some(val) = doc["edge"].as_vec() {
                let val: Vec<f32> = val.iter().filter_map(|x| x.as_f64()).map(|x| x as f32).collect();
                if val.len() == 3 {
                    EdgeSolution::Value((val[0], val[1], val[2]))
                } else {
                    println!("Value must be a tuple of 3 (r, g, b) values.");
                    EdgeSolution::Clamp
                }
            } else {
                println!("Invalid edge value!");
                EdgeSolution::Clamp
            }
        };
        Self::new(
            doc["screen"][0].as_i64().unwrap().try_into().unwrap(),
            doc["screen"][1].as_i64().unwrap().try_into().unwrap(),
            doc["fun"].as_str().unwrap(),
            doc["kernel"]
                .as_vec()
                .unwrap()
                .into_iter()
                .map(|s| {
                    s.as_vec()
                        .expect(&format!(
                            "Error reading program file: Kernel not an array {:?}",
                            s
                        ))
                        .into_iter()
                        .map(|yaml| {
                            yaml.as_f64().expect(&format!(
                                "Error reading program file: Kernel not a float ({:?})",
                                s
                            )) as f32
                        })
                        .collect::<Vec<f32>>()
                })
                .collect::<Vec<Vec<f32>>>(),
            display,
            edge
        )
    }
}

fn convolution_shader_src(fun_src: &str, csample_src: &str, kernel_size_sq: usize) -> String {
    format!(
        "#version 430

    layout(local_size_x = 16, local_size_y = 16, local_size_z = 1) in;
    
    uniform uint uWidth;
    uniform uint uHeight;
    uniform int uKernelSize;
    uniform uKernel{{
        float kernel[{kernel_size_sq}];
    }};
    uniform layout(binding=3, rgba32f) image2D uTextureWrite;
    uniform layout(binding=3, rgba32f) image2D uTexture;

    vec4 csample(ivec2 i) {{
        {csample_src}
    }}
    vec3 fun(vec3 v, vec3 prev) {{
        {fun_src}
    }}

    void main() {{
        ivec2 i = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
        if (i.x >= int(uWidth) || i.y >= int(uHeight))
            return;

        vec3 sum = vec3(0.);
        int offset = uKernelSize / 2;
        for (int k = 0; k < uKernelSize*uKernelSize; ++k)
            sum += csample(i + ivec2(mod(k, uKernelSize) - offset, k / uKernelSize - offset)).rgb * vec3(kernel[k]);

        vec4 pixel_sample = imageLoad(uTexture, i);
        imageStore(uTextureWrite, i, vec4(fun(sum, pixel_sample.rgb), pixel_sample.a) );
    }}" )
}