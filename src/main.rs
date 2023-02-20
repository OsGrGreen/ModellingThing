use rustCad::*;

use beryllium::*;
use ultraviolet::*;

use core::{
    convert::{TryFrom, TryInto},
    mem::{size_of, size_of_val},
};
use ogl33::*;

use std::ffi::CString;
use std::time::UNIX_EPOCH;

type Vertex = [f32; 3];
type TriIndexes = [u32; 3];

const WINDOW_TITLE: &str = "RUST CAD";

const VERT_SHADER: &str = r#"#version 330 core
  layout (location = 0) in vec3 pos;

    out VS_OUTPUT {
        vec3 colPos;
    }OUT;

  void main() {
    gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
    OUT.colPos = pos;
  }
"#;

const FRAG_SHADER: &str = r#"#version 330 core
  out vec4 final_color;

  in VS_OUTPUT {
    vec3 colPos;
  } IN;

  void main() {
    final_color = vec4(IN.colPos, 1.0);;
  }
"#;

fn main() {
    // Initiate all of the libraries (turn on SDL)
    let sdl = SDL::init(InitFlags::Everything).expect("couldn't start SDL");
    // Unwrap and init OPENGL
    sdl.gl_set_attribute(SdlGlAttr::MajorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::MinorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::Profile, GlProfile::Core)
        .unwrap();
    // Set forwardcompatible flag for macos, is not needed for other operatoringsystems
    #[cfg(target_os = "macos")]
    {
        sdl.gl_set_attribute(SdlGlAttr::Flags, ContextFlag::ForwardCompatible)
            .unwrap();
    }

    // Creates the window
    let win = sdl
        .create_gl_window(
            WINDOW_TITLE,             // Title of the window
            WindowPosition::Centered, // How the window should be posistioned
            1280,                     //width
            720,                      // height
            WindowFlags::Shown,
        )
        .expect("couldn't make a window and context");
    win.set_swap_interval(SwapInterval::Vsync); // Enable Vsync

    unsafe {
        // Load every openGL function into OGL33, and make it usable
        load_gl_with(|f_name| win.get_proc_address(f_name)); //
    }
    rustCad::clear_color(0.2, 0.3, 0.3, 1.0);

    //Prepare stuff

    let shader_program = ShaderProgram::from_vert_frag(VERT_SHADER, FRAG_SHADER).unwrap();
    shader_program.use_program();
    // The vertex_array_object
    let vao = VertexArray::new().expect("Could not make Vertex Array Object");

    let mut vert_vec: Vec<Vertex> = Vec::new();
    vert_vec.push([0.5, 0.5, 0.0]);
    vert_vec.push([0.5, -0.5, 0.0]);
    vert_vec.push([-0.5, -0.5, 0.0]);
    vert_vec.push([-0.5, 0.5, 0.0]);
    let mut indicies_vec: Vec<TriIndexes> = Vec::new();
    indicies_vec.push([0, 1, 3]);
    indicies_vec.push([1, 2, 3]);
    let mut vbo = create_vbo(&vao, &vert_vec);
    let mut ebo = create_ebo(&vao, &indicies_vec);
    let mut k = -1.0;

    rustCad::polygon_mode(rustCad::PolygonMode::Fill);
    'main_loop: loop {
        // handle events this frame
        while let Some(event) = sdl.poll_events().and_then(Result::ok) {
            match event {
                Event::Quit(_) => break 'main_loop,
                //Event::Keyboard(_) => update_VBO(vao, vert_vec),
                Event::Keyboard(e) => {
                    if e.is_pressed && e.key.keycode == Keycode::SPACE {
                        vert_vec[2] = [0.5-k/2.0, 0.5, 0.0];
                        k = k * -1.0;
                        vbo = update_single_vbo(2, &vao, vbo, vert_vec[2]);
                    }else if e.is_pressed && e.key.keycode == Keycode::L {
                        rustCad::polygon_mode(rustCad::PolygonMode::Line);
                    }else if e.is_pressed && e.key.keycode == Keycode::F {
                        rustCad::polygon_mode(rustCad::PolygonMode::Fill);
                    }else if e.is_pressed && e.key.keycode == Keycode::P {
                        rustCad::polygon_mode(rustCad::PolygonMode::Point);
                    }else if e.is_pressed && e.key.keycode == Keycode::ESCAPE {
                        break 'main_loop;
                    }
                }
                _ => (),
            }
        }

        //Make changes

        //DRAW
        unsafe {
            glClear(GL_COLOR_BUFFER_BIT);
            //glDrawArrays(GL_TRIANGLES, 0, vert_vec.len().try_into().unwrap());
            glDrawElements(
                GL_TRIANGLES,
                (indicies_vec.len()*3).try_into().unwrap(),
                GL_UNSIGNED_INT,
                0 as *const _,
            );
        }
        win.swap_window(); // Swap the draw_buffer and the display buffer which actually displays what we have drawn.
    }
}

