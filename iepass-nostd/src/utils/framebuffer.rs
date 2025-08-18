use core::cmp::Ordering;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::AtomicUsize;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::priority_channel::{Min, PriorityChannel};
use embassy_sync::watch;
use embassy_sync::watch::Watch;
use esp_hal::dma::{DmaDescriptor, DmaTxBuffer, Preparation, TransferDirection, CHUNK_SIZE};

use crate::tasks;
use crate::utils::colors::Color;
use crate::tasks::draw::DRAW_TASKS_MAX;
use crate::peripherials::display::{HEIGHT, WIDTH};

// Framebuffer(BUFFER_SIZE)
// = Transfer(MAX_TRANSFER) * TRANSFERS
// = DmaDescriptor(CHUNK_SIZE) * CHUNKS * TRANSFERS

pub const MAX_FRAMEBUFFERS: usize = 3;
pub const BUFFER_SIZE: usize = WIDTH as usize * HEIGHT as usize * 2;
pub const MAX_TRANSFER: usize = 32736;
pub const TRANSFERS: usize = BUFFER_SIZE.div_ceil(MAX_TRANSFER);
pub const CHUNKS: usize = MAX_TRANSFER.div_ceil(CHUNK_SIZE);

pub static FRAMEBUFFERS: AtomicUsize = AtomicUsize::new(0);

#[must_use]
pub struct Framebuffer {
	descs: &'static mut [[DmaDescriptor; CHUNKS]; TRANSFERS],
	buffer: &'static mut [u8; BUFFER_SIZE],
	pub seq: usize,
}

impl Framebuffer {
	pub fn new(descs: &'static mut [[DmaDescriptor; CHUNKS]; TRANSFERS],
	           buffer: &'static mut [u8; BUFFER_SIZE],
	           seq: usize)
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
		
		Self { descs, buffer, seq }
	}
	
	pub fn fill(&mut self, color: Color) {
		self.buffer
			.as_mut_slice()
			.array_chunks_mut()
			.for_each(|chunk| *chunk = color.as_u16().to_be_bytes());
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

impl PartialEq<Self> for Framebuffer {
	fn eq(&self, other: &Self) -> bool {
		self.seq.eq(&other.seq)
	}
}

impl Eq for Framebuffer {}

impl PartialOrd for Framebuffer {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.seq.partial_cmp(&other.seq)
	}
}

impl Ord for Framebuffer {
	fn cmp(&self, other: &Self) -> Ordering {
		self.seq.cmp(&other.seq)
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

pub struct FramebufferManager {
	empty: PriorityChannel<CriticalSectionRawMutex, Framebuffer, Min, MAX_FRAMEBUFFERS>,
	drawn: PriorityChannel<CriticalSectionRawMutex, Framebuffer, Min, MAX_FRAMEBUFFERS>,
	watch: Watch<CriticalSectionRawMutex, (), DRAW_TASKS_MAX>,
}

impl FramebufferManager {
	pub const fn new() -> Self {
		Self {
			empty: PriorityChannel::new(),
			drawn: PriorityChannel::new(),
			watch: Watch::new(),
		}
	}
	
	pub fn producer(&'static self) -> FramebufferProducer<'static> {
		FramebufferProducer::new(self, self.watch.receiver().expect("too many framebuffer producers created"))
	}
	
	pub async fn get_drawn(&self) -> Framebuffer {
		self.drawn.receive().await
	}
	
	pub async fn put_empty(&self, fb: Framebuffer) {
		self.empty.send(fb).await;
		self.watch.sender().send(());
	}
}

#[non_exhaustive]
#[must_use]
pub struct FramebufferProducer<'a> {
	manager: &'a FramebufferManager,
	receiver: watch::Receiver<'a, CriticalSectionRawMutex, (), DRAW_TASKS_MAX>,
}

impl<'a> FramebufferProducer<'a> {
	pub fn new(manager: &'a FramebufferManager, receiver: watch::Receiver<'a, CriticalSectionRawMutex, (), DRAW_TASKS_MAX>) -> Self {
		Self { manager, receiver }
	}
	
	pub async fn get_empty(&mut self) -> Framebuffer {
		self.receiver.changed().await;
		self.manager.empty.receive().await
	}
	
	pub async fn put_drawn(&self, fb: Framebuffer) {
		self.manager.drawn.send(fb).await;
	}
}

macro_rules! static_framebuffer {
    ($seq:ident) => {{
	    use esp_hal::dma::DmaDescriptor;
	    use core::sync::atomic::Ordering;
	    use $crate::utils::framebuffer::{MAX_FRAMEBUFFERS, FRAMEBUFFERS, CHUNKS, TRANSFERS, BUFFER_SIZE, Framebuffer};
	    
	    if FRAMEBUFFERS.fetch_add(1, Ordering::SeqCst) >= MAX_FRAMEBUFFERS {
		    panic!("Too many framebuffers created.");
	    }
	    
	    static mut DESCS: [[DmaDescriptor; CHUNKS]; TRANSFERS] = [[DmaDescriptor::EMPTY; CHUNKS]; TRANSFERS];
	    static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
	    
	    let seq = $seq;
	    $seq += 1;
	    
	    #[allow(static_mut_refs)]
	    unsafe { Framebuffer::new(&mut DESCS, &mut BUFFER, seq) }
    }};
}

pub(crate) use static_framebuffer;
