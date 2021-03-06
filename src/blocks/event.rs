use crate::broadcaster::BroadcastMsg;

use super::*;

pub fn get_block(
    name: &str,
    id: BlockID,
    runtime: Runtime,
) -> Result<Box<dyn Block + Send + Sync>> {
    Ok(match name {
        "whenflagclicked" => Box::new(WhenFlagClicked::new(id, runtime)),
        "whenbroadcastreceived" => Box::new(WhenBroadcastReceived::new(id, runtime)),
        "broadcast" => Box::new(Broadcast::new(id, runtime)),
        "broadcastandwait" => Box::new(BroadcastAndWait::new(id, runtime)),
        "whenthisspriteclicked" => Box::new(WhenThisSpriteClicked::new(id, runtime)),
        _ => return Err(Error::msg(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct WhenFlagClicked {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
}

impl WhenFlagClicked {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait]
impl Block for WhenFlagClicked {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "WhenFlagClicked",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        if self.runtime.sprite.read().await.is_a_clone() {
            Ok(Next::None)
        } else {
            Next::continue_(self.next)
        }
    }
}

#[derive(Debug)]
pub struct WhenBroadcastReceived {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    broadcast_id: String,
    started: bool,
}

impl WhenBroadcastReceived {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            broadcast_id: String::new(),
            started: false,
        }
    }
}

#[async_trait]
impl Block for WhenBroadcastReceived {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "WhenBroadcastReceived",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("BROADCAST_OPTION", self.broadcast_id.clone())],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "BROADCAST_OPTION" {
            self.broadcast_id = get_field_value(field, 0)?.to_string();
        }
        Ok(())
    }

    async fn execute(&mut self) -> Result<Next> {
        if self.started {
            self.runtime
                .global
                .broadcaster
                .send(BroadcastMsg::Finished(self.broadcast_id.clone()))?;
            self.started = false;
            return Ok(Next::None);
        }

        let mut recv = self.runtime.global.broadcaster.subscribe();
        loop {
            if let BroadcastMsg::Start(s) = recv.recv().await? {
                if s == self.broadcast_id {
                    self.started = true;
                    return Next::loop_(self.next);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Broadcast {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    message: Box<dyn Block + Send + Sync>,
}

impl Broadcast {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            message: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for Broadcast {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Broadcast",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("message", self.message.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        if key == "BROADCAST_INPUT" {
            self.message = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let msg = self.message.value().await?.to_string();
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::Start(msg))?;
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct BroadcastAndWait {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    message: Box<dyn Block + Send + Sync>,
}

impl BroadcastAndWait {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            message: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for BroadcastAndWait {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "BroadcastAndWait",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("message", self.message.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        if key == "BROADCAST_INPUT" {
            self.message = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let msg = self.message.value().await?.to_string();
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::Start(msg.clone()))?;
        let mut recv = self.runtime.global.broadcaster.subscribe();
        loop {
            if let BroadcastMsg::Finished(s) = recv.recv().await? {
                if s == msg {
                    return Next::continue_(self.next);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct WhenThisSpriteClicked {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
}

impl WhenThisSpriteClicked {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait]
impl Block for WhenThisSpriteClicked {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "WhenThisSpriteClicked",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let mut channel = self.runtime.global.broadcaster.subscribe();
        loop {
            if let BroadcastMsg::MouseClick(c) = channel.recv().await? {
                let curr_rectangle = self.runtime.sprite.read().await.rectangle();
                if curr_rectangle.contains(&c.into()) {
                    return Next::continue_(self.next);
                }
            }
        }
    }
}
