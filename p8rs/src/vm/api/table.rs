use std::mem;
use std::pin::Pin;
use gc_arena::Collect;
use p8rs_macros::api_callback;
use p8rs_piccolo::{Context, Table, Stack, String, Value, IntoValue, MetaMethod, CallbackReturn, async_sequence, meta_ops, SequenceReturn, StashedFunction, StashedTable, StashedValue, StashedError, Sequence, Execution, RuntimeRef, SequencePoll, Error, BoxSequence};
use p8rs_piccolo::async_callback::{AsyncSequence, Locals};
use p8rs_piccolo::meta_ops::{MetaCall, MetaResult};
use p8rs_piccolo::table::{NextValue, RawTable};
use crate::vm::api::base::rawequal;

pub fn install(ctx: Context) {
	ctx.set_global(b"count", count::callback(ctx));
	ctx.set_global(b"add", add::callback(ctx));
	ctx.set_global(b"del", del::callback(ctx));
	ctx.set_global(b"deli", deli::callback(ctx));
	ctx.set_global(b"pack", pack::callback(ctx));
	ctx.set_global(b"unpack", unpack::callback(ctx));
	ctx.set_global(b"pairs", pairs::callback(ctx));
	ctx.set_global(b"ipairs", ipairs::callback(ctx));
	ctx.set_global(b"next", next::callback(ctx));
	ctx.set_global(b"inext", inext::callback(ctx));
}

#[api_callback]
pub fn count<'gc>(ctx: Context<'gc>, table: Table<'gc>, value: Option<Value<'gc>>) -> CallbackReturn<'gc> {
	CallbackReturn::Sequence(async_sequence(&ctx, |locals, mut seq| {
		let table = locals.stash(&ctx, table);
		let value = value.map(|v| locals.stash(&ctx, v));
		
		async move {
			let len = len_async(&mut seq, &table.clone().into(), 0).await?;
			
			let ret = if let Some(value) = value {
				let mut count = 0;
				
				for i in 1..=len {
					let item = seq.enter(|ctx, locals, _, _| {
						let item = locals.fetch(&table).get_value(ctx, i.cast_signed());
						locals.stash(&ctx, item)
					});
					
					if eq_async(&mut seq, &value, &item, 0).await? {
						count += 1;
					}
				}
				
				count
			} else {
				len
			};
			
			seq.enter(|ctx, _, _, mut stack| stack.replace(ctx, ret.cast_signed()));
			
			Ok(SequenceReturn::Return)
		}
	}))
}

#[api_callback]
pub fn add<'gc>(ctx: Context<'gc>, mut stack: Stack<'gc, '_>, table: Table<'gc>, value: Option<Value<'gc>>, index: Value<'gc>) -> Result<CallbackReturn<'gc>, Value<'gc>> {
	let Some(value) = value else { return Ok(CallbackReturn::Return) };
	
	let index =
		if index.is_nil() {
			None
		} else if let Some(num) = index.to_number() {
			Some(num.to_integer())
		} else {
			return Err(String::from_static(&ctx, b"bad argument #0 to 'add' (position out of bounds)").into());
		};
	
	stack.replace(ctx, value); // Return added value
	
	let metatable = table.metatable();
	let use_fallback = metatable.is_some_and(|mt|
		!mt.get_value(ctx, MetaMethod::Len).is_nil()
		|| !mt.get_value(ctx, MetaMethod::Index).is_nil()
		|| !mt.get_value(ctx, MetaMethod::NewIndex).is_nil());
	
	let length =
		if !use_fallback { // Try the fast path
			match array_insert_shift(&mut table.into_inner().borrow_mut(&ctx).raw_table, index, value) {
				(RawArrayOpResult::Success(_), _) => return Ok(CallbackReturn::Return),
				(RawArrayOpResult::Failed, _) => return Err(String::from_static(&ctx, b"bad argument #0 to 'add' (position out of bounds)").into()),
				(RawArrayOpResult::Possible, len) => Some(len),
			}
		} else {
			None
		};
	
	// Fast path failed, fall back to direct indexing
	Ok(CallbackReturn::Sequence(async_sequence(&ctx, |locals, mut seq| {
		let table = locals.stash(&ctx, table);
		let value = locals.stash(&ctx, value);
		
		async move {
			let length = match length {
				Some(len) => len,
				None => len_async(&mut seq, &table.clone().into(), 0).await?,
			};
			
			let end_index = length.wrapping_add(1);
			let index = index.map_or(end_index, i16::cast_unsigned);
			
			// Avoid evaluating (index + 1), which may overflow, if the
			// index is already at or past the end.
			if index < end_index {
				// Could make this more efficient by inlining the stack manipulation;
				// only pushing the table once.
				for i in (index + 1..=end_index).rev() {
					// table[i] = table[i - 1]
					let value = index_async(&mut seq, &table, (i - 1) as i16, 0).await?;
					index_set_async(&mut seq, &table, i as i16, value, 0).await?;
				}
			}
			
			// table[index] = value
			index_set_async(&mut seq, &table, index as i16, value, 0).await?;
			
			Ok(SequenceReturn::Return)
		}
	})))
}

