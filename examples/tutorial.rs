use ure::{data::Data, game::tf::Transform2D, get_group, group, new_span};

fn main() {
    // Default Data is empty.
    let mut data = Data::default();
    let windows = new_span!(data, 1, window, surface);
    // Defining a span takes in all components which the span uses.
    let instances_1 = new_span!(data, 10, transform_2d);
    // Multiple spans can be of the same component.
    let instances_2 = new_span!(data, 10, transform_2d);
    // Groups combine spans into iterators.
    // Groups do not keep track of what components they have in common.
    let instances = group!(instances_1, instances_2);
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

// Functions take contiguous slices.
fn example_fn(transform_2d: &mut[Transform2D]) {

}