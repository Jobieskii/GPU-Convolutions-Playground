use glium::{program::ComputeShader, uniform, Display, Texture2d};
use yaml_rust::Yaml;

use super::{Program, EdgeSolution};

pub struct ValProgram {
    width: u32,
    height: u32,
    convolution_shader: ComputeShader,
    swap_shader: ComputeShader,
    kernel: [[f32; 3]; 3],
}

impl ValProgram {
    pub fn new(
        width: u32,
        height: u32,
        fun: &str,
        kernel: &Vec<Vec<f32>>,
        display: &Display,
        edge_solution: EdgeSolution,
        edge_val: Option<f32>
    ) -> Self {
        let mut kernel_arr: [[f32; 3]; 3] = [[0.; 3]; 3];
        for (y, row) in kernel.into_iter().enumerate() {
            for (x, v) in row.into_iter().enumerate() {
                kernel_arr[y][x] = *v;
            }
        }
        let clamp_src = match edge_solution {
            EdgeSolution::Clamp => CSAMPLE_CLAMP_SRC.to_string(),
            EdgeSolution::Wrap => CSAMPLE_WRAP_SRC.to_string(),
            EdgeSolution::Value => csample_val_src(edge_val.unwrap()),
        };
        Self {
            width,
            height,
            convolution_shader: glium::program::ComputeShader::from_source(
                display,
                &convolution_shader_src(fun, &clamp_src)
            )
            .unwrap(),
            swap_shader: glium::program::ComputeShader::from_source(display, &SWAP_SHADER_SRC)
                .unwrap(),
            kernel: kernel_arr,
        }
    }
}

impl Program for ValProgram {
    fn step(&self, board: &mut Texture2d) {
        let image_unit = board
            .image_unit(glium::uniforms::ImageUnitFormat::RGBA32F)
            .unwrap()
            .set_access(glium::uniforms::ImageUnitAccess::ReadWrite);

        self.convolution_shader.execute(
            uniform! { uWidth: self.width, uHeight: self.height, uKernelSize: 3, uKernel: self.kernel, uTexture: image_unit}, 
            self.width/16 + if self.width % 16 > 0 {1} else {0}, 
            self.height/16 + if self.height % 16 > 0 {1} else {0}, 
            1
        );

        let image_unit = board
            .image_unit(glium::uniforms::ImageUnitFormat::RGBA32F)
            .unwrap()
            .set_access(glium::uniforms::ImageUnitAccess::ReadWrite);
        self.swap_shader.execute(
            uniform! { uWidth: self.width, uHeight: self.height, uTexture: image_unit},
            self.width/16 + if self.width % 16 > 0 {1} else {0}, 
            self.height/16 + if self.height % 16 > 0 {1} else {0}, 
            1,
        );
    }

    fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn from_yaml(doc: &Yaml, display: &Display) -> Self {
        let (edge, val) = {
            if let Some(edge) = doc["edge"].as_str() {
                if edge == "wrap" {
                    (EdgeSolution::Wrap, None)
                } else if edge == "clamp" {
                    (EdgeSolution::Clamp, None)
                } else {
                    println!("Invalid edge value!");
                    (EdgeSolution::Clamp, None)
                }
            } else if let Some(val) = doc["edge"].as_f64().map(|x| x as f32) {
                (EdgeSolution::Value, Some(val))
            } else {
                println!("Invalid edge value!");
                (EdgeSolution::Clamp, None)
            }
        };
        Self::new(
            doc["screen"][0].as_i64().unwrap().try_into().unwrap(),
            doc["screen"][1].as_i64().unwrap().try_into().unwrap(),
            doc["fun"].as_str().unwrap(),
            &doc["kernel"]
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
                .collect(),
            display,
            edge,
            val
        )
    }
}

const SWAP_SHADER_SRC: &'static str = r#"
#version 430

layout(local_size_x = 16, local_size_y = 16, local_size_z = 1) in;

uniform uint uWidth;
uniform uint uHeight;
uniform layout(binding=3, rgba32f) image2D uTexture;


void main() {
    ivec2 i = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
    if (i.x >= int(uWidth) || i.y >= int(uHeight))
        return;

    vec4 pixel_sample = imageLoad(uTexture, i);
    imageStore(uTexture, i, vec4(pixel_sample.g, pixel_sample.r, pixel_sample.b, pixel_sample.a) );
}
"#;

fn convolution_shader_src(fun_src: &str, csample_src: &str) -> String {
    format!(
        "#version 430

    layout(local_size_x = 16, local_size_y = 16, local_size_z = 1) in;
    
    uniform uint uWidth;
    uniform uint uHeight;
    uniform int uKernelSize;
    uniform mat3 uKernel;
    uniform layout(binding=3, rgba32f) image2D uTexture;

    vec4 csample(ivec2 i) {{
        {}
    }}
    float fun(float x, float prev) {{
        {}
    }}

    void main() {{
        ivec2 i = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
        int kernelSize = uKernelSize/2;
        if (i.x >= int(uWidth) || i.y >= int(uHeight))
            return;

        float sum = 0;
        for (int y = -kernelSize; y <= kernelSize; ++y) 
        for (int x = -kernelSize; x <= kernelSize; ++x) 
            sum += csample(i + ivec2(x,y)).r * uKernel[y + kernelSize][x + kernelSize];

        vec4 pixel_sample = imageLoad(uTexture, i);
        imageStore(uTexture, i, vec4(pixel_sample.r, fun(sum, pixel_sample.r), pixel_sample.b, pixel_sample.a) );
    }}",
        csample_src, fun_src
    )
}


static CSAMPLE_CLAMP_SRC: &'static str = r#"
i = ivec2(clamp(i.x, 0, int(uWidth)-1), clamp(i.y, 0, int(uHeight)-1));
return imageLoad(uTexture, i);
"#;
static CSAMPLE_WRAP_SRC: &'static str = r#"
i = ivec2(mod(i.x, int(uWidth)), mod(i.y, int(uHeight)));
return imageLoad(uTexture, i);
"#;
fn csample_val_src(val: f32) -> String {
    format!("ivec2 im = ivec2(clamp(i.x, 0, int(uWidth)-1), clamp(i.y, 0, int(uHeight)-1));
    if (i != im) {{
        return vec4({}, 0., 0., 1.);
    }}
    return imageLoad(uTexture, i);", val)
}