#[api_callback]
pub fn del<'gc>(ctx: Context<'gc>, mut stack: Stack<'gc, '_>, table: Table<'gc>, value: Option<Value<'gc>>) -> Result<CallbackReturn<'gc>, Value<'gc>> {
	let Some(value) = value else {
		return Ok(CallbackReturn::Return)
	};
	
	let metatable = table.metatable();
	let val_metatable = if let Value::Table(table) = value { table.metatable() } else { None };
	let use_fallback =
		metatable.is_some_and(|mt|
			!mt.get_value(ctx, MetaMethod::Len).is_nil()
			|| !mt.get_value(ctx, MetaMethod::Index).is_nil()
			|| !mt.get_value(ctx, MetaMethod::NewIndex).is_nil())
		|| val_metatable.is_some_and(|mt| !mt.get_value(ctx, MetaMethod::Eq).is_nil());
	
	let length =
		if !use_fallback { // Try the fast path
			let raw_table = &mut table.into_inner().borrow_mut(&ctx).raw_table;
			let index = raw_table.array().iter().position(|&cur_value| rawequal(cur_value, value));
			
			if let Some(index) = index.and_then(|i| i.checked_add(1)) {
				match array_remove_shift(raw_table, Some(index as i16)) {
					(RawArrayOpResult::Success(val), _) => {
						stack.replace(ctx, val);
						return Ok(CallbackReturn::Return);
					}
					(RawArrayOpResult::Failed, _) => return Ok(CallbackReturn::Return),
					(RawArrayOpResult::Possible, len) => Some(len),
				}
			} else {
				None
			}
		} else {
			None
		};
	
	Ok(CallbackReturn::Sequence(async_sequence(&ctx, |locals, mut seq| {
		let table = locals.stash(&ctx, table);
		let value = locals.stash(&ctx, value);
		
		async move {
			let length = match length {
				Some(len) => len,
				None => len_async(&mut seq, &table.clone().into(), 0).await?,
			};
			
			for index in 1..=length {
				let cur_value = index_async(&mut seq, &table, index as i16, 0).await?;
				
				if eq_async(&mut seq, &value, &cur_value, 0).await? {
					seq.enter(|ctx, locals, _, mut stack| stack.replace(ctx, locals.fetch(&cur_value)));
					
					for i in index..length {
						// table[i] = table[i + 1]
						let value = index_async(&mut seq, &table, (i + 1) as i16, 0).await?;
						index_set_async(&mut seq, &table, i as i16, value, 0).await?;
					}
					
					// table[length] = nil
					index_set_async(&mut seq, &table, length as i16, StashedValue::Nil, 0).await?;
					
					break;
				}
			}
			
			Ok(SequenceReturn::Return)
		}
	})))
}

