// buffer.rs - Combined circular & buffered output

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread_local;
use ryu;
use itoa;
use std::sync::atomic::AtomicBool;

// Shared stats
static OPERATIONS: AtomicUsize = AtomicUsize::new(0);
static BYTES_WRITTEN: AtomicUsize = AtomicUsize::new(0);
static BYTES_SAVED: AtomicUsize = AtomicUsize::new(0);
static FORCE_DIRECT: AtomicBool = AtomicBool::new(false);

// Circular buffer
const CIRC_CAP: usize = 8192;
const FLUSH_TH: usize = 4096;
const MAX_INTERNED: usize = 64;

struct CircularBuffer { buf: Vec<u8>, read: usize, write: usize, size: usize, cap: usize }
impl CircularBuffer {
    fn new(cap: usize) -> Self { CircularBuffer { buf: vec![0;cap], read:0, write:0, size:0, cap } }
    fn write_byte(&mut self, b: u8) -> io::Result<()> { /* flush if full, then write */
        if self.size==self.cap { self.flush()? }
        self.buf[self.write]=b; self.write=(self.write+1)%self.cap; self.size+=1; Ok(())
    }
    fn write(&mut self, s: &[u8]) -> io::Result<()> { if s.len()>self.cap { self.flush()?; io::stdout().write_all(s)?; return Ok(()) }
        if s.len()>self.cap-self.size { self.flush()? }
        for &b in s { self.write_byte(b)? }
        if self.size>FLUSH_TH { self.flush()? }
        Ok(())
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.size==0 { return Ok(()) }
        if self.read<self.write { io::stdout().write_all(&self.buf[self.read..self.write])?; }
        else { io::stdout().write_all(&self.buf[self.read..self.cap])?; io::stdout().write_all(&self.buf[0..self.write])?; }
        io::stdout().flush()?;
        self.read=0; self.write=0; self.size=0; Ok(())
    }
}

thread_local! {
    static CIRC: RefCell<CircularBuffer> = RefCell::new(CircularBuffer::new(CIRC_CAP));
    static CACHE: RefCell<HashMap<u64,Vec<u8>>> = RefCell::new(HashMap::with_capacity(MAX_INTERNED));
}

/// Initialize buffer systems
pub fn init() {
    OPERATIONS.store(0, Ordering::Relaxed);
    BYTES_WRITTEN.store(0, Ordering::Relaxed);
    BYTES_SAVED.store(0, Ordering::Relaxed);
    FORCE_DIRECT.store(false, Ordering::Relaxed);
    CIRC.with(|c| c.borrow_mut().flush().ok());
    CACHE.with(|c| c.borrow_mut().clear());
}

/// Write raw bytes
fn write_bytes(b: &[u8]) {
    OPERATIONS.fetch_add(1,Ordering::Relaxed);
    if FORCE_DIRECT.load(Ordering::Relaxed) {
        let _=io::stdout().write_all(b);
        return;
    }
    if let Err(_) = CIRC.with(|c| c.borrow_mut().write(b)) {
        let _=io::stdout().write_all(b);
    }
}

/// Flush
pub fn flush() { let _=CIRC.with(|c| c.borrow_mut().flush()); }

/// Write string
pub fn write_str(s: &str) { write_bytes(s.as_bytes()); }
/// Write newline
pub fn write_newline() { write_bytes(b"\n"); flush(); }
/// Write int
pub fn write_int(v: i64) {
    OPERATIONS.fetch_add(1,Ordering::Relaxed);
    if FORCE_DIRECT.load(Ordering::Relaxed) { let _=write!(io::stdout(),"{}",v); return; }
    static mut ITOA_BUF: [Option<itoa::Buffer>;10] = [None,None,None,None,None,None,None,None,None,None];
    let idx = 0;
    let buf = unsafe { ITOA_BUF[idx].get_or_insert_with(|| itoa::Buffer::new()) };
    write_bytes(buf.format(v).as_bytes());
}

/// Write float
pub fn write_float(v: f64) { OPERATIONS.fetch_add(1,Ordering::Relaxed);
    if FORCE_DIRECT.load(Ordering::Relaxed) { let _=write!(io::stdout(),"{}",v); return; }
    let mut b=ryu::Buffer::new(); write_bytes(b.format(v).as_bytes());
}

/// Write bool
pub fn write_bool(v: bool) { write_str(if v {"True"} else {"False"}); }

/// Clean up and report
pub fn cleanup() {
    flush();
    let ops=OPERATIONS.load(Ordering::Relaxed);
    let _written=BYTES_WRITTEN.load(Ordering::Relaxed);
    let saved=BYTES_SAVED.load(Ordering::Relaxed);
    if ops>0 { eprintln!("[BUFFER] ops={}, saved={}", ops, saved); }
}
