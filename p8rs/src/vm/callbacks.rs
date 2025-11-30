use core::fmt::Debug;
use p8rs_types::p8scii;

#[allow(unused_variables)]
pub trait Callbacks: Debug {
	fn printh(&mut self, text: &[u8], filename: Option<&[u8]>, overwrite: Option<bool>, save_to_desktop: Option<bool>) {
		println!("INFO: {}", p8scii::Display(text));
	}
	
	fn get_buttons(&mut self) -> [u8; 8] {
		[0; 8]
	}
}

#[derive(Debug)]
pub struct DefaultCallbacks;
impl Callbacks for DefaultCallbacks {}
