use super::*;
use crate::broadcaster::BroadcastMsg;
use crate::coordinate::CanvasRectangle;
use crate::event_sender::{KeyOption, KeyboardKey};
use crate::sprite::SpriteID;
use gloo_timers::future::TimeoutFuture;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::str::FromStr;


pub fn get_block(name: &str, id: BlockID, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "keypressed" => Box::new(KeyPressed::new(id, runtime)),
        "keyoptions" => Box::new(KeyOptions::new(id, runtime)),
        "coloristouchingcolor" => Box::new(ColorIsTouchingColor::new(id, runtime)),
        "touchingcolor" => Box::new(TouchingColor::new(id)),
        "touchingobject" => Box::new(TouchingObject::new(id, runtime)),
        "touchingobjectmenu" => Box::new(TouchingObjectMenu::new(id)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct KeyPressed {
    id: BlockID,
    runtime: Runtime,
    key_option: Box<dyn Block>,
}

impl KeyPressed {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            key_option: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait(?Send)]
impl Block for KeyPressed {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "KeyPressed",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("KEY_OPTION", &self.key_option)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "KEY_OPTION" {
            self.key_option = block;
        }
    }

    async fn value(&self) -> Result<Value> {
        let key_option: KeyOption = self.key_option.value().await?.try_into()?;
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::RequestPressedKeys)?;
        let mut receiver = self.runtime.global.broadcaster.subscribe();
        loop {
            if let BroadcastMsg::PressedKeys(keys) = receiver.recv().await? {
                return Ok(match key_option {
                    KeyOption::Any => true,
                    KeyOption::Key(key) => keys.contains(&key),
                }
                .into());
            }
        }
    }
}

#[derive(Debug)]
pub struct KeyOptions {
    id: BlockID,
    key: KeyOption,
}

impl KeyOptions {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self {
            id,
            key: KeyOption::Key(KeyboardKey::Space),
        }
    }
}

#[async_trait(?Send)]
impl Block for KeyOptions {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "KeyOptions",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("KEY_OPTION", format!("{}", self.key))],
            vec![],
            vec![],
        )
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "KEY_OPTION" {
            self.key = KeyOption::from_str(get_field_value(field, 0)?)?;
        }
        Ok(())
    }

    async fn value(&self) -> Result<Value> {
        Ok(Value::KeyOption(self.key))
    }
}

try_from_value!(KeyOption);

#[derive(Debug)]
pub struct ColorIsTouchingColor {
    id: BlockID,
}

impl ColorIsTouchingColor {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for ColorIsTouchingColor {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ColorIsTouchingColor",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}

#[derive(Debug)]
pub struct TouchingColor {
    id: BlockID,
    color: Box<dyn Block>,
}

impl TouchingColor {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            color: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait(?Send)]
impl Block for TouchingColor {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "TouchingColor",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("COLOR", &self.color)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "COLOR" {
            self.color = block;
        }
    }

    // async fn value(&self) -> Result<Value> {}
}

#[derive(Debug)]
pub struct TouchingObject {
    id: BlockID,
    runtime: Runtime,
    menu: Box<dyn Block>,
}

impl TouchingObject {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            menu: Box::new(EmptyInput {}),
        }
    }

    fn sprite_on_edge(rectangle: &CanvasRectangle) -> bool {
        rectangle.top_left.x < 0.0
            || rectangle.top_left.y < 0.0
            || rectangle.top_left.x + rectangle.size.width > 460.0
            || rectangle.top_left.y + rectangle.size.height > 180.0
    }
}

#[async_trait(?Send)]
impl Block for TouchingObject {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "TouchingObject",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("menu", &self.menu)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "TOUCHINGOBJECTMENU" {
            self.menu = block;
        }
    }

    async fn value(&self) -> Result<Value> {
        let option: TouchingObjectOption = self.menu.value().await?.try_into()?;

        let sprite_rectangle = self.runtime.sprite.read().await.rectangle();

        match option {
            TouchingObjectOption::MousePointer => {
                TimeoutFuture::new(0).await; // Prevents unresponsiveness
                self.runtime
                    .global
                    .broadcaster
                    .send(BroadcastMsg::RequestMousePosition)?;

                let canvas_rectangle: CanvasRectangle = sprite_rectangle.into();
                let mut channel = self.runtime.global.broadcaster.subscribe();
                loop {
                    if let BroadcastMsg::MousePosition(position) = channel.recv().await? {
                        return Ok(canvas_rectangle.contains(&position).into());
                    }
                }
            }
            TouchingObjectOption::Edge => {
                return Ok(TouchingObject::sprite_on_edge(&sprite_rectangle.into()).into())
            }
            TouchingObjectOption::Sprite(sprite_name) => {
                let id = SpriteID::from_sprite_name(&sprite_name);
                let msg = BroadcastMsg::RequestSpriteRectangle(id);
                self.runtime.global.broadcaster.send(msg)?;

                let mut channel = self.runtime.global.broadcaster.subscribe();
                loop {
                    if let BroadcastMsg::SpriteRectangle { sprite, rectangle } =
                        channel.recv().await?
                    {
                        if sprite == id {
                            return Ok(sprite_rectangle.intersects(&rectangle).into());
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct TouchingObjectMenu {
    id: BlockID,
    option: TouchingObjectOption,
}

impl TouchingObjectMenu {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            option: TouchingObjectOption::MousePointer,
        }
    }
}

#[async_trait(?Send)]
impl Block for TouchingObjectMenu {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "TouchingObjectMenu",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("option", self.option.to_string())],
            vec![],
            vec![],
        )
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "TOUCHINGOBJECTMENU" {
            self.option = TouchingObjectOption::from_str(get_field_value(field, 0)?)?;
        }
        Ok(())
    }

    async fn value(&self) -> Result<Value> {
        Ok(Value::TouchingObjectOption(self.option.clone()))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TouchingObjectOption {
    MousePointer,
    Edge,
    Sprite(String),
}

impl FromStr for TouchingObjectOption {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "_mouse_" => Self::MousePointer,
            "_edge_" => Self::Edge,
            _ => Self::Sprite(s.to_string()),
        })
    }
}

impl Display for TouchingObjectOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TouchingObjectOption::MousePointer => "_mouse_",
            TouchingObjectOption::Edge => "_edge_",
            TouchingObjectOption::Sprite(s) => &s,
        })
    }
}

try_from_value!(TouchingObjectOption);
