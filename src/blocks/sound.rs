use super::*;

pub fn get_block(
    name: &str,
    id: String,
    runtime: Rc<RwLock<SpriteRuntime>>,
) -> Result<Box<dyn Block>> {
    Ok(match name {
        "play" => Box::new(Play::new(id, runtime)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct Play {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Play {
    pub fn new(id: String, _runtime: Rc<RwLock<SpriteRuntime>>) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for Play {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Play",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}