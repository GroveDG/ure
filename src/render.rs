use std::{
    task::{Context, RawWaker, RawWakerVTable, Waker}, time::Duration
};

use spin_sleep::SpinSleeper;
use wgpu::{
    wgt::DeviceDescriptor, Adapter, Device, Instance, InstanceDescriptor, Queue, RequestAdapterOptions
};

use crate::game::tf::{Matrix2D, Precision};

pub mod _2d;

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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
#[allow(dead_code)]
impl Color {
    pub const WHITE: Self = Color {
        r: 1.,
        g: 1.,
        b: 1.,
        a: 1.,
    };
    pub const BLUE: Self = Color {
        r: 0.,
        g: 0.,
        b: 1.,
        a: 1.,
    };
}
impl From<Color> for wgpu::Color {
    fn from(value: Color) -> Self {
        Self {
            r: value.r as f64,
            g: value.g as f64,
            b: value.b as f64,
            a: value.a as f64,
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
