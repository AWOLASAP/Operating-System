use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::{stream::Stream,task::AtomicWaker,stream::StreamExt};
use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
use crate::println;
use crate::keyboard_routing::KEYBOARD_ROUTER;
use x86_64::instructions::interrupts;

static WAKER: AtomicWaker = AtomicWaker::new();
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub async fn print_keypresses() {
    // creates new queue for keys
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1,
        HandleControl::Ignore);

    // waits for next keypress then hands it over to the KEYBOARD_ROUTER to handle
    while let Some(scancode) = scancodes.next().await {
        interrupts::without_interrupts(|| {
            KEYBOARD_ROUTER.lock().handle_scancode(scancode,&mut keyboard);
        });
    }
}

// adds new keypresses to the queue to be dealt with
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        }else{
            // after pusing the scancode to queue the waker will notify the executor
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    _private: (),
}

// initializes scancode stream and returns an error if it's tried again
impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    // polls the next item in the queue
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // immediately tries to poll from the queue
        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        // if the item from queue is pending it gets a waker
        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}
