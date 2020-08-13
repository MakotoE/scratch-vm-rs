use super::*;
use gloo_timers::future::TimeoutFuture;

#[derive(Debug)]
pub struct Say {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    message: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Say {
    pub fn new(id: String, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            message: None,
            next: None,
        }
    }

    fn value_to_string(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => value.to_string(),
        }
    }
}

#[async_trait(?Send)]
impl Block for Say {
    fn block_name(&self) -> &'static str {
        "Say"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "MESSAGE" => self.message = Some(block),
            _ => {}
        }
    }

    fn next(&mut self) -> Next {
        self.next.clone().into()
    }

    async fn execute(&mut self) -> Result<()> {
        let message = match &self.message {
            Some(b) => Say::value_to_string(&b.value()?),
            None => return Err("message is None".into()),
        };
        self.runtime.borrow_mut().say(Some(&message));
        self.runtime.borrow().redraw()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SayForSecs {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    message: Option<Box<dyn Block>>,
    secs: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl SayForSecs {
    pub fn new(id: String, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            message: None,
            secs: None,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SayForSecs {
    fn block_name(&self) -> &'static str {
        "SayForSecs"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "MESSAGE" => self.message = Some(block),
            "SECS" => self.secs = Some(block),
            _ => {}
        }
    }

    fn next(&mut self) -> Next {
        self.next.clone().into()
    }

    async fn execute(&mut self) -> Result<()> {
        let message = match &self.message {
            Some(b) => Say::value_to_string(&b.value()?),
            None => return Err("message is None".into()),
        };

        let seconds = match &self.secs {
            Some(b) => value_to_float(&b.value()?)?,
            None => return Err("secs is None".into()),
        };

        self.runtime.borrow_mut().say(Some(&message));
        self.runtime.borrow().redraw()?;
        TimeoutFuture::new((MILLIS_PER_SECOND * seconds).round() as u32).await;
        Ok(())
    }
}