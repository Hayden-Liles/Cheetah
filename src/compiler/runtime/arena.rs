// arena.rs - Bump-pointer arena allocator for BoxedAny values
//
// This file implements a thread-local arena allocator for BoxedAny values.
// The arena allocates fixed-size chunks of memory and provides fast, lock-free
// allocation for BoxedAny headers. Large allocations (strings, lists, etc.)
// still use malloc, with only the fixed-size BoxedAny header living in the arena.

use std::cell::RefCell;
use super::boxed_any::BoxedAny;

/// Size of each memory slab (1 MiB)
const CHUNK: usize = 1024 * 1024;

/// Size of a BoxedAny header
const HEADER: usize = std::mem::size_of::<BoxedAny>();

/// Bit 31 in the `tag` field means "owned by arena – don't free"
pub const ARENA_FLAG: i32 = 1 << 31;

/// A memory slab for the arena
struct Slab {
    /// Current allocation pointer
    cur: *mut u8,
    /// End of the slab
    end: *mut u8,
    /// The actual memory buffer
    /// This field is never directly read, but we need to keep the Box alive
    /// to prevent the memory from being freed
    #[allow(dead_code)]
    buf: Box<[u8]>,
}

impl Slab {
    /// Create a new slab
    fn new() -> Self {
        let mut v = vec![0u8; CHUNK].into_boxed_slice();
        let start = v.as_mut_ptr();
        Self {
            cur: start,
            end: unsafe { start.add(CHUNK) },
            buf: v,
        }
    }

    /// Allocate a BoxedAny from the slab
    #[inline]
    fn alloc(&mut self) -> *mut BoxedAny {
        if (self.end as usize - self.cur as usize) < HEADER {
            *self = Slab::new(); // start new slab
        }
        let p = self.cur as *mut BoxedAny;
        self.cur = unsafe { self.cur.add(HEADER) };
        p
    }
}

thread_local! {
    // Thread-local arena
    static ARENA: RefCell<Slab> = RefCell::new(Slab::new());
}

/// Allocate a BoxedAny from the arena
#[inline]
pub fn alloc() -> *mut BoxedAny {
    ARENA.with(|a| a.borrow_mut().alloc())
}

/// Reset the arena (free all allocations)
#[no_mangle]
pub extern "C" fn arena_reset() {
    ARENA.with(|a| *a.borrow_mut() = Slab::new());
}

/// Register arena functions for JIT execution
pub fn register_arena_runtime_functions(
    engine: &inkwell::execution_engine::ExecutionEngine<'_>,
    module: &inkwell::module::Module<'_>,
) -> Result<(), String> {
    if let Some(f) = module.get_function("arena_reset") {
        engine.add_global_mapping(&f, arena_reset as usize);
    }

    Ok(())
}

/// Register arena functions in the module
pub fn register_arena_functions<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &mut inkwell::module::Module<'ctx>,
) {
    let void_type = context.void_type();

    // Register arena_reset function
    let arena_reset_type = void_type.fn_type(&[], false);
    module.add_function("arena_reset", arena_reset_type, None);
}
