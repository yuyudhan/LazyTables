// FilePath: src/event/mod.rs

#![forbid(unsafe_code)]

use crate::core::error::{Error, Result};
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::{
    sync::mpsc::{self, Receiver, RecvTimeoutError},
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
                // Calculate remaining time until next tick
                let timeout = tick_rate.saturating_sub(last_tick.elapsed());

                // If tick is due, send it immediately without polling for events
                if timeout.is_zero() {
                    if sender.send(Event::Tick).is_err() {
                        break;
                    }
                    last_tick = std::time::Instant::now();
                    continue;
                }

                // Poll for events with remaining time until next tick
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

                // Check again if tick is due after processing events
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

    /// Get the next event, blocking with timeout to allow CPU to idle
    pub fn next(&self) -> Result<Option<Event>> {
        // Use recv_timeout to block and allow CPU to enter idle states
        // Timeout matches the tick rate to ensure timely UI updates
        match self.receiver.recv_timeout(Duration::from_millis(250)) {
            Ok(event) => Ok(Some(event)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => {
                Err(Error::Event("Event handler disconnected".to_string()))
            }
        }
    }
}
