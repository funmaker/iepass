use core::any::Any;

pub trait Runtime {
	fn as_any(&mut self) -> &mut dyn Any;
	fn peek(&mut self, addr: u16) -> u8;
	
	fn peek2(&mut self, addr: u16) -> u16 {
		u16::from_be_bytes([
			self.peek(addr),
			self.peek(addr.wrapping_add(1)),
		])
	}
	
	fn peek4(&mut self, addr: u16) -> u32 {
		u32::from_be_bytes([
			self.peek(addr),
			self.peek(addr.wrapping_add(1)),
			self.peek(addr.wrapping_add(2)),
			self.peek(addr.wrapping_add(3)),
		])
	}
	
	fn as_ref(&mut self) -> RuntimeRef<'_> where Self: Sized {
		self as RuntimeRef
	}
}

pub type RuntimeRef<'a> = &'a mut dyn Runtime;

impl dyn Runtime + '_ {
	pub fn reborrow<'a, 'b: 'a>(&'b mut self) -> RuntimeRef<'a> {
		&mut *self
	}
	
	pub fn downcast<T: 'static>(&mut self) -> &mut T {
		self.as_any().downcast_mut().expect("Mismatched Runtime type")
	}
}

impl Runtime for () {
	fn as_any(&mut self) -> &mut dyn Any {
		self
	}
	
	fn peek(&mut self, _addr: u16) -> u8 {
		panic!("Attempted to peek without runtime.");
	}
}
