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
    /// If it was revealed after a pop, return the states that were popped off,
    /// topmost state last.
    fn on_reveal(&mut self, states: Option<Vec<GamemodeBox>>, assets: &Assets) {}
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
    /// Pop the top mode off the stack
    Pop,
    /// The most customizable: pop N entries off the stack, then push some new ones.
    /// The last entry in the vec will become the top of the stack.
    PopNAndPush(usize, Vec<GamemodeBox>),
}

impl Transition {
    /// Apply the transition
    pub fn apply(self, stack: &mut Vec<GamemodeBox>, assets: &Assets) {
        match self {
            Transition::None => {}
            Transition::Swap(new) => {
                if !stack.is_empty() {
                    stack.pop();
                }
                stack.push(new);
                stack.last_mut().unwrap().on_reveal(None, assets);
            }
            Transition::Push(new) => {
                stack.push(new);
                stack.last_mut().unwrap().on_reveal(None, assets);
            }
            Transition::Pop => {
                // At 2 or more, we pop down to at least one state
                // this would be very bad otherwise
                if stack.len() >= 2 {
                    let popped = stack.pop().unwrap();
                    stack
                        .last_mut()
                        .unwrap()
                        .on_reveal(Some(vec![popped]), assets);
                }
            }
            Transition::PopNAndPush(count, mut news) => {
                let lower_limit = if news.is_empty() { 1 } else { 0 };
                let trunc_len = lower_limit.max(stack.len() - count);
                let removed = stack.drain(trunc_len..).collect();

                if news.is_empty() {
                    // we only popped, so the last is revealed!
                    stack.last_mut().unwrap().on_reveal(Some(removed), assets);
                } else {
                    stack.append(&mut news);
                    stack.last_mut().unwrap().on_reveal(None, assets);
                }
            }
        }
    }
}
