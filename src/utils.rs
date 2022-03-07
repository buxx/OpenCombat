use std::sync::atomic::AtomicUsize;

pub fn new_squad_uuid() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
