use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::{stream::{Stream, StreamExt}, task::AtomicWaker};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use crate::{println, print};

/// A queue containing keyboard scancodes received from the keyboard 
/// interrupt handler
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

/// A waker used to notify the 'ScancodeStream' when a new scancode
/// becomes available.
static WAKER: AtomicWaker = AtomicWaker::new();

/// An asynchronous stream of keyboard scancodes.
pub struct ScancodeStream {
    _private: (),
}

/// Called by the keyboard interrupt handler
/// 
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

impl ScancodeStream {
    /// Creates a new 'ScancodeStream'.
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    /// The type of value produced by this stream
    type Item = u8;

    /// Attempts to retrieve the next keyboard scancode
    /// 
    /// If a scancode is already availabeli n the queue, it is
    /// returned immediately
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.try_get().expect("not initialized");
        
        // First, check whether a scancode is already available
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        // No scancode is currently available
        WAKER.register(&cx.waker());

        match queue.pop() {
            // available
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            // Not available
            None => Poll::Pending,
        }
    }
}

/// Asynchronously prints keyboard input to the screen
pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(ScancodeSet1::new(),
        layouts::Us104Key, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}