mod event;
mod mouse;
mod parser;
mod spawn;

pub(crate) use event::InputEvent;
pub(crate) use spawn::spawn_input_thread;
