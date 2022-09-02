#![warn(clippy::nursery, clippy::pedantic)]
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::must_use_candidate,
    clippy::missing_panics_doc
)]

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator, TextureQuery};
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;

use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

static SCREEN_WIDTH: u16 = 800;
static SCREEN_HEIGHT: u16 = 600;

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
        println!("Scaling down! The text will look worse!");
        if wr > hr {
            let h = (rect_height as f32 / wr) as i32;
            (cons_width as i32, h)
        } else {
            let w = (rect_width as f32 / hr) as i32;
            (w, cons_height as i32)
        }
    } else {
        (rect_width as i32, rect_height as i32)
    };

    let cx = (i32::from(SCREEN_WIDTH) - w) / 2;
    let cy = (i32::from(SCREEN_HEIGHT) - h) / 2;
    rect!(cx, cy, w, h)
}

struct App {
    sdl_context: Sdl,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,

    buffer: String,
    command_line: String,
}

impl App {
    pub fn new() -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsys = sdl_context.video()?;
        let window = video_subsys
            .window("game", SCREEN_WIDTH.into(), SCREEN_HEIGHT.into())
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

            buffer: String::new(),
            command_line: String::new(),
        })
    }

    pub fn run_command(command_line: &str) -> Result<String, Box<dyn Error>> {
        if command_line.is_empty() {
            return Err("Running empty command".into());
        }
        let mut result = String::new();

        // assume one command in command_line
        let words: Vec<&str> = command_line.split_ascii_whitespace().collect();
        let command = words[0];
        let arguments = &words[0..];
        if let Some(command) = find_command(command) {
            result = Command::new(command)
                .args(arguments)
                .output()
                .map(|out| String::from_utf8(out.stdout))??;
        }

        Ok(result)
    }

    pub fn run(&mut self) -> Result<(), String> {
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

        // Load a font
        //let font_path: &Path = Path::new("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc");
        let font_path: &Path = Path::new("./assets/fonts/VictorMono-Regular.ttf");
        let mut font = ttf_context.load_font(font_path, 24)?;
        font.set_style(sdl2::ttf::FontStyle::NORMAL);

        'mainloop: loop {
            match self.input() {
                Err(error) => {
                    println!("{error}");
                    break;
                }
                Ok(events) => {
                    for event in events {
                        match event {
                            AppAction::None => (),
                            AppAction::Exit => break 'mainloop,
                            AppAction::RunCommand => {
                                if let Ok(append) = Self::run_command(&self.command_line) {
                                    self.buffer.push_str(&append);
                                    self.command_line = String::new();
                                }
                            }
                        }
                    }
                }
            }

            let prompt = "> ";
            // render a surface, and convert it to a texture bound to the canvas
            let surface = font
                .render(&format!("{}\n{prompt}{}", &self.buffer, &self.command_line))
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
        }

        Ok(())
    }

    fn input(&mut self) -> Result<Vec<AppAction>, String> {
        let mut events = Vec::new();
        for event in self.sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::TextInput { .. } | Event::TextEditing { .. } => (),
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => events.push(AppAction::Exit),
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => events.push(AppAction::RunCommand),
                Event::KeyDown {
                    keycode: Some(Keycode::Backspace),
                    keymod: Mod::LSHIFTMOD,
                    ..
                } => {
                    let mut words: Vec<&str> = self.command_line.split_ascii_whitespace().collect();
                    words.pop();
                    self.command_line = words.join(" ");
                }
                Event::KeyDown {
                    keycode: Some(key),
                    keymod: Mod::NOMOD,
                    ..
                } => match key {
                    Keycode::Backspace => {
                        self.command_line.pop();
                    }
                    Keycode::A => self.command_line.push('a'),
                    Keycode::B => self.command_line.push('b'),
                    Keycode::C => self.command_line.push('c'),
                    Keycode::D => self.command_line.push('d'),
                    Keycode::E => self.command_line.push('e'),
                    Keycode::F => self.command_line.push('f'),
                    Keycode::G => self.command_line.push('g'),
                    Keycode::H => self.command_line.push('h'),
                    Keycode::I => self.command_line.push('i'),
                    Keycode::J => self.command_line.push('j'),
                    Keycode::K => self.command_line.push('k'),
                    Keycode::L => self.command_line.push('l'),
                    Keycode::M => self.command_line.push('m'),
                    Keycode::N => self.command_line.push('n'),
                    Keycode::O => self.command_line.push('o'),
                    Keycode::P => self.command_line.push('p'),
                    Keycode::Q => self.command_line.push('q'),
                    Keycode::R => self.command_line.push('r'),
                    Keycode::S => self.command_line.push('s'),
                    Keycode::T => self.command_line.push('t'),
                    Keycode::U => self.command_line.push('u'),
                    Keycode::V => self.command_line.push('v'),
                    Keycode::W => self.command_line.push('w'),
                    Keycode::X => self.command_line.push('x'),
                    Keycode::Y => self.command_line.push('y'),
                    Keycode::Z => self.command_line.push('z'),
                    Keycode::Space => self.command_line.push(' '),
                    Keycode::Slash => self.command_line.push('/'),
                    Keycode::Period => self.command_line.push('.'),
                    key => println!("Unhandled NOMOD {:?}", key),
                },

                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Space => self.command_line.push(' '),
                    Keycode::Slash => self.command_line.push('/'),
                    Keycode::Period => self.command_line.push('.'),
                    key => println!("Unhandled {:?}", key),
                },
                other => {
                    println!("{:?}", other);
                }
            }
        }
        if events.is_empty() {
            events.push(AppAction::None);
        }
        Ok(events)
    }
}

impl Drop for App {
    fn drop(&mut self) {}
}

#[derive(Debug, Clone, Copy)]
enum AppAction {
    None,
    Exit,
    RunCommand,
}

fn main() -> Result<(), String> {
    let _args: Vec<_> = env::args().collect();

    println!("linked sdl2_ttf: {}", sdl2::ttf::get_linked_version());

    let mut app = App::new()?;
    app.run()?;

    Ok(())
}

pub fn find_command(to_check: &str) -> Option<PathBuf> {
    let path = env::var("PATH").unwrap_or_else(|_| String::new());
    let paths = env::split_paths(&path);
    for path in paths {
        if let Ok(metadata) = path.metadata() {
            if metadata.is_dir() {
                let candidate = path.join(to_check);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

pub fn find_commands(to_check: &[&str]) -> Vec<Option<PathBuf>> {
    let mut found: Vec<Option<PathBuf>> = vec![None; to_check.len()];
    let path = env::var("PATH").unwrap_or_else(|_| String::new());
    let paths = env::split_paths(&path);
    for path in paths {
        if let Ok(metadata) = path.metadata() {
            if metadata.is_dir() {
                for (i, to_check) in to_check.iter().enumerate() {
                    let candidate = path.join(to_check);
                    if candidate.exists() {
                        std::mem::swap(found.get_mut(i).unwrap(), &mut Some(candidate));
                    }
                }
            }
        }
    }

    found
}