#[api_callback]
pub fn deli<'gc>(ctx: Context<'gc>, mut stack: Stack<'gc, '_>, table: Table<'gc>, index: Option<i16>) -> Result<CallbackReturn<'gc>, Value<'gc>> {
	let metatable = table.metatable();
	let use_fallback = metatable.is_some_and(|mt|
		!mt.get_value(ctx, MetaMethod::Len).is_nil()
		|| !mt.get_value(ctx, MetaMethod::Index).is_nil()
		|| !mt.get_value(ctx, MetaMethod::NewIndex).is_nil());
	
	let length =
		if !use_fallback { // Try the fast path
			match array_remove_shift(&mut table.into_inner().borrow_mut(&ctx).raw_table, index) {
				(RawArrayOpResult::Success(val), _) => {
					stack.replace(ctx, val);
					return Ok(CallbackReturn::Return);
				}
				(RawArrayOpResult::Failed, _) => return Ok(CallbackReturn::Return),
				(RawArrayOpResult::Possible, len) => Some(len),
			}
		} else {
			None
		};
	
	// Fast path failed, fall back to direct indexing
	Ok(CallbackReturn::Sequence(async_sequence(&ctx, |locals, mut seq| {
		let table = locals.stash(&ctx, table);
		async move {
			let length = match length {
				Some(len) => len,
				None => len_async(&mut seq, &table.clone().into(), 0).await?,
			};
			
			let index = index.map_or(length, i16::cast_unsigned);
			
			// either index and length are zero, or index == length + 1 (without overflow)
			if index.saturating_sub(1) == length {
				seq.enter(|_, _, _, mut stack| stack.push_back(Value::Nil));
			} else if index >= 1 && index <= length {
				// Get the value of the element to remove
				let ret = index_async(&mut seq, &table, index as i16, 0).await?;
				
				// Could make this more efficient by inlining the stack manipulation;
				// only pushing the table once.
				for i in index..length {
					// table[i] = table[i + 1]
					let value = index_async(&mut seq, &table, (i + 1) as i16, 0).await?;
					index_set_async(&mut seq, &table, i as i16, value, 1).await?;
				}
				
				// table[length] = nil
				index_set_async(&mut seq, &table, length as i16, StashedValue::Nil, 0).await?;
				
				// Put the removed value on the stack
				seq.enter(|ctx, locals, _, mut stack| stack.replace(ctx, locals.fetch(&ret)));
			}
			
			Ok(SequenceReturn::Return)
		}
	})))
}

#[api_callback]
pub fn pack<'gc>(ctx: Context<'gc>, mut stack: Stack<'gc, '_>) {
	let table = Table::new(&ctx);
	
	table.into_inner()
	     .borrow_mut(&ctx)
	     .raw_table
	     .grow_array(stack.len());
	
	for i in 0..stack.len() {
		table.set(ctx, (i as i16).wrapping_add(1), stack[i]).unwrap();
	}
	table.set(ctx, "n", stack.len() as i16).unwrap();
	
	stack.replace(ctx, table);
}

#[api_callback]
pub fn unpack<'gc>(ctx: Context<'gc>, mut stack: Stack<'gc, '_>, table: Table<'gc>, start: Option<i16>, end: Option<i16>) {
	let start = start.unwrap_or(1);
	let end = end.unwrap_or_else(|| table.length().cast_signed());
	
	if start <= end {
		stack.resize((end - start + 1) as usize);
		for i in start..=end {
			stack[(i - start) as usize] = table.get_value(ctx, i);
		}
	}
}

#[api_callback]
pub fn pairs<'gc>(ctx: Context<'gc>, mut stack: Stack<'gc, '_>, table: Value<'gc>) -> Result<CallbackReturn<'gc>, Error<'gc>> {
	let mt = match table {
		Value::Table(t) => t.metatable(),
		Value::UserData(u) => u.metatable(),
		_ => None,
	};
	
	if let Some(mt) = mt {
		/// Simply matches PUC-Rio behavior of returning the first 3 elements of the __pairs metacall
		#[derive(Collect)]
		#[collect(require_static)]
		struct PairsReturn;
		
		impl<'gc> Sequence<'gc> for PairsReturn {
			fn poll(
				self: Pin<&mut Self>,
				_ctx: Context<'gc>,
				_exec: Execution<'gc, '_>,
				mut stack: Stack<'gc, '_>,
				_rt: RuntimeRef<'_>,
			) -> Result<SequencePoll<'gc>, Error<'gc>> {
				if stack.len() > 3 {
					stack.drain(3..);
				}
				Ok(SequencePoll::Return)
			}
		}
		
		let pairs = mt.get_value(ctx, MetaMethod::Pairs);
		if !pairs.is_nil() {
			let function = meta_ops::call(ctx, pairs)?;
			stack.replace(ctx, (table, Value::Nil));
			return Ok(CallbackReturn::Call {
				function,
				then: Some(BoxSequence::new(&ctx, PairsReturn)),
			});
		}
	}
	
	stack.replace(ctx, (next::callback(ctx), table));
	
	Ok(CallbackReturn::Return)
}

#[api_callback]
pub fn ipairs<'gc>(ctx: Context<'gc>, mut stack: Stack<'gc, '_>) {
	stack.into_front(ctx, inext::callback(ctx));
}

