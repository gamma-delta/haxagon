use std::any::Any;

use crate::{assets::Assets, controls::InputSubscriber};

pub type GamemodeBox = Box<dyn Gamemode>;
pub type DrawerBox = Box<dyn GamemodeDrawer>;

/// Things the engine can update and draw
pub trait Gamemode: Any {
    /// Update the state.
    ///
    /// Return how to swap to another state if need be.
    fn update(
        &mut self,
        controls: &InputSubscriber,
        frame_info: FrameInfo,
        assets: &Assets,
    ) -> Transition;

    /// Gather information about how to draw this state.
    fn get_draw_info(&mut self) -> DrawerBox;

    /// Called when the state newly comes on top of the stack,
    /// either from being pushed there or revealed after a pop.
    ///
    /// If the `PopWith` variant of `Transition` was used, this contains the data popped.
    fn on_reveal(&mut self, _passed: Option<Box<dyn Any>>, _assets: &Assets) {}
}

/// Data on how to draw a state
pub trait GamemodeDrawer: Send + Any {
    fn draw(&self, assets: &Assets, frame_info: FrameInfo);
}

/// Information about a frame.
#[derive(Copy, Clone)]
pub struct FrameInfo {
    /// Time the previous frame took in seconds.
    pub dt: f32,
    /// Number of frames that have happened since the program started.
    /// For Gamemodes this is update frames; for GamemodeDrawers this is draw frames.
    // at 2^64 frames, this will run out about when the sun dies!
    // 0.97 x expected sun lifetime!
    // how exciting.
    pub frames_ran: u64,
}
/// Ways modes can transition
#[allow(dead_code)]
pub enum Transition {
    /// Do nothing
    None,
    /// Pop the top mode off and replace it with this
    Swap(GamemodeBox),
    /// Push this mode onto the stack
    Push(GamemodeBox),
    /// Pop the current mode off the stack
    Pop,
    /// Pop the current mode and pass the given data down to the next state.
    PopWith(Box<dyn Any>),
    /// The most customizable: pop N entries off the stack, then push some new ones.
    /// The last entry in the vec will become the top of the stack.
    PopNAndPush(usize, Vec<GamemodeBox>),
}

impl Transition {
    /// Apply the transition
    pub fn apply(self, stack: &mut Vec<GamemodeBox>, assets: &Assets) {
        match self {
            Transition::None => {
                return;
            }
            Transition::Swap(new) => {
                if !stack.is_empty() {
                    stack.pop();
                }
                stack.push(new);
            }
            Transition::Push(new) => {
                stack.push(new);
            }
            Transition::Pop => {
                // At 2 or more, we pop down to at least one state
                // this would be very bad otherwise
                if stack.len() >= 2 {
                    stack.pop().unwrap();
                }
            }
            Transition::PopWith(data) => {
                if stack.len() >= 2 {
                    stack.pop().unwrap();
                    stack.last_mut().unwrap().on_reveal(Some(data), assets);
                }
                return;
            }
            Transition::PopNAndPush(count, news) => {
                let lower_limit = if news.is_empty() { 1 } else { 0 };
                let trunc_len = lower_limit.max(stack.len() - count);
                stack.truncate(trunc_len);
                stack.extend(news);
            }
        }
        stack.last_mut().unwrap().on_reveal(None, assets);
    }
}
