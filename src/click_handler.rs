use piston_window::{Button, Event, MouseButton, PressEvent};
use std::time::{Duration, Instant};

/// Handles double click events.
pub struct DoubleClickHandler {
    pub last_click: Instant,
    pub click_timeout: Duration,
    pub handler: Box<dyn FnMut() -> bool>,
    pub button: MouseButton,
}

impl DoubleClickHandler {
    pub fn new(
        handler: Box<dyn FnMut() -> bool>,
        button: MouseButton,
        timeout: Option<u64>,
    ) -> DoubleClickHandler {
        let click_timeout = Duration::from_millis(timeout.unwrap_or(500));
        DoubleClickHandler {
            last_click: Instant::now(),
            click_timeout,
            handler,
            button,
        }
    }

    /// Handle the event on the condition that the mouse button and timeout is/is less than what was specified in the constructor.
    pub fn handle_if_button_pressed(&mut self, event: &Event) -> bool {
        if let Some(Button::Mouse(button)) = event.press_args() {
            if button == self.button && self.is_double_click() {
                return (self.handler)();
            }
        }
        false
    }

    /// Check if the time between the last click and now is less than the click timeout
    fn is_double_click(&mut self) -> bool {
        let now = Instant::now();
        let is_double_click = now - self.last_click < self.click_timeout;
        self.last_click = now;
        is_double_click
    }
}
