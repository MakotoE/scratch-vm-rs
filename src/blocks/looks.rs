use super::*;
use crate::coordinate::Scale;
use gloo_timers::future::TimeoutFuture;
use sprite_runtime::{HideStatus, Text};

pub fn get_block(name: &str, id: BlockID, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "say" => Box::new(Say::new(id, runtime)),
        "sayforsecs" => Box::new(SayForSecs::new(id, runtime)),
        "gotofrontback" => Box::new(GoToFrontBack::new(id, runtime)),
        "hide" => Box::new(Hide::new(id, runtime)),
        "show" => Box::new(Show::new(id, runtime)),
        "seteffectto" => Box::new(SetEffectTo::new(id, runtime)),
        "nextcostume" => Box::new(NextCostume::new(id, runtime)),
        "changeeffectby" => Box::new(ChangeEffectBy::new(id, runtime)),
        "setsizeto" => Box::new(SetSizeTo::new(id, runtime)),
        "switchcostumeto" => Box::new(SwitchCostumeTo::new(id, runtime)),
        "costume" => Box::new(Costume::new(id, runtime)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct Say {
    id: BlockID,
    runtime: Runtime,
    message: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Say {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            message: None,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Say {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Say",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![("message", &self.message)],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "MESSAGE" => self.message = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let message = match &self.message {
            Some(b) => value_to_string(b.value().await?),
            None => return Next::Err(wrap_err!("message is None")),
        };
        self.runtime.sprite.write().await.say(Text {
            id: self.id,
            text: Some(message),
        });
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SayForSecs {
    id: BlockID,
    runtime: Runtime,
    message: Option<Box<dyn Block>>,
    secs: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl SayForSecs {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            message: None,
            secs: None,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SayForSecs {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SayForSecs",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![("message", &self.message), ("secs", &self.secs)],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "MESSAGE" => self.message = Some(block),
            "SECS" => self.secs = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let message = match &self.message {
            Some(b) => value_to_string(b.value().await?),
            None => return Next::Err(wrap_err!("message is None")),
        };

        let seconds = match &self.secs {
            Some(b) => value_to_float(&b.value().await?)?,
            None => return Next::Err(wrap_err!("secs is None")),
        };

        self.runtime.sprite.write().await.say(Text {
            id: self.id,
            text: Some(message),
        });
        TimeoutFuture::new((MILLIS_PER_SECOND * seconds).round() as u32).await;
        self.runtime.sprite.write().await.say(Text {
            id: self.id,
            text: None,
        });
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct GoToFrontBack {
    id: BlockID,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl GoToFrontBack {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for GoToFrontBack {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "GoToFrontBack",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}

#[derive(Debug)]
pub struct Hide {
    id: BlockID,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Hide {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Hide {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Hide",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }

    async fn execute(&mut self) -> Next {
        self.runtime.sprite.write().await.set_hide(HideStatus::Hide);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct Show {
    id: BlockID,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Show {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Show {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Show",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }

    async fn execute(&mut self) -> Next {
        self.runtime.sprite.write().await.set_hide(HideStatus::Show);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SetEffectTo {
    id: BlockID,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl SetEffectTo {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for SetEffectTo {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetEffectTo",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}

#[derive(Debug)]
pub struct NextCostume {
    id: BlockID,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl NextCostume {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for NextCostume {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "NextCostume",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}

#[derive(Debug)]
pub struct ChangeEffectBy {
    id: BlockID,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl ChangeEffectBy {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for ChangeEffectBy {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ChangeEffectBy",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}

#[derive(Debug)]
pub struct SetSizeTo {
    id: BlockID,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    size: Option<Box<dyn Block>>, // TODO noop block to get rid of Option
}

impl SetSizeTo {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            size: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SetSizeTo {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetSizeTo",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "SIZE" => self.size = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let size = match &self.size {
            Some(b) => b,
            None => return Next::Err(wrap_err!("size is none")),
        };

        let scale = value_to_float(&size.value().await?)? / 100.0;

        self.runtime
            .sprite
            .write()
            .await
            .set_scale(Scale { x: scale, y: scale });

        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SwitchCostumeTo {
    id: BlockID,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl SwitchCostumeTo {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for SwitchCostumeTo {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SwitchCostumeTo",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}

#[derive(Debug)]
pub struct Costume {
    id: BlockID,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Costume {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for Costume {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Costume",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}
