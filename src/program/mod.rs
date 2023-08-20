pub mod val_program;
pub mod rgb_program;

use glium::{Texture2d, Display};
use yaml_rust::Yaml;

pub trait Program {
    fn from_yaml(doc: &Yaml, display: &Display) -> Self;
    fn step(&self, board: &mut Texture2d);
    fn get_dimensions(&self) -> (u32, u32);
}

pub enum EdgeSolution<T> {
    Clamp, Wrap, Value(T)
}
impl EdgeSolution<f32> {
    pub fn csample_src(self) -> String {
        match self {
            EdgeSolution::Clamp => CSAMPLE_CLAMP_SRC.to_string(),
            EdgeSolution::Wrap => CSAMPLE_WRAP_SRC.to_string(),
            EdgeSolution::Value(x) => Self::csample_val_src_f32(x),
        }
    }
    fn csample_val_src_f32(val: f32) -> String {
        format!("ivec2 im = ivec2(clamp(i.x, 0, int(uWidth)-1), clamp(i.y, 0, int(uHeight)-1));
        if (i != im) {{
            return vec4({}, 0., 0., 1.);
        }}
        return imageLoad(uTexture, i);", val)
    }
}
impl EdgeSolution<(f32, f32, f32)> {
    pub fn csample_src(self) -> String {
        match self {
            EdgeSolution::Clamp => CSAMPLE_CLAMP_SRC.to_string(),
            EdgeSolution::Wrap => CSAMPLE_WRAP_SRC.to_string(),
            EdgeSolution::Value(x) => Self::csample_val_src_3f32(x),
        }
    }
    fn csample_val_src_3f32(val: (f32, f32, f32)) -> String {
        format!("ivec2 im = ivec2(clamp(i.x, 0, int(uWidth)-1), clamp(i.y, 0, int(uHeight)-1));
        if (i != im) {{
            return vec4({}, {}, {}, 1.);
        }}
        return imageLoad(uTexture, i);", val.0, val.1, val.2)
    }
}


const CSAMPLE_CLAMP_SRC: &'static str = r#"
i = ivec2(clamp(i.x, 0, int(uWidth)-1), clamp(i.y, 0, int(uHeight)-1));
return imageLoad(uTexture, i);
"#;
const CSAMPLE_WRAP_SRC: &'static str = r#"
i = ivec2(mod(i.x, int(uWidth)), mod(i.y, int(uHeight)));
return imageLoad(uTexture, i);
"#;

