use super::*;
use crate::coordinate::{CanvasCoordinate, SpriteRectangle};
use crate::sprite::SpriteID;
use crate::vm::ThreadID;
use graphics_buffer::RenderBuffer;
use input::Key;
use std::collections::HashSet;
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct Broadcaster {
    sender: Sender<BroadcastMsg>,
}

impl Broadcaster {
    pub fn new() -> Self {
        Self {
            sender: channel(32).0,
        }
    }

    pub fn send(&self, m: BroadcastMsg) -> Result<()> {
        self.sender.send(m)?;
        Ok(())
    }

    pub fn subscribe(&self) -> Receiver<BroadcastMsg> {
        self.sender.subscribe()
    }
}

#[derive(Debug, Clone)]
pub enum BroadcastMsg {
    Start(String),
    Finished(String),
    Clone(SpriteID),
    DeleteClone(SpriteID),
    Stop(Stop),
    ChangeLayer {
        sprite: SpriteID,
        action: LayerChange,
    },
    MouseClick(CanvasCoordinate),
    RequestMousePosition,
    MousePosition(CanvasCoordinate),
    RequestPressedKeys,
    PressedKeys(HashSet<Key>),
    RequestSpriteRectangle(SpriteID),
    SpriteRectangle {
        sprite: SpriteID,
        rectangle: SpriteRectangle,
    },
    /// Requests image but with sprite removed
    RequestCanvasImage(SpriteID),
    CanvasImage(RenderBuffer),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Stop {
    All,
    ThisThread(ThreadID),
    OtherThreads(ThreadID),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LayerChange {
    Front,
    Back,
}
