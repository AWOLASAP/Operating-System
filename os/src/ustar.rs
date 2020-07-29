use lazy_static::lazy_static;
use spin::Mutex;

struct Ustar

lazy_static! {
    pub static ref USTARFS: Mutex<ModeController> = {
        Mutex::new(ModeController::new())
    };
}