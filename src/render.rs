use std::{
    task::{Context, RawWaker, RawWakerVTable, Waker},
    time::Duration,
};

use spin_sleep::SpinSleeper;
use wgpu::{
    Device, Instance, InstanceDescriptor, Queue, RequestAdapterOptions, wgt::DeviceDescriptor,
};

use crate::game::tf::{Matrix2D, Precision};

pub mod _2d;
pub mod color;

// [NOTE] https://www.reddit.com/r/opengl/comments/v5w80e/instancing_how_to_account_for_new_data_after/

pub struct Matrix2DGPU {
    pub inner: [Precision; 12],
}
impl From<Matrix2D> for Matrix2DGPU {
    fn from(value: Matrix2D) -> Self {
        let array = value.to_cols_array();
        Self {
            inner: [
                array[0], array[1], array[2], 0.0, array[3], array[4], array[5], 0.0, array[6],
                array[7], array[8], 0.0,
            ],
        }
    }
}

pub async fn new_gpu() -> (Instance, Device, Queue) {
    let instance = Instance::new(&InstanceDescriptor::from_env_or_default());
    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(&DeviceDescriptor::default())
        .await
        .unwrap();
    (instance, device, queue)
}

// https://users.rust-lang.org/t/simplest-possible-block-on/48364
unsafe fn rwclone(_p: *const ()) -> RawWaker {
    make_raw_waker()
}
unsafe fn rwwake(_p: *const ()) {}
unsafe fn rwwakebyref(_p: *const ()) {}
unsafe fn rwdrop(_p: *const ()) {}

static VTABLE: RawWakerVTable = RawWakerVTable::new(rwclone, rwwake, rwwakebyref, rwdrop);

fn make_raw_waker() -> RawWaker {
    static DATA: () = ();
    RawWaker::new(&DATA, &VTABLE)
}

pub trait BlockingFuture: Future + Sized {
    fn block(self) -> <Self as Future>::Output {
        let sleeper = SpinSleeper::default();
        let mut boxed = Box::pin(self);
        let waker = unsafe { Waker::from_raw(make_raw_waker()) };
        let mut ctx = Context::from_waker(&waker);
        loop {
            match boxed.as_mut().poll(&mut ctx) {
                std::task::Poll::Ready(x) => {
                    return x;
                }
                std::task::Poll::Pending => {
                    sleeper.sleep(Duration::from_millis(10));
                }
            }
        }
    }
}

impl<F: Future + Sized> BlockingFuture for F {}
