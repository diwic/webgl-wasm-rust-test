// Some code copy-pasted from https://github.com/rustwasm/wasm-bindgen examples, which is MIT licensed.

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlBuffer, WebGlUniformLocation};
use js_sys::WebAssembly;
use nalgebra::{Matrix4, Vector3, Point3, Unit};

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error creating shader".into()))
    }
}

fn link_program<'a, T: IntoIterator<Item = &'a WebGlShader>>(
    context: &WebGlRenderingContext,
    shaders: T,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    for shader in shaders {
        context.attach_shader(&program, shader)
    }
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error creating program object".into()))
    }
}


const VERT_SHADER: &str = 
r#"
    attribute vec4 aVertexPosition;
    uniform vec4 uVertexColor;
    uniform mat4 uModelViewMatrix;
    uniform mat4 uProjectionMatrix;
    varying lowp vec4 vColor;
    void main(void) {
        gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
        vColor = uVertexColor;
    }
"#;

const FRAG_SHADER: &str = 
r#"
varying lowp vec4 vColor;
void main(void) {
    gl_FragColor = vColor;
}
"#;

/*
const VERT_SHADER_TRI: &str = 
r#"
    attribute vec4 position;
    void main() {
        gl_Position = position;
    }
"#;

const FRAG_SHADER_TRI: &str = r#"
    void main() {
        gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    }
"#;

struct TriProgram {
    prog: WebGlProgram,
    buffer: WebGlBuffer,
}

impl TriProgram {
    pub fn new(m: &Main) -> Result<Self, String> {
        let prog = m.make_program(VERT_SHADER_TRI, FRAG_SHADER_TRI)?;
        let buffer = m.make_array_buffer(&[-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0])?;
        Ok(TriProgram { prog, buffer })
    }

    pub fn run(&self, m: &Main) {
        let gl = &m.context;
        gl.use_program(Some(&self.prog));
        gl.vertex_attrib_pointer_with_i32(0, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);
        gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 3);
    }
}
*/

#[derive(Debug)]
struct SquareProgram {
    prog: WebGlProgram,
    vertex_position: u32,
    vertex_color: WebGlUniformLocation,
    model_view_matrix: WebGlUniformLocation,
    projection_matrix: WebGlUniformLocation,
    position_buffer: WebGlBuffer,
}

impl SquareProgram {
    pub fn new(m: &Main) -> Result<Self, String> {
        let prog = m.make_program(VERT_SHADER, FRAG_SHADER)?;
        let gl = &m.context;
        let vertex_position = {
            let v = gl.get_attrib_location(&prog, "aVertexPosition");
            if v < 0 { Err("aVertexPosition not found")? };
            v as u32
        };
        let vertex_color = gl.get_uniform_location(&prog, "uVertexColor").ok_or("uVertexColor not found")?;
        let model_view_matrix = gl.get_uniform_location(&prog, "uModelViewMatrix").ok_or("uModelViewMatrix not found")?;
        let projection_matrix = gl.get_uniform_location(&prog, "uProjectionMatrix").ok_or("uProjectionMatrix not found")?;

        let positions: [f32; 18] = [
            -1.0, -1.0, 0.0,
             1.0, -1.0, 0.0,
             1.0,  1.0, 0.0,

            -1.0, -1.0, 0.0,
             1.0,  1.0, 0.0,
            -1.0,  1.0, 0.0
        ];
        let position_buffer = m.make_array_buffer(&positions)?;

        Ok(SquareProgram { prog, vertex_position, vertex_color, model_view_matrix, projection_matrix, position_buffer })
    }

    pub fn run(&self, m: &Main, color: &[f32; 4]) {
        let gl = &m.context;
        gl.use_program(Some(&self.prog));
        let mut proj_matrix = Matrix4::new_perspective(1200.0 / 700.0, 3.14 / 2.0, 0.1, 10000.0);
        let mut view_matrix = Matrix4::look_at_rh(&Point3::new(0.0, 0.0, -5.0), &Point3::new(0.0, 0.0, 0.0), &Vector3::y());
        proj_matrix = proj_matrix * view_matrix;

        let mut model_matrix = Matrix4::new_scaling(1.0);
        let rot = Matrix4::from_axis_angle(&Unit::new_normalize(Vector3::y()), (m.framecount as f32) / 100.0);
        model_matrix = model_matrix * rot;
 
        // let mut half_matrix: Vec<_> = unit_matrix.iter().map(|&x| x * 0.5).collect();
        gl.uniform_matrix4fv_with_f32_array(Some(&self.projection_matrix), false, proj_matrix.as_mut_slice());
        gl.uniform_matrix4fv_with_f32_array(Some(&self.model_view_matrix), false, model_matrix.as_mut_slice());
        gl.uniform4fv_with_f32_array(Some(&self.vertex_color), &mut color.clone());

        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.position_buffer));
        gl.vertex_attrib_pointer_with_i32(0, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(self.vertex_position);

        gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 6);

        let err = gl.get_error();
        if err != WebGlRenderingContext::NO_ERROR { console::log_1(&format!("Gl error: {}", err).into()); }
        // console::log_1(&format!("No error").into());
        gl.use_program(None);
    }
}

struct Main {
    framecount: u64,
    context: WebGlRenderingContext,
}


impl Main {
    pub fn new() -> Result<Self, JsValue> {
        let document = window().document().expect("should have a document on window");
        let canvas = document
            .create_element("canvas")?
            .dyn_into::<web_sys::HtmlCanvasElement>()?;
        document.body().unwrap().append_child(&canvas)?;
        canvas.set_width(1200);
        canvas.set_height(700);
        canvas.style().set_property("border", "solid")?;

        let context = canvas
            .get_context("webgl")?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()?;

        Ok(Main {
            framecount: 0,
            context: context,
        })
    }

    pub fn make_program(&self, vert_shader: &str, frag_shader: &str) -> Result<WebGlProgram, String> {
        let v = compile_shader(&self.context, WebGlRenderingContext::VERTEX_SHADER, vert_shader)?;
        let f = compile_shader(&self.context, WebGlRenderingContext::FRAGMENT_SHADER, frag_shader)?;
        link_program(&self.context, [v, f].iter())
    }

    pub fn make_array_buffer(&self, data: &[f32]) -> Result<WebGlBuffer, String> {
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>().unwrap().buffer();
        let location = data.as_ptr() as u32 / 4;
        let array = js_sys::Float32Array::new(&memory_buffer)
            .subarray(location, location + data.len() as u32);

        let buffer = self.context.create_buffer().ok_or("failed to create buffer")?;
        self.context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        self.context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &array,
            WebGlRenderingContext::STATIC_DRAW,
        );
        self.context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);
        Ok(buffer)
    }

    pub fn frame(&mut self) {
        let blue = ((self.framecount as f32) / 256.).fract();
        let blue = if blue >= 0.5 { 1.0 - blue } else { blue };
        self.context.clear_color(0.0, 0.5, blue, 1.0);
        self.context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
        self.framecount += 1;
    }
}


// Called by our JS entry point to run the example
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let mut m = Main::new()?;
    let prog = SquareProgram::new(&m)?;
    console::log_1(&format!("{:?}", prog).into());
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
         m.frame();
         prog.run(&m, &[0.5, 0.5, 0.5, 1.0]);
         request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}
