use core::alloc::Allocator;
use core::ops::{Div, Not};
use alloc::alloc::Global;
use alloc::boxed::Box;

use crate::pico8::memory::Memory;
use crate::utils;

pub struct Env<A: Allocator = Global> {
	pub cart_memory: Box<[u8; 0x8000], A>,
	pub memory: Memory<A>,
	pub buttons: Buttons,
	pub fps: u8,
}

impl<A: Allocator + Clone> Env<A> {
	pub fn new(alloc: A) -> Env<A> {
		Self {
			cart_memory: utils::new_zeroed_box_in(alloc.clone()),
			memory: Memory::new(alloc),
			buttons: Buttons::new(),
			fps: 30,
		}
	}
	
	pub fn finish_update_frame(&mut self) {
		self.buttons.finish_update_frame(self.fps);
	}
	
	pub fn update_buttons(&mut self, buttons: &[u8; 8]) {
		self.buttons.update_state(buttons);
		for i in 0..8 {
			self.memory[0x5f4c + i] = buttons[i] & 0x3f;
		}
	}
}

pub struct Buttons {
	buttons: [u8; 8],
	buttons_repeat_state: [u8; 8],
	buttons_held_frames: [u8; 8*8],
}

#[allow(dead_code)]
impl Buttons {
	pub fn new() -> Buttons {
		Self {
			buttons: [0; 8],
			buttons_repeat_state: [0; 8],
			buttons_held_frames: [0; 8*8],
		}
	}
	
	pub fn finish_update_frame(&mut self, fps: u8) {
		for player in 0..8 {
			let buttons = self.buttons[player];
			
			for i in 0..8 {
				let mask = 1 << i;
				let idx = player * 8 + i;
				
				if buttons & mask != 0 { // button is pressed
					self.buttons_held_frames[idx] += 1;
					
					let repeating = self.buttons_repeat_state[player] & mask != 0;
					let limit = fps.div(if repeating { 7 } else { 2 });
					
					if self.buttons_held_frames[idx] >= limit {
						self.buttons_held_frames[idx] = 0;
						if !repeating {
							self.buttons_held_frames[player] |= mask;
						}
					}
				} else { // button is not pressed
					self.buttons_held_frames[idx] = 0;
					self.buttons_repeat_state[player] &= mask.not();
				}
			}
		}
	}
	
	fn update_state(&mut self, state: &[u8; 8]) {
		self.buttons.copy_from_slice(state);
	}
	
	fn get_mask_for_player(&self, player: usize) -> u8 {
		assert!(player < 8, "player idx > 8");
		self.buttons[player]
	}
	
	fn is_down(&self, player: usize, button: usize) -> bool {
		assert!(button < 8, "button idx > 8");
		(self.get_mask_for_player(player) & (1 << button)) != 0
	}
	
	fn is_just_pressed(&self, player: usize, button: usize) -> bool {
		self.is_down(player, button) && self.buttons_held_frames[player * 8 + button] == 1
	}
}
