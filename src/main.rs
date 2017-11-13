extern crate sdl2;
extern crate vec2d;

use std::path::Path;

use sdl2::event::{Event,WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::image::LoadTexture;
use std::time::Duration;
use std::cmp::min;
use vec2d::{Vec2D, Coord, Size};


#[derive(PartialEq)]
#[derive(Debug)]
enum CharacterStateEnum {
	Idle,
	Walking(u8),
	Running(u8),
	Jumping(u8),
}

struct CharacterState {
	state: CharacterStateEnum,
	facing_left: bool,
}

struct Character<'a> {
	texture: Texture<'a>,
	height: u32,
	width: u32,
	state: CharacterState,
	pos: (i32, i32, i32),
	velocity: (i32, i32, i32),
}

struct Controls {
	up: bool,
	down: bool,
	right: bool,
	left: bool,
}

struct Map {
	altitude: [[i32]],
}

static GRAVITY: [i32; 5] = [5, 2, 0, -2, -5];

impl<'a> Character<'a> {
	fn update(&mut self, controls: &Controls, map: &Map) {
		self.update_animation(map);
		self.update_velocity(controls);
		let (x, y, z) = self.pos;
		let (vx, vy, vz) = self.velocity;
		self.pos = (x+vx, y+vy, z+vz);
	}

	fn update_animation(&mut self, map: &Map) {
		match self.state.state {
			CharacterStateEnum::Jumping(i) => {
				if i > 0 && 
					self.pos.2 <= *map.altitude
						.get(Coord { x: self.pos.0 as usize, y: self.pos.1 as usize}).unwrap() {
					self.state.state = CharacterStateEnum::Idle;
				}
				else {
					self.state.state = CharacterStateEnum::Jumping(i + 1);
				}
			},
			CharacterStateEnum::Running(i) => {
				if i < 29 { self.state.state = CharacterStateEnum::Running(i + 1) } 
				else { self.state.state = CharacterStateEnum::Running(0) }
			},
			CharacterStateEnum::Walking(i) => {
				if i < 39 { self.state.state = CharacterStateEnum::Walking(i + 1) } 
				else { self.state.state = CharacterStateEnum::Walking(0) }
			},
			CharacterStateEnum::Idle => (),
		};
	}

	fn update_velocity(&mut self, controls: &Controls) {
		self.velocity = (0, 0, 0);
		match self.state.state {
			CharacterStateEnum::Jumping(i) => {
				self.velocity.2 = GRAVITY[(i-1) as usize/10];
				if controls.left && !controls.right { self.velocity.0 = -2 }
				else if controls.right && !controls.left { self.velocity.0 = 2 }
			},
			CharacterStateEnum::Running(_) => {
			 	if controls.left && !controls.right { self.velocity.0 = -5 }
				else if controls.right && !controls.left { self.velocity.0 = 5 }
			},
			_ => (),
		};
		if controls.up && !controls.down { self.velocity.1 = -2 }
		else if controls.down && !controls.up { self.velocity.1 = 2 }
	}

	fn display(&self, renderer: &mut Canvas<Window>, shadow: &Texture) -> Result<(), String> {
		let (offsetx, offsety) = self.display_offset(&self.state.state);
		let (x, y, z) = self.pos;
		if let Err(a) = renderer.copy(shadow, None, Rect::new(x, y + self.height as i32 - 20/2, 79, 20)) {
			return Err(a);
		}
	    renderer.copy_ex(&self.texture, Rect::new(offsetx,offsety,self.height,self.width),
	 		Rect::new(x, y-z, self.height, self.width),
	 		0., None, self.state.facing_left, false)
	}

	fn display_offset(&self, state: &CharacterStateEnum) -> (i32, i32) {
		match *state {
			CharacterStateEnum::Idle => (0, 0),
			CharacterStateEnum::Walking(i) => (i as i32 / 10 * 80 + 240, 0),
			CharacterStateEnum::Running(i) => (i as i32 / 10 * 80, 160),
			CharacterStateEnum::Jumping(i) => (min((i-1) as i32 / 10, 4) * 80, 640),
		}
	}