#[api_callback]
pub fn next<'gc>(ctx: Context<'gc>, table: Table<'gc>, index: Value<'gc>) -> Result<(Value<'gc>, Option<Value<'gc>>), Value<'gc>> {
	match table.next(index) {
		NextValue::Found { key, value } => Ok((key, Some(value))),
		NextValue::Last => Ok((Value::Nil, None)),
		NextValue::NotFound => Err(String::from_static(&ctx, "invalid table key").into()),
	}
}

#[api_callback]
pub fn inext<'gc>(ctx: Context<'gc>, mut stack: Stack<'gc, '_>, table: Value<'gc>, index: Option<i16>) -> Result<CallbackReturn<'gc>, Error<'gc>> {
	let next_index = index.unwrap_or(0).wrapping_add(1);
	match meta_ops::index(ctx, table, next_index.into())? {
		MetaResult::Value(v) => {
			if !v.is_nil() {
				stack.extend([next_index.into(), v]);
			}
			Ok(CallbackReturn::Return)
		}
		MetaResult::Call(call) => {
			#[derive(Collect)]
			#[collect(require_static)]
			struct INext(i16);
			
			impl<'gc> Sequence<'gc> for INext {
				fn poll(
					self: Pin<&mut Self>,
					_ctx: Context<'gc>,
					_exec: Execution<'gc, '_>,
					mut stack: Stack<'gc, '_>,
					_rt: RuntimeRef,
				) -> Result<SequencePoll<'gc>, Error<'gc>> {
					if !stack.get(0).is_nil() {
						stack.push_front(self.0.into());
					}
					Ok(SequencePoll::Return)
				}
			}
			
			stack.extend(call.args);
			Ok(CallbackReturn::Call {
				function: call.function,
				then: Some(BoxSequence::new(&ctx, INext(next_index))),
			})
		}
	}
}

#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum RawArrayOpResult<T> {
	Success(T),
	Possible,
	Failed,
}

// Try to efficiently remove a key from the array part of the table.  (`key` is one-indexed; if it
// is None, the length of the array is used instead.)
//
// If successful, returns the removed value; otherwise, indicates whether the operation is possible
// to implement with a fallback, or is impossible due to an out-of-range index.
//
// Additionally, always returns the computed length of the array from before the operation.
fn array_remove_shift<'gc>(
	table: &mut RawTable<'gc>,
	key: Option<i16>,
) -> (RawArrayOpResult<Value<'gc>>, u16) {
	fn inner<'gc>(
		table: &mut RawTable<'gc>,
		length: u16,
		key: Option<i16>,
	) -> RawArrayOpResult<Value<'gc>> {
		let index;
		if let Some(k) = key {
			if k == 0 && length == 0 || k as u16 == length + 1 {
				return RawArrayOpResult::Success(Value::Nil);
			} else if k >= 1 && k as u16 <= length {
				index = k as usize - 1;
			} else {
				return RawArrayOpResult::Failed;
			}
		} else {
			if length == 0 {
				return RawArrayOpResult::Success(Value::Nil);
			} else {
				index = length as usize - 1;
			}
		}
		
		let length = length as usize;
		let array = table.array_mut();
		if length > array.len() {
			return RawArrayOpResult::Possible;
		}
		
		let value = mem::replace(&mut array[index], Value::Nil);
		if length - index > 1 {
			array[index..length].rotate_left(1);
		}
		RawArrayOpResult::Success(value)
	}
	
	let length = table.length();
	(inner(table, length, key), length)
}

// Try to efficiently insert a key and value into the array part of the table.  (`key` is
// one-indexed; if it is `None`, the length of the array is used instead.)
//
// The returned [`RawArrayOpResult`] indicates whether the operation was successful, or if it
// failed, whether the operation is possible to implement with a fallback, or is impossible due to
// an out-of-range index.
//
// Additionally, always returns the computed length of the array from before the operation.
fn array_insert_shift<'gc>(
	table: &mut RawTable<'gc>,
	key: Option<i16>,
	value: Value<'gc>,
) -> (RawArrayOpResult<()>, u16) {
	fn inner<'gc>(
		table: &mut RawTable<'gc>,
		length: u16,
		key: Option<i16>,
		value: Value<'gc>,
	) -> RawArrayOpResult<()> {
		let index;
		if let Some(k) = key {
			if k >= 1 && k as u16 <= length + 1 {
				index = k as usize - 1;
			} else {
				return RawArrayOpResult::Failed;
			}
		} else {
			index = length as usize;
		}
		
		let length = length as usize;
		let array_len = table.array().len();
		if length > array_len {
			return RawArrayOpResult::Possible;
		}
		
		assert!(index <= length);
		
		if length == array_len {
			// If the array is full, grow it.
			table.grow_array(1);
		}
		
		let array = table.array_mut();
		// We know here that length < array.len(), so we shift each
		// element to the right by one.
		// array[length] == nil, which gets rotated back to array[index];
		// we replace it with the value to insert.
		array[index..=length].rotate_right(1);
		array[index] = value;
		RawArrayOpResult::Success(())
	}
	
	let length = table.length();
	(inner(table, length, key, value), length)
}