pub fn update_vbo(
    vert_offset: usize,
    vao: &rustCad::VertexArray,
    vbo: rustCad::VertexBuffer,
    verts: &Vec<Vertex>,
) -> rustCad::VertexBuffer {
    let vertices: &[Vertex] = &verts[..];
    let offset = vert_offset * size_of::<Vertex>();
    vao.bind();
    vbo.bind(BufferType::Array);
    rustCad::buffer_sub_data(BufferType::Array, bytemuck::cast_slice(vertices), offset);
    return vbo;
}

pub fn update_single_vbo(
    vertex_num: usize,
    vao: &rustCad::VertexArray,
    vbo: rustCad::VertexBuffer,
    vert: Vertex,
) -> rustCad::VertexBuffer {
    let vertices: &[Vertex] = &[vert];
    let offset = vertex_num * size_of::<Vertex>();
    vao.bind();
    vbo.bind(BufferType::Array);
    rustCad::buffer_sub_data(BufferType::Array, bytemuck::cast_slice(vertices), offset);
    return vbo;
}

pub fn update_whole_vbo(
    vao: &rustCad::VertexArray,
    vbo: rustCad::VertexBuffer,
    verts: &Vec<Vertex>,
) -> rustCad::VertexBuffer {
    let vertices: &[Vertex] = &verts[..];
    vao.bind();
    vbo.bind(BufferType::Array);
    rustCad::buffer_sub_data(BufferType::Array, bytemuck::cast_slice(vertices), 0);
    return vbo;
}

pub fn create_vbo(vao: &rustCad::VertexArray, verts: &Vec<Vertex>) -> rustCad::VertexBuffer {
    vao.bind();

    let vertices: &[Vertex] = &verts[..];
    let vbo = VertexBuffer::new().expect("Could not make Vertex Buffer Object");
    vbo.bind(BufferType::Array);
    rustCad::buffer_data(
        BufferType::Array,
        bytemuck::cast_slice(vertices),
        GL_DYNAMIC_DRAW,
    );

    unsafe {
        // How will the GPU know the correct way to use/interpret the data we sent it? We describe the "vertex attributes" and then it will be able to interpret these correctly
        // For each vertex attribute we have to call "glVertexAttribPointer"
        glVertexAttribPointer(
            0,        // The index of the attribute we want to describe
            3, // The number of components in the attribute (in this case 3 since each posistion consists of 3D XYZ posistion)
            GL_FLOAT, // The type of data in/for the attribute
            GL_FALSE, // Has to do fixed_point data values, dunno cheif
            //Alternately, we can use size_of::<f32>() * 3
            size_of::<Vertex>().try_into().unwrap(), // "The number of bytes from the start of this attribute in one vertex to the start of the same attribute in the next vertex"
            0 as *const _, // (pointer to) The starting point of the vertex attribute in the buffer
        );
        glEnableVertexAttribArray(0);
    }

    return vbo;
}

pub fn create_ebo(vao: &rustCad::VertexArray, inds: &Vec<TriIndexes>) -> rustCad::VertexBuffer {
    vao.bind();

    let indicies: &[TriIndexes] = &inds[..];

    let ebo = VertexBuffer::new().expect("Could not make Element Buffer Object");
    ebo.bind(BufferType::ElementArray);
    rustCad::buffer_data(
        BufferType::ElementArray,
        bytemuck::cast_slice(indicies),
        GL_DYNAMIC_DRAW,
    );
    return ebo;
}