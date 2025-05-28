use wgpu::{
    util::{BufferInitDescriptor, DeviceExt}, wgt::DeviceDescriptor, Adapter, Buffer, Device, Instance, InstanceDescriptor, Queue, RequestAdapterOptions
};



// [NOTE] https://www.reddit.com/r/opengl/comments/v5w80e/instancing_how_to_account_for_new_data_after/

pub type Pixels = u16;

pub struct GPU {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl GPU {
    pub async fn new() -> Self {
        let instance = Instance::new(&InstanceDescriptor::from_env_or_default());
        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .await
            .unwrap();
        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }
}

fn _modify_buffer<T: bytemuck::Pod + Send + Sync>(buffer: Buffer, mut data: Vec<T>) {
    let capturable = buffer.clone();
    buffer.map_async(wgpu::MapMode::Write, .., move |result| {
        if result.is_err() {
            return;
        }
        let mut view = capturable.get_mapped_range_mut(..);
        let buffer_gpu: &mut [T] = bytemuck::cast_slice_mut(&mut view);
        buffer_gpu.swap_with_slice(&mut data);
        drop(view);
        capturable.unmap();
    });
}
fn _replace_buffer<T: bytemuck::Pod + Send + Sync>(buffer: Buffer, data: Vec<T>, device: &Device) {
    let usage = buffer.usage();
    buffer.destroy();
    device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&data),
        usage,
    });
}
pub fn update_buffer<T: bytemuck::Pod + Send + Sync>(buffer: Buffer, data: Vec<T>, device: &Device) {
    if buffer.size() / size_of::<T>() as u64 == data.len() as u64 {
        _modify_buffer(buffer, data);
    } else {
        _replace_buffer(buffer, data, device);
    }
}
