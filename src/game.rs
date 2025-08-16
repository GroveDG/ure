#[repr(usize)]
pub enum GameComponent {
    Color,
    Transform,
    InstanceBuffer,
    Mesh,
    _SIZE,
}