enum StashedMetaResult {
	Value(StashedValue),
	Call(StashedFunction),
}

impl StashedMetaResult {
	fn new<'gc, const N: usize>(ctx: Context<'gc>, stack: Stack<'gc, '_>, locals: Locals<'gc, '_>, result: MetaResult<'gc, N>) -> StashedMetaResult {
		match result {
			MetaResult::Value(v) => StashedMetaResult::Value(locals.stash(&ctx, v)),
			MetaResult::Call(call) => StashedMetaResult::from_call(ctx, stack, locals, call),
		}
	}
	
	fn from_call<'gc, const N: usize>(ctx: Context<'gc>, mut stack: Stack<'gc, '_>, locals: Locals<'gc, '_>, call: MetaCall<'gc, N>) -> StashedMetaResult {
		stack.extend(call.args);
		StashedMetaResult::Call(locals.stash(&ctx, call.function))
	}
	
	async fn eval(self, seq: &mut AsyncSequence, bottom: usize) -> Result<StashedValue, StashedError> {
		match self {
			StashedMetaResult::Value(value) => Ok(value),
			StashedMetaResult::Call(call) => {
				seq.call(&call, bottom).await?;
				seq.try_enter(|ctx, locals, _, mut stack| {
					let value = stack.sub_stack(bottom)
					                 .consume::<Value>(ctx)?;
					
					Ok(locals.stash(&ctx, value))
				})
			}
		}
	}
}

async fn index_set_async(seq: &mut AsyncSequence, table: &StashedTable, key: i16, value: StashedValue, bottom: usize) -> Result<(), StashedError> {
	let result = seq.try_enter(|ctx, locals, _, stack| {
		let call = meta_ops::new_index(
			ctx,
			locals.fetch(table).into(),
			key.into(),
			locals.fetch(&value)
		)?;
		
		Ok(match call {
			None => StashedMetaResult::Value(StashedValue::Nil),
			Some(call) => StashedMetaResult::from_call(ctx, stack, locals, call),
		})
	})?;
	
	result.eval(seq, bottom).await?;
	
	Ok(())
}

async fn index_async(seq: &mut AsyncSequence, table: &StashedTable, key: i16, bottom: usize) -> Result<StashedValue, StashedError> {
	let result = seq.try_enter(|ctx, locals, _, stack| {
		let result = meta_ops::index(ctx, locals.fetch(table).into(), key.into())?;
		
		Ok(StashedMetaResult::new(ctx, stack, locals, result))
	})?;
	
	result.eval(seq, bottom).await
}

async fn len_async(seq: &mut AsyncSequence, value: &StashedValue, bottom: usize) -> Result<u16, StashedError> {
	let result = seq.try_enter(|ctx, locals, _, stack| {
		let result = meta_ops::len(ctx, locals.fetch(value))?;
		
		Ok(StashedMetaResult::new(ctx, stack, locals, result))
	})?;
	
	let result = result.eval(seq, bottom).await?;
	
	match result {
		StashedValue::Number(num) => Ok(num.to_integer() as u16),
		_ => seq.try_enter(|ctx, _, _, _| Err(String::from_static(&ctx, b"__len did not returned numeric value").into_value(ctx))?),
	}
}

async fn eq_async(seq: &mut AsyncSequence, lhs: &StashedValue, rhs: &StashedValue, bottom: usize) -> Result<bool, StashedError> {
	let result = seq.try_enter(|ctx, locals, _, stack| {
		let result = meta_ops::equal(ctx, locals.fetch(lhs), locals.fetch(rhs))?;
		
		Ok(StashedMetaResult::new(ctx, stack, locals, result))
	})?;
	
	let result = result.eval(seq, bottom).await?;
	
	match result {
		StashedValue::Boolean(res) => Ok(res),
		_ => seq.try_enter(|ctx, _, _, _| Err(String::from_static(&ctx, b"__eq did not returned a boolean").into_value(ctx))?),
	}
}