	fn run(&mut self, left: bool) {
		match self.state.state {
			CharacterStateEnum::Idle => self.state.state = CharacterStateEnum::Running(0),
			CharacterStateEnum::Walking(_) => self.state.state = CharacterStateEnum::Running(0),
			_ => (),
		}
		self.state.facing_left = left;
	}

	fn stop_run(&mut self) {
		match self.state.state {
			CharacterStateEnum::Running(_) => self.state.state = CharacterStateEnum::Idle,
			_ => (),
		}
	}

	fn jump(&mut self) {
		match self.state.state {
			CharacterStateEnum::Jumping(_) => (),
			_ => self.state.state = CharacterStateEnum::Jumping(0),
		}
	}

	fn walk_y(&mut self) {
		match self.state.state {
			CharacterStateEnum::Idle => self.state.state = CharacterStateEnum::Walking(0),
			_ => (),
		}
	}

	fn control(&mut self, controls: &Controls) {
        if controls.up || controls.down { self.walk_y() }
    	else if let CharacterStateEnum::Walking(_) = self.state.state {
    		self.state.state = CharacterStateEnum::Idle 
    	}
    	if controls.right { self.run(false) }
    	else if controls.left { self.run(true) }
    	else { self.stop_run() };
	}
}

fn main() {
    let ctx = sdl2::init().unwrap();
    let video_ctx = ctx.video().unwrap();

    let window  = video_ctx.window("eg05", 500, 434).position_centered().build().unwrap();

    let mut renderer = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = renderer.texture_creator();

    // Convert a surface to a texture.
    // Textures can be used more efficiently by the GPU. (If one is available.)
    let mut charac = Character {
    	texture: texture_creator.load_texture(&Path::new("gad_0.png")).unwrap(),
    	height: 79,
    	width: 79,
    	state: CharacterState { state: CharacterStateEnum::Idle, facing_left: false },
    	pos: (0, 434 - 79, 0),
    	velocity: (0, 0, 0),
    };
    let background = texture_creator.load_texture(&Path::new("bg5.bmp")).unwrap();
    let shadow = texture_creator.load_texture(&Path::new("shadow.png")).unwrap();
    let mut timer = ctx.timer().unwrap();

    let mut events = ctx.event_pump().unwrap();
    let mut controls = Controls{up: false, down: false, right: false, left: false};
    let map = Vec2D::from_example(Size {width: 500, height: 434}, 0);

    // loop until we receive a QuitEvent or press escape.
    'event : loop {
    	let ticks = timer.ticks() as i32;
        for event in events.poll_iter() {
            match event {
                Event::Quit{..} => break 'event,
                Event::Window {win_event, ..} => {
                    match win_event {
                        // refresh our window, for example if it is no longer
                        // covered by other windows.
                        WindowEvent::Exposed => renderer.present(),
                        _ => (),
                    }
                }
                Event::KeyDown {keycode: Some(keycode), ..} => {
                    match keycode {
                    	Keycode::Escape => break 'event,
                    	Keycode::Up => controls.up = true,
                    	Keycode::Down => controls.down = true,
                    	Keycode::Right => controls.right = true,
                    	Keycode::Left => controls.left = true,
                    	Keycode::Space => charac.jump(),
                    	_ => (),
                	}
                }
                Event::KeyUp {keycode: Some(keycode), ..} => {
                    match keycode {
                    	Keycode::Up => controls.up = false,
                    	Keycode::Down => controls.down = false,
                    	Keycode::Right => controls.right = false,
                    	Keycode::Left => controls.left = false,
                    	_ => (),
                	}
                }
                _               => continue
            }
        }

        charac.control(&controls);


        charac.update(&controls);
	    renderer.clear();
	    renderer.copy(&background, Rect::new(0,0,500,434), None).unwrap();
	    charac.display(&mut renderer, &shadow).unwrap();
	    renderer.present();

	    let lag = ticks + 16 - timer.ticks() as i32;
	    if lag > 0 {
        	std::thread::sleep(Duration::from_millis(lag as u64));
	    }
    }
}