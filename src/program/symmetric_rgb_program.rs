use std::mem::size_of;

use glium::{program::ComputeShader, uniform, Display, Texture2d, uniforms::{UniformBuffer, ImageUnitAccess, ImageUnitFormat}, texture::{UncompressedFloatFormat, MipmapsOption}, Surface};
use yaml_rust::Yaml;

use super::{Program, EdgeSolution};
const WORK_GROUP_SIZE: (u32, u32) = (32, 32);
pub struct SymmetricRgbProgram {
    width: u32,
    height: u32,
    convolution_shader: ComputeShader,
    kernel_buf_hor: UniformBuffer<[f32]>,
    kernel_buf_ver: UniformBuffer<[f32]>,
    kernel_size: usize,
    buffer_texture: Texture2d
}

impl SymmetricRgbProgram {
    pub fn new(
        width: u32,
        height: u32,
        fun: &str,
        kernel_hor: Vec<f32>,
        kernel_ver: Vec<f32>,
        display: &Display,
        edge_solution: EdgeSolution<(f32, f32, f32)>
    ) -> Self {
        let clamp_src = edge_solution.csample_src();

        assert!(kernel_hor.len() == kernel_ver.len());
        let kernel_size = kernel_hor.len();

        let kernel_buf_hor: UniformBuffer<[f32]> = UniformBuffer::empty_unsized_immutable(display, kernel_size*size_of::<f32>()).unwrap();
        kernel_buf_hor.write(&kernel_hor);

        let kernel_buf_ver: UniformBuffer<[f32]> = UniformBuffer::empty_unsized_immutable(display, kernel_size*size_of::<f32>()).unwrap();
        kernel_buf_ver.write(&kernel_ver);

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
                &convolution_shader_src(fun, &clamp_src, kernel_size)
            )
            .unwrap(),
            kernel_buf_hor,
            kernel_buf_ver,
            kernel_size,
            buffer_texture
        }
    }
}

impl Program for SymmetricRgbProgram {
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
                uKernel: &self.kernel_buf_hor,
                uKernelDir: 0,
                uTextureWrite: image_unit,
                uTexture: image_buffer
            }, 
            self.width/WORK_GROUP_SIZE.0 + if self.width % WORK_GROUP_SIZE.0 > 0 {1} else {0}, 
            self.height/WORK_GROUP_SIZE.1 + if self.height % WORK_GROUP_SIZE.1 > 0 {1} else {0}, 
            1
        );

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
                uKernel: &self.kernel_buf_ver,
                uKernelDir: 1,
                uTextureWrite: image_unit,
                uTexture: image_buffer
            }, 
            self.width/WORK_GROUP_SIZE.0 + if self.width % WORK_GROUP_SIZE.0 > 0 {1} else {0}, 
            self.height/WORK_GROUP_SIZE.1 + if self.height % WORK_GROUP_SIZE.1 > 0 {1} else {0}, 
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
            doc["kernelHor"]
                .as_vec()
                .unwrap()
                .into_iter()
                .map(|yaml| {
                    yaml.as_f64().expect(&format!(
                        "Error reading program file: Kernel not a float ({:?})",
                        doc["kernelHor"]
                    )) as f32
                }).collect(),
                doc["kernelVer"]
                .as_vec()
                .unwrap()
                .into_iter()
                .map(|yaml| {
                    yaml.as_f64().expect(&format!(
                        "Error reading program file: Kernel not a float ({:?})",
                        doc["kernelVer"]
                    )) as f32
                }).collect(),
            display,
            edge
        )
    }
}

fn convolution_shader_src(fun_src: &str, csample_src: &str, kernel_size: usize) -> String {
    format!(
        "#version 430

    layout(local_size_x = {}, local_size_y = {}, local_size_z = 1) in;
    
    uniform uint uWidth;
    uniform uint uHeight;
    uniform int uKernelSize;
    uniform uKernel{{
        float kernel[{kernel_size}];
    }};
    uniform int uKernelDir;
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

        vec3 sum = vec3(0);

        int offset = uKernelSize / 2;
        ivec2 p = ivec2(0);
        if (uKernelDir == 0 ) {{
            p.x = 1;
        }} else {{
            p.y = 1;
        }}
        for (int k = 0; k < uKernelSize; ++k)
            sum += csample(i + (k - offset)*p).rgb * vec3(kernel[k]);
        

        vec4 pixel_sample = imageLoad(uTexture, i);
        imageStore(uTextureWrite, i, vec4(fun(sum, pixel_sample.rgb), pixel_sample.a) );
    }}", WORK_GROUP_SIZE.0, WORK_GROUP_SIZE.1 )
}