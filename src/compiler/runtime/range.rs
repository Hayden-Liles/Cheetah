// range.rs - Combined range operations and iterator

use inkwell::context::Context;
use inkwell::module::Module;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::cell::RefCell;
use std::thread_local;


// Constants and globals
static RANGE_OP_COUNT: AtomicUsize = AtomicUsize::new(0);
static RANGE_DEBUG: AtomicBool = AtomicBool::new(false);
const VERY_LARGE_THRESHOLD: i64 = 10_000_000;
const RANGE_SIZE_LIMIT: i64 = 1_000_000_000_000;
const ITERATOR_POOL_SIZE: usize = 8;
const CHUNK_MEDIUM: i64 = 10_000;
const CHUNK_LARGE: i64 = 100_000;
const SIZE_MEDIUM: i64 = 1_000_000;
const SIZE_LARGE: i64 = 10_000_000;

//--------- Range size calculation & tracking ---------

fn track_range(start: i64, stop: i64, step: i64) {
    let c = RANGE_OP_COUNT.fetch_add(1, Ordering::Relaxed);
    if RANGE_DEBUG.load(Ordering::Relaxed) && c % 1000 == 0 {
        eprintln!("[RANGE DEBUG] count={} {:?}..{:?} step={}", c, start, stop, step);
    }
    if (stop - start).abs() > VERY_LARGE_THRESHOLD {
        eprintln!("[RANGE WARNING] large range {:?}..{:?}", start, stop);
    }
}

fn calculate_size(start: i64, stop: i64, step: i64) -> i64 {
    let mut size = if step == 0 {
        0
    } else if step == 1 && start < stop {
        stop - start
    } else if (step > 0 && start < stop) || (step < 0 && start > stop) {
        (stop - start) / step + ((stop - start) % step != 0) as i64
    } else { 0 };
    if size > RANGE_SIZE_LIMIT {
        eprintln!("[RANGE WARNING] size {} exceeds limit", size);
        size = RANGE_SIZE_LIMIT;
    }
    size
}

// C API: range_1, range_2, range_3, range_cleanup

#[no_mangle]
pub extern "C" fn range_1(stop: i64) -> i64 {
    let s = stop.min(RANGE_SIZE_LIMIT);
    track_range(0, s, 1);
    calculate_size(0, s, 1)
}

#[no_mangle]
pub extern "C" fn range_2(start: i64, stop: i64) -> i64 {
    let size = (stop - start).max(0);
    let (s0, s1) = if size > RANGE_SIZE_LIMIT {(start, start + RANGE_SIZE_LIMIT)} else {(start, stop)};
    track_range(s0, s1, 1);
    calculate_size(s0, s1, 1)
}

#[no_mangle]
pub extern "C" fn range_3(start: i64, stop: i64, step: i64) -> i64 {
    let st = if step == 0 {1} else {step};
    let size = calculate_size(start, stop, st);
    let (s0, s1) = if size > RANGE_SIZE_LIMIT {
        let end = start + RANGE_SIZE_LIMIT * st;
        (start, end)
    } else {(start, stop)};
    track_range(s0, s1, st);
    calculate_size(s0, s1, st)
}

#[no_mangle]
pub extern "C" fn range_cleanup() { RANGE_OP_COUNT.store(0, Ordering::Relaxed); }

//--------- Iterator pool & streaming ---------

#[derive(Clone)]
struct ChunkState { current_start: i64, chunk_end: i64, chunk_size: i64, overall_end: i64 }

#[repr(C)]
pub struct RangeIterator { start: i64, stop: i64, step: i64, current: i64, size: i64, active: bool, chunk: Option<ChunkState> }

