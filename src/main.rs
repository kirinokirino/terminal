extern crate sdl2;

use std::env;
use std::path::Path;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator, TextureQuery};
use sdl2::video::Window;
use sdl2::video::WindowContext;
use sdl2::Sdl;

static SCREEN_WIDTH: u32 = 800;
static SCREEN_HEIGHT: u32 = 600;

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

// Scale fonts to a reasonable size when they're too big (though they might look less smooth)
fn get_centered_rect(rect_width: u32, rect_height: u32, cons_width: u32, cons_height: u32) -> Rect {
    let wr = rect_width as f32 / cons_width as f32;
    let hr = rect_height as f32 / cons_height as f32;

    let (w, h) = if wr > 1f32 || hr > 1f32 {
        if wr > hr {
            println!("Scaling down! The text will look worse!");
            let h = (rect_height as f32 / wr) as i32;
            (cons_width as i32, h)
        } else {
            println!("Scaling down! The text will look worse!");
            let w = (rect_width as f32 / hr) as i32;
            (w, cons_height as i32)
        }
    } else {
        (rect_width as i32, rect_height as i32)
    };

    let cx = (SCREEN_WIDTH as i32 - w) / 2;
    let cy = (SCREEN_HEIGHT as i32 - h) / 2;
    rect!(cx, cy, w, h)
}

struct App {
    sdl_context: Sdl,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
}

impl App {
    pub fn new() -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsys = sdl_context.video()?;
        let window = video_subsys
            .window("game", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .borderless()
            .resizable()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();
        Ok(Self {
            sdl_context,
            canvas,
            texture_creator,
        })
    }

    pub fn run(&mut self) -> Result<(), String> {
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

        // Load a font
        //let font_path: &Path = Path::new("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc");
        let font_path: &Path = Path::new("./assets/fonts/VictorMono-Regular.ttf");
        let mut font = ttf_context.load_font(font_path, 24)?;
        font.set_style(sdl2::ttf::FontStyle::NORMAL);

        // render a surface, and convert it to a texture bound to the canvas
        let surface = font
            .render("Hello Rust! Здравствуй Раст! \nこんいちはRUST！")
            .blended_wrapped(Color::RGBA(170, 170, 170, 255), 0)
            .map_err(|e| e.to_string())?;
        let texture = self
            .texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        self.canvas.set_draw_color(Color::RGBA(50, 50, 50, 255));
        self.canvas.clear();

        let TextureQuery { width, height, .. } = texture.query();

        // If the example text is too big for the screen, downscale it (and center irregardless)
        // let padding = 64;
        // let target = get_centered_rect(
        //     width,
        //     height,
        //     SCREEN_WIDTH - padding,
        //     SCREEN_HEIGHT - padding,
        // );

        let target = rect!(0, 0, width, height);

        self.canvas.copy(&texture, None, Some(target))?;
        self.canvas.present();

        'mainloop: loop {
            for event in self.sdl_context.event_pump()?.poll_iter() {
                match event {
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    }
                    | Event::Quit { .. } => break 'mainloop,
                    other => {
                        println!("{:?}", other)
                    }
                }
            }
        }

        Ok(())
    }
}

fn main() -> Result<(), String> {
    let args: Vec<_> = env::args().collect();

    println!("linked sdl2_ttf: {}", sdl2::ttf::get_linked_version());

    let mut app = App::new()?;
    app.run()?;

    Ok(())
}
