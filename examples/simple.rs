use ure::{data::Data, get_group, group, new_span};

fn main() {
    let mut data = Data::default();
    let windows = new_span!(data, 1, window, surface);
    let instances_1 = new_span!(data, 10, transform_2d);
    let instances_2 = new_span!(data, 10, transform_2d);
    let instances = group!(instances_1, instances_2);
    get_group!(data, instances, transform_2d);
}