impl RangeIterator {
    fn new(start: i64, stop: i64, step: i64) -> Self {
        let sz = calculate_size(start, stop, step);
        let chunk = if sz > SIZE_MEDIUM {
            let cs = if sz > SIZE_LARGE {CHUNK_LARGE} else {CHUNK_MEDIUM};
            Some(ChunkState { current_start: start, chunk_end: (start + cs*step).min(stop), chunk_size: cs, overall_end: stop })
        } else { None };
        RangeIterator { start, stop, step, current: start, size: sz, active: true, chunk }
    }
    fn get(start: i64, stop: i64, step: i64) -> Self {
        ITER_POOL.with(|p| p.borrow_mut().pop()).unwrap_or_else(|| RangeIterator::new(start, stop, step)).reset(start, stop, step)
    }
    fn reset(&mut self, start: i64, stop: i64, step: i64) -> Self {
        let new_iter = RangeIterator::new(start, stop, step);
        *self = new_iter;
        RangeIterator::new(start, stop, step)
    }
    fn next(&mut self) -> Option<i64> {
        if !self.active { return None; }
        if let Some(ref mut ch) = self.chunk {
            let done = if self.step>0 { self.current>=ch.chunk_end } else { self.current<=ch.chunk_end };
            if done {
                ch.current_start = ch.chunk_end;
                ch.chunk_end = (ch.current_start + ch.chunk_size*self.step).min(ch.overall_end);
                if if self.step>0 {ch.current_start>=ch.overall_end} else {ch.current_start<=ch.overall_end} { return None; }
                self.current = ch.current_start;
            }
        } else {
            if if self.step>0 {self.current>=self.stop} else {self.current<=self.stop} { return None; }
        }
        let val = self.current;
        self.current += self.step;
        Some(val)
    }
    fn return_to_pool(self) {
        let mut s = self; s.active=false;
        ITER_POOL.with(|p| if p.borrow().len()<ITERATOR_POOL_SIZE { p.borrow_mut().push(s) });
    }
    fn size(&self) -> i64 { self.size }
}

impl Drop for RangeIterator { fn drop(&mut self) { self.active=false; }}

thread_local! { static ITER_POOL: RefCell<Vec<RangeIterator>> = RefCell::new(Vec::with_capacity(ITERATOR_POOL_SIZE)); }

#[no_mangle]
pub extern "C" fn range_iterator_1(stop: i64) -> *mut RangeIterator { Box::into_raw(Box::new(RangeIterator::get(0, stop, 1))) }
#[no_mangle]
pub extern "C" fn range_iterator_2(start: i64, stop: i64) -> *mut RangeIterator { Box::into_raw(Box::new(RangeIterator::get(start, stop, 1))) }
#[no_mangle]
pub extern "C" fn range_iterator_3(start: i64, stop: i64, step: i64) -> *mut RangeIterator { Box::into_raw(Box::new(RangeIterator::get(start, stop, step.max(1)))) }
#[no_mangle]
pub extern "C" fn range_iterator_next(it: *mut RangeIterator, out: *mut i64) -> bool {
    if it.is_null()||out.is_null() { return false; }
    unsafe { if let Some(v)=(&mut *it).next() { *out=v; true } else { false } }
}
#[no_mangle]
pub extern "C" fn range_iterator_size(it: *mut RangeIterator) -> i64 { if it.is_null() {0} else { unsafe{(&*it).size()} }}
#[no_mangle]
pub extern "C" fn range_iterator_free(it: *mut RangeIterator) { if !it.is_null() { unsafe{Box::from_raw(it)}.return_to_pool(); }}

// Initialization and cleanup

pub fn init() {
    RANGE_OP_COUNT.store(0, Ordering::Relaxed);
    RANGE_DEBUG.store(false, Ordering::Relaxed);
}

pub fn cleanup() {
    let ops = RANGE_OP_COUNT.load(Ordering::Relaxed);
    if ops > 0 && RANGE_DEBUG.load(Ordering::Relaxed) {
        eprintln!("[RANGE] ops={}", ops);
    }
}

// Registration

pub fn register_range_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    use inkwell::AddressSpace;
    module.add_function("range_1", context.i64_type().fn_type(&[context.i64_type().into()], false), None);
    module.add_function("range_2", context.i64_type().fn_type(&[context.i64_type().into(), context.i64_type().into()], false), None);
    module.add_function("range_3", context.i64_type().fn_type(&[context.i64_type().into(), context.i64_type().into(), context.i64_type().into()], false), None);
    module.add_function("range_cleanup", context.void_type().fn_type(&[], false), None);
    module.add_function("range_iterator_1", context.ptr_type(AddressSpace::default()).fn_type(&[context.i64_type().into()], false), None);
    module.add_function("range_iterator_2", context.ptr_type(AddressSpace::default()).fn_type(&[context.i64_type().into(), context.i64_type().into()], false), None);
    module.add_function("range_iterator_3", context.ptr_type(AddressSpace::default()).fn_type(&[context.i64_type().into(), context.i64_type().into(), context.i64_type().into()], false), None);
    module.add_function("range_iterator_next", context.bool_type().fn_type(&[context.ptr_type(AddressSpace::default()).into(), context.ptr_type(AddressSpace::default()).into()], false), None);
    module.add_function("range_iterator_size", context.i64_type().fn_type(&[context.ptr_type(AddressSpace::default()).into()], false), None);
    module.add_function("range_iterator_free", context.void_type().fn_type(&[context.ptr_type(AddressSpace::default()).into()], false), None);
}