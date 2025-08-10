use core::ops::{Deref, DerefMut};
use esp_hal::dma::{DmaDescriptor, DmaTxBuffer, Preparation, TransferDirection, CHUNK_SIZE};

use crate::tasks;
use crate::utils::colors::Color;
use crate::peripherials::display::{HEIGHT, WIDTH};

// Framebuffer(BUFFER_SIZE)
// = Transfer(MAX_TRANSFER) * TRANSFERS
// = DmaDescriptor(CHUNK_SIZE) * CHUNKS * TRANSFERS

pub const BUFFER_SIZE: usize = WIDTH as usize * HEIGHT as usize * 2;
pub const MAX_TRANSFER: usize = 32736;
pub const TRANSFERS: usize = BUFFER_SIZE.div_ceil(MAX_TRANSFER);
pub const CHUNKS: usize = MAX_TRANSFER.div_ceil(CHUNK_SIZE);

#[must_use]
pub struct Framebuffer {
	descs: &'static mut [[DmaDescriptor; CHUNKS]; TRANSFERS],
	buffer: &'static mut [u8; BUFFER_SIZE],
}

impl Framebuffer {
	pub fn new(descs: &'static mut [[DmaDescriptor; CHUNKS]; TRANSFERS],
	           buffer: &'static mut [u8; BUFFER_SIZE])
	           -> Self {
		for (buf, descs) in buffer.chunks_mut(CHUNK_SIZE * CHUNKS).zip(descs.iter_mut()) {
			let mut last_id = 0;
			for (id, (buf, desc)) in buf.chunks_mut(CHUNK_SIZE).zip(descs.iter_mut()).enumerate() {
				desc.buffer = buf.as_mut_ptr();
				desc.flags.set_length(buf.len() as u16);
				desc.flags.set_size(buf.len() as u16);
				last_id = id;
			}
			
			for id in 0 ..= last_id {
				descs[id].next = if id == last_id { core::ptr::null_mut() } else { &mut descs[id + 1] };
				descs[id].reset_for_tx(descs[id].next.is_null());
			}
		}
		
		Self { descs, buffer }
	}
	
	pub fn fill(&mut self, color: Color) {
		self.buffer
			.as_mut_slice()
			.array_chunks_mut()
			.for_each(|chunk| *chunk = color.as_u16().to_be_bytes());
	}
	
	pub async fn show(self) {
		tasks::display::FRAMES_READY.send(self).await;
	}
	
	pub fn transfers(&mut self) -> impl Iterator<Item = FramebufferTransfer<'_>> {
		self.descs
			.iter_mut()
			.enumerate()
			.map(|(seq, descs)| FramebufferTransfer {
				descs,
				len: usize::min(CHUNK_SIZE * CHUNKS, BUFFER_SIZE.saturating_sub(seq * CHUNK_SIZE * CHUNKS)),
			})
	}
}

impl Deref for Framebuffer {
	type Target = [u8];
	
	fn deref(&self) -> &Self::Target {
		self.buffer
	}
}

impl DerefMut for Framebuffer {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.buffer
	}
}

pub struct FramebufferTransfer<'a> {
	descs: &'a mut [DmaDescriptor; CHUNKS],
	len: usize,
}

impl<'a> FramebufferTransfer<'a> {
	pub fn len(&self) -> usize {
		self.len
	}
}

unsafe impl<'a> DmaTxBuffer for FramebufferTransfer<'a> {
	type View = Self;
	
	fn prepare(&mut self) -> Preparation {
		Preparation {
			start: &mut self.descs[0],
			direction: TransferDirection::Out,
			accesses_psram: false,
			burst_transfer: Default::default(),
			check_owner: None,
			auto_write_back: false,
		}
	}
	
	fn into_view(self) -> Self::View {
		self
	}
	
	fn from_view(view: Self::View) -> Self {
		view
	}
}

#[non_exhaustive]
#[must_use]
pub struct FramebufferSource;

impl FramebufferSource {
	pub async fn next_frame(&self) -> Framebuffer {
		tasks::display::FRAMES_EMPTY.receive().await
	}
}

macro_rules! static_framebuffer {
    () => {{
	    use esp_hal::dma::DmaDescriptor;
	    use $crate::utils::framebuffer::{CHUNKS, TRANSFERS, BUFFER_SIZE, Framebuffer};
	    
	    static mut DESCS: [[DmaDescriptor; CHUNKS]; TRANSFERS] = [[DmaDescriptor::EMPTY; CHUNKS]; TRANSFERS];
	    static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
	    
	    #[allow(static_mut_refs)]
	    unsafe { Framebuffer::new(&mut DESCS, &mut BUFFER) }
    }};
}

pub(crate) use static_framebuffer;
