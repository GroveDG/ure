use sdl2::{event::EventPollIterator, EventPump, Sdl};

pub struct Events {
    event_pump: EventPump,
}

impl Events {
    pub fn new(sdl: &Sdl) -> Result<Self, String> {
        Ok(Self {
            event_pump: sdl.event_pump()?,
        })
    }
    pub fn poll(&mut self) -> EventPollIterator {
        self.event_pump.poll_iter()
    }
}
