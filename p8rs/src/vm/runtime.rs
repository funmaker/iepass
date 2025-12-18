use core::alloc::Allocator;
use core::any::Any;
use alloc::boxed::Box;
use bytemuck::Zeroable;
use p8rs_types::p8num::P8Num;

use crate::utils;
use crate::vm::api;
use crate::vm::callbacks::{Callbacks, DefaultCallbacks};
use crate::vm::memory::{Memory, MemoryAccess};
use crate::vm::memory::machine_state::{BtnpRepDelay, BtnpRepInterval};

#[derive(Debug, Zeroable)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Runtime {
	pub memory: Memory,
	pub cart_memory: [u8; 0x8000],
	pub buttons: Buttons,
	pub target_fps: u16,
	pub time: P8Num,
	#[cfg_attr(feature = "defmt", defmt(Debug2Format))]
	callbacks: Option<Box<dyn Callbacks>>,
	cursor: [i16; 2],
	cursor_home: i16,
}

impl Runtime {
	pub(crate) fn new<A: Allocator>(alloc: A) -> Box<Runtime, A> {
		let mut this: Box<Self, A> = utils::new_zeroed_box_in(alloc);
		
		this.memory.reset();
		this.reseed_rnd();
		this.target_fps = 30;
		this.cursor = [0, 6];
		
		this
	}
	
	pub fn set_callbacks(&mut self, callbacks: impl Callbacks + 'static) {
		self.callbacks = Some(Box::new(callbacks));
		self.reseed_rnd();
	}
	
	pub fn reseed_rnd(&mut self) {
		let seed = self.callbacks().get_rnd_seed();
		api::rnd::srand(self, Some(P8Num::from_raw(seed as i32)));
	}
	
	/// Should be called before every frame
	pub(crate) fn start_frame(&mut self) {
		let buttons = self.callbacks().get_buttons();
		*self.memory.machine_state().btn_state() = buttons;
		
		let delay = match *self.memory.machine_state().btnp_rep_delay() {
			BtnpRepDelay::DEFAULT => 15,
			BtnpRepDelay::DISABLED => 0,
			n => n.get() as u32,
		};
		let delay = delay * self.target_fps as u32 / 30;
		
		let interval = match *self.memory.machine_state().btnp_rep_interval() {
			BtnpRepInterval::DEFAULT => 4,
			n => n.get() as u32,
		};
		let interval = interval * self.target_fps as u32 / 30;
		
		self.buttons.update(buttons, delay, interval);
		
		self.time += P8Num::from(self.target_fps as i16).recip();
	}
	
	/// Returns actual cursor position (memory only contains lower u8 of each coordinate)
	pub fn get_cursor_position(&self) -> [i16; 2] {
		self.cursor
	}
	
	pub fn set_cursor_position(&mut self, pos: [i16; 2]) {
		self.set_cursor_x(pos[0]);
		self.set_cursor_y(pos[1]);
	}
	
	pub fn set_cursor_x(&mut self, val: i16) {
		self.cursor[0] = val;
		self.memory.machine_state()._set_cursor_x(val as u8);
	}

	pub fn set_cursor_y(&mut self, val: i16) {
		self.cursor[1] = val;
		self.memory.machine_state()._set_cursor_y(val as u8);
	}
	
	
	pub fn get_cursor_home(&self) -> i16 {
		self.cursor_home
	}
	
	pub fn set_cursor_home(&mut self, x: i16) {
		self.cursor_home = x;
		self.memory.machine_state()._set_cursor_home_x(x as u8);
	}
	
	pub fn callbacks(&mut self) -> &mut dyn Callbacks {
		&mut **self.callbacks.get_or_insert_with(|| Box::new(DefaultCallbacks))
	}
}

impl p8rs_piccolo::Runtime for Runtime {
	fn as_any(&mut self) -> &mut dyn Any {
		self
	}
	
	fn peek(&mut self, addr: u16) -> u8 {
		self.memory.read(addr)
	}
	
	fn peek2(&mut self, addr: u16) -> u16 {
		self.memory.read(addr)
	}
	
	fn peek4(&mut self, addr: u16) -> u32 {
		self.memory.read(addr)
	}
}

#[derive(Debug, Copy, Clone, Hash, Zeroable)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Buttons {
	pressed: [u8; 8],
	pressed_now: [u8; 8],
	state: [[ButtonState; 8]; 8],
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Zeroable)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
enum ButtonState {
	Done,
	Initial(u32),
	Repeat(u32),
}

#[allow(dead_code)]
impl Buttons {
	pub fn new() -> Buttons {
		Self {
			pressed: [0; 8],
			pressed_now: [0; 8],
			state: [[ButtonState::Done; 8]; 8],
		}
	}
	
	pub fn update(&mut self, buttons: [u8; 8], delay: u32, interval: u32) {
		self.pressed = buttons;
		
		for player in 0..8 {
			let mut pressed_now = 0;
			
			for i in 0..8 {
				let mask = 1 << i;
				
				if self.pressed[player] & mask != 0 { // pressed
					let mut new_state = match self.state[player][i] {
						ButtonState::Done => ButtonState::Initial(0),
						ButtonState::Initial(n) => ButtonState::Initial(n.saturating_add(1)),
						ButtonState::Repeat(n) => ButtonState::Repeat(n.saturating_add(1)),
					};
					
					if let ButtonState::Initial(n) = new_state && delay > 0 && n >= delay {
						new_state = ButtonState::Repeat(0);
					} else if let ButtonState::Repeat(n) = new_state && interval > 0 && n >= interval {
						new_state = ButtonState::Repeat(0);
					}
					
					if new_state.is_pressed_now() { pressed_now |= mask; }
					self.state[player][i] = new_state;
				} else { // not pressed
					self.state[player][i] = ButtonState::Done;
				}
			}
			
			self.pressed_now[player] = pressed_now;
		}
	}
	
	pub fn buttons_pressed(&self, player: usize) -> u8 {
		assert!(player < 8, "player idx > 8");
		self.pressed[player]
	}
	
	pub fn button_pressed(&self, player: usize, button: usize) -> bool {
		assert!(player < 8, "player idx > 8");
		assert!(button < 8, "button idx > 8");
		self.state[player][button].is_pressed()
	}
	
	pub fn buttons_pressed_now(&self, player: usize) -> u8 {
		assert!(player < 8, "player idx > 8");
		self.pressed_now[player]
	}
	
	pub fn button_pressed_now(&self, player: usize, button: usize) -> bool {
		assert!(player < 8, "player idx > 8");
		assert!(button < 8, "button idx > 8");
		self.state[player][button].is_pressed_now()
	}
}

impl ButtonState {
	fn is_pressed(&self) -> bool {
		match self {
			ButtonState::Initial(_) |
			ButtonState::Repeat(_) => true,
			_ => false,
		}
	}
	
	fn is_pressed_now(&self) -> bool {
		match self {
			ButtonState::Initial(0) |
			ButtonState::Repeat(0) => true,
			_ => false,
		}
	}
}
