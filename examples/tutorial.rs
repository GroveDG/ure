use ure::{app::init_windows, data::{Data, Group, Span}, extend_span, get_group, get_span, gpu::{init_surfaces, Gpu}, new_span, tf::Transform2D};

fn main() {
    // Default Data is empty.
    let mut data = Data::default();
    // Defining a span takes in all components which the span uses.
    let instances_1 = new_span!(data, 10, transform_2d);
    // Multiple spans can be of the same component.
    let instances_2 = new_span!(data, 10, transform_2d);
    // Groups combine spans into iterators.
    // Groups do not keep track of what components they have in common.
    let instances = Group::new(&[instances_1, instances_2]);
    // Enclose when getting a group so that the components are not
    // leaked into the surrounding context.
    {
        // When getting a group, you have to specify components which
        // the group has in common.
        get_group!(data, instances, transform_2d mut);
        // Components of groups yield the slices represented by their
        // member spans.
        for transform_2d_slice in transform_2d {
            example_fn(transform_2d_slice);
        }
    }
}

struct Game {
    data: Data,
    gpu: Gpu,
    windows: Span,
}
impl ure::app::Game for Game {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let mut data = Data::default();
        let gpu = futures::executor::block_on(Gpu::new());
        let mut windows = new_span!(data, 1, window, surface);
        {
            extend_span!(data, windows, 1, window);
            init_windows(window, event_loop);
            get_span!(data, windows, window);
            extend_span!(data, windows, 1, surface);
            init_surfaces(window, surface, &gpu);
        }
        Game {
            data: Data::default(),
            gpu,
            windows,
        }
    }

    fn run(self) {
        todo!()
    }
}

// Functions take contiguous slices.
fn example_fn(transform_2d: &mut[Transform2D]) {

}