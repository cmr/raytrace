#![feature(phase)]

extern crate cgmath;
extern crate gfx;
extern crate glfw;
extern crate glfw_platform;
#[phase(plugin)]
extern crate gfx_macros;

use glfw_platform::BuilderExtension;

#[vertex_format]
struct Vertex {
    pos: [f32, ..2]
}
#[shader_param]
struct Param {
    tex: gfx::shade::TextureParam
}

static VERTEX: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    in vec2 pos;
    out vec2 uv;
    void main() {
        gl_Position = vec4(pos, 0.0, 1.0);
        uv = pos;
    }
"
};
static FRAGMENT: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    uniform sampler2D tex;
    in vec2 uv;
    out vec4 color;
    void main() {
        color = texture(tex, uv);
    }
"
};

fn main() {
    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let (mut window, events) = glfw_platform::WindowBuilder::new(&glfw)
        .title("Raytracer")
        .size(600, 600)
        .try_modern_context_hints()
        .create()
        .expect("Could not create GLFW window :-(");

    window.set_key_polling(true);

    let mut device = gfx::build()
        .with_glfw_window(&mut window)
        .with_queue_size(2)
        .spawn(proc(rend) render(rend))
        .unwrap();

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::KeyEvent(glfw::KeyEscape, _, glfw::Press, _) => {
                    window.set_should_close(true);
                }
                _ => { }
            }
        }
        device.update();
    }
    device.close();
}

fn render(mut renderer: gfx::Renderer) {
    // we want the alpha to be 1
    let tex_data = Vec::from_fn(600*600, |_| std::rand::random::<u32>() | 0xFF000000);

    let frame = gfx::Frame::new(600, 600);
    let state = gfx::DrawState::new();

    let vertex_data = vec![
        Vertex { pos: [-1.0, -1.0] },
        Vertex { pos: [-1.0, 1.0] },
        Vertex { pos: [1.0, -1.0] },
        Vertex { pos: [1.0, 1.0] },
    ];
    let mesh = renderer.create_mesh(vertex_data);
    let slice = {
        let buf = renderer.create_buffer(Some(vec![0i, 2, 3, 0, 3, 1]));
        gfx::IndexSlice(buf, 0, 6)
    };

    let texinfo = gfx::tex::TextureInfo {
        width: 600,
        height: 600,
        depth: 1,
        mipmap_range: (0, -1),
        kind: gfx::tex::Texture2D,
        format: gfx::tex::RGBA8
    };
    let imginfo = texinfo.to_image_info();
    let texture = renderer.create_texture(texinfo);
    renderer.update_texture(texture, imginfo, tex_data);

    let sampler = renderer.create_sampler(gfx::tex::SamplerInfo::new(gfx::tex::Scale, gfx::tex::Clamp));
    let program = renderer.create_program(VERTEX.clone(), FRAGMENT.clone());
    let program = renderer.connect_program(program, Param { tex: (texture, Some(sampler)) }).unwrap();

    let clear = gfx::ClearData {
        color: Some(gfx::Color([0.3, 0.3, 0.3, 1.0])),
        depth: None,
        stencil: None
    };

    while !renderer.should_finish() {
        renderer.clear(clear, frame);
        renderer.draw(&mesh, slice, &frame, &program, &state).unwrap();
        renderer.end_frame();
        for err in renderer.errors() {
            println!(":-( {}", err);
        }
    }
}
