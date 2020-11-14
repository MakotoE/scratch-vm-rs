use super::*;

pub fn get_block(name: &str, id: BlockID, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "setvariableto" => Box::new(SetVariable::new(id, runtime)),
        "changevariableby" => Box::new(ChangeVariable::new(id, runtime)),
        "hidevariable" => Box::new(HideVariable::new(id, runtime)),
        "showvariable" => Box::new(ShowVariable::new(id, runtime)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct SetVariable {
    id: BlockID,
    runtime: Runtime,
    variable_id: String,
    value: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl SetVariable {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            variable_id: String::new(),
            value: None,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SetVariable {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetVariable",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![("variable_id", self.variable_id.clone())],
            vec![("value", &self.value)],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "VALUE" => self.value = Some(block),
            _ => {}
        }
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "VARIABLE" {
            if let Some(s) = field.get(1).cloned().flatten() {
                self.variable_id = s;
            }
        }
        Ok(())
    }

    async fn execute(&mut self) -> Next {
        let value = match &self.value {
            Some(v) => v.value().await?,
            None => return Next::Err(wrap_err!("value is None")),
        };
        self.runtime
            .global
            .variables
            .write()
            .await
            .insert(self.variable_id.clone(), value);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct ChangeVariable {
    id: BlockID,
    runtime: Runtime,
    variable_id: String,
    value: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl ChangeVariable {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            variable_id: String::new(),
            value: None,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for ChangeVariable {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ChangeVariable",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![("variable_id", self.variable_id.clone())],
            vec![("value", &self.value)],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "VALUE" => self.value = Some(block),
            _ => {}
        }
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "VARIABLE" {
            if let Some(s) = field.get(1).cloned().flatten() {
                self.variable_id = s;
            }
        }
        Ok(())
    }

    async fn execute(&mut self) -> Next {
        let previous_value = match self
            .runtime
            .global
            .variables
            .write()
            .await
            .remove(&self.variable_id)
        {
            Some(v) => v,
            None => {
                return Next::Err(wrap_err!(format!(
                    "variable {} does not exist",
                    &self.variable_id
                )))
            }
        };

        let previous_float = value_to_float(&previous_value).unwrap_or(0.0);

        let value = match &self.value {
            Some(b) => b.value().await?,
            None => return Next::Err(wrap_err!("value is None")),
        };

        let new_value = previous_float + value_to_float(&value)?;
        self.runtime
            .global
            .variables
            .write()
            .await
            .insert(self.variable_id.clone(), new_value.into());
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct HideVariable {
    id: BlockID,
    runtime: Runtime,
}

impl HideVariable {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for HideVariable {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "HideVariable",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}

#[derive(Debug)]
pub struct ShowVariable {
    id: BlockID,
    runtime: Runtime,
}

impl ShowVariable {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for ShowVariable {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ShowVariable",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}
