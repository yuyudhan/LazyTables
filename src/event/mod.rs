// FilePath: src/event/mod.rs

use crate::core::error::{Error, Result};
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::{
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

/// Application events
#[derive(Debug, Clone)]
pub enum Event {
    /// Key press event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize event
    Resize(u16, u16),
    /// Periodic tick for updates
    Tick,
}

/// Event handler that manages input events
pub struct EventHandler {
    receiver: Receiver<Event>,
    _handler: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Create a new event handler with specified tick rate
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();

        let handler = thread::spawn(move || {
            let mut last_tick = std::time::Instant::now();

            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            if sender.send(Event::Key(key)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Mouse(mouse)) => {
                            if sender.send(Event::Mouse(mouse)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Resize(width, height)) => {
                            if sender.send(Event::Resize(width, height)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if sender.send(Event::Tick).is_err() {
                        break;
                    }
                    last_tick = std::time::Instant::now();
                }
            }
        });

        Self {
            receiver,
            _handler: handler,
        }
    }

    /// Start the event handler
    pub fn start(&self) -> Result<()> {
        Ok(())
    }

    /// Get the next event if available
    pub fn next(&self) -> Result<Option<Event>> {
        match self.receiver.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => {
                Err(Error::Event("Event handler disconnected".to_string()))
            }
        }
    }
}
