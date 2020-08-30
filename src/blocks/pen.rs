use super::*;
use palette::IntoColor;
use palette::Mix;

pub fn get_block(
    name: &str,
    id: &str,
    runtime: Rc<RefCell<SpriteRuntime>>,
) -> Result<Box<dyn Block>> {
    Ok(match name {
        "penDown" => Box::new(PenDown::new(id, runtime)),
        "penUp" => Box::new(PenUp::new(id, runtime)),
        "setPenColorToColor" => Box::new(SetPenColorToColor::new(id, runtime)),
        "setPenSizeTo" => Box::new(SetPenSizeTo::new(id, runtime)),
        "clear" => Box::new(Clear::new(id, runtime)),
        "setPenShadeToNumber" => Box::new(SetPenShadeToNumber::new(id, runtime)),
        "setPenHueToNumber" => Box::new(SetPenHueToNumber::new(id, runtime)),
        _ => return Err(format!("{} does not exist", name).into()),
    })
}

#[derive(Debug)]
pub struct PenDown {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl PenDown {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for PenDown {
    fn block_name(&self) -> &'static str {
        "PenDown"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        self.runtime.borrow_mut().pen_down();
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct PenUp {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl PenUp {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for PenUp {
    fn block_name(&self) -> &'static str {
        "PenUp"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        self.runtime.borrow_mut().pen_up();
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SetPenColorToColor {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    color: Option<Box<dyn Block>>,
}

impl SetPenColorToColor {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
            color: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SetPenColorToColor {
    fn block_name(&self) -> &'static str {
        "SetPenColorToColor"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "COLOR" => self.color = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let color_value = match &self.color {
            Some(b) => b.value()?,
            None => return Next::Err("color is None".into()),
        };
        let color = color_value
            .as_str()
            .ok_or_else(|| Error::from("color is not a string"))?;
        self.runtime
            .borrow_mut()
            .set_pen_color(&runtime::hex_to_color(color)?);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SetPenSizeTo {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    size: Option<Box<dyn Block>>,
}

impl SetPenSizeTo {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
            size: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SetPenSizeTo {
    fn block_name(&self) -> &'static str {
        "SetPenSizeTo"
    }

    fn id(&self) -> &str {
        &self.id
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
            Some(b) => value_to_float(&b.value()?)?,
            None => return Next::Err("color is None".into()),
        };

        self.runtime.borrow_mut().set_pen_size(size);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct Clear {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Clear {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Clear {
    fn block_name(&self) -> &'static str {
        "Clear"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        self.runtime.borrow_mut().pen_clear();
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SetPenShadeToNumber {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    shade: Option<Box<dyn Block>>,
}

impl SetPenShadeToNumber {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
            shade: None,
        }
    }

    fn set_shade(color: &palette::Hsv, shade: f32) -> palette::Hsv {
        // https://github.com/LLK/scratch-vm/blob/c6962cb390ba2835d64eb21c0456707b51642084/src/extensions/scratch3_pen/index.js#L718
        let mut new_shade = shade % 200.0;
        if new_shade < 0.0 {
            new_shade += 200.0
        }

        // https://github.com/LLK/scratch-vm/blob/c6962cb390ba2835d64eb21c0456707b51642084/src/extensions/scratch3_pen/index.js#L750
        let constrained_shade = if new_shade > 100.0 {
            200.0 - new_shade
        } else {
            new_shade
        };

        let bright = palette::Hsv::new(color.hue, 1.0, 1.0);
        if constrained_shade < 50.0 {
            palette::Hsv::new(0.0, 0.0, 0.0).mix(&bright, (10.0 + shade) / 60.0)
        } else {
            bright.mix(&palette::Hsv::new(0.0, 0.0, 1.0), (shade - 50.0) / 60.0)
        }
    }
}

#[async_trait(?Send)]
impl Block for SetPenShadeToNumber {
    fn block_name(&self) -> &'static str {
        "SetPenShadeToNumber"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "SHADE" => self.shade = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let shade = match &self.shade {
            Some(b) => value_to_float(&b.value()?)?,
            None => return Next::Err("shade is None".into()),
        };
        let mut runtime = self.runtime.borrow_mut();
        let color = runtime.pen_color().into_hsv();
        let new_color = SetPenShadeToNumber::set_shade(&color, shade as f32);
        runtime.set_pen_color(&new_color.into());
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SetPenHueToNumber {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    hue: Option<Box<dyn Block>>, // [0, 100]
}

impl SetPenHueToNumber {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
            hue: None,
        }
    }

    fn set_hue(color: &palette::Hsv, hue: f32) -> palette::Hsv {
        palette::Hsv::new(hue / 200.0 * 360.0, color.saturation, color.value)
    }
}

#[async_trait(?Send)]
impl Block for SetPenHueToNumber {
    fn block_name(&self) -> &'static str {
        "SetPenHueToNumber"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "HUE" => self.hue = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let hue = match &self.hue {
            Some(b) => value_to_float(&b.value()?)?,
            None => return Next::Err("hue is None".into()),
        };

        let mut runtime = self.runtime.borrow_mut();
        let new_color = SetPenHueToNumber::set_hue(runtime.pen_color(), hue as f32);
        runtime.set_pen_color(&new_color);
        Next::continue_(self.next.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod set_pen_shade_to_number {
        use super::*;

        #[test]
        fn test_set_shade() {
            struct Test {
                color: palette::Hsv,
                shade: f32,
                expected: palette::Hsv,
            }

            let tests: Vec<Test> = vec![
                Test {
                    color: palette::Hsv::new(0.0, 0.0, 0.0),
                    shade: 0.0,
                    expected: palette::Hsv::new(0.0, 0.16666667, 0.16666667),
                },
                Test {
                    color: palette::Hsv::new(0.0, 0.0, 1.0),
                    shade: 0.0,
                    expected: palette::Hsv::new(0.0, 0.16666667, 0.16666667),
                },
                Test {
                    color: palette::Hsv::new(0.0, 0.0, 0.0),
                    shade: 100.0,
                    expected: palette::Hsv::new(0.0, 0.16666669, 1.0),
                },
                Test {
                    color: palette::Hsv::new(0.0, 0.0, 1.0),
                    shade: 100.0,
                    expected: palette::Hsv::new(0.0, 0.16666669, 1.0),
                },
                Test {
                    color: palette::Hsv::new(0.0, 0.0, 0.0),
                    shade: 50.0,
                    expected: palette::Hsv::new(0.0, 1.0, 1.0),
                },
                Test {
                    color: palette::Hsv::new(240.0, 1.0, 1.0),
                    shade: 50.0,
                    expected: palette::Hsv::new(240.0, 1.0, 1.0),
                },
            ];

            for (i, test) in tests.iter().enumerate() {
                let result = SetPenShadeToNumber::set_shade(&test.color, test.shade);
                assert_eq!(result, test.expected, "{}", i);
            }
        }
    }

    mod set_pen_hue_to_number {
        use super::*;

        #[test]
        fn test_set_hue() {
            struct Test {
                color: palette::Hsv,
                hue: f32,
                expected: palette::Hsv,
            }

            let tests: Vec<Test> = vec![
                Test {
                    color: palette::Hsv::new(0.0, 0.0, 0.0),
                    hue: 0.0,
                    expected: palette::Hsv::new(0.0, 0.0, 0.0),
                },
                Test {
                    color: palette::Hsv::new(0.0, 1.0, 1.0),
                    hue: 0.0,
                    expected: palette::Hsv::new(0.0, 1.0, 1.0),
                },
                Test {
                    color: palette::Hsv::new(0.0, 0.0, 0.0),
                    hue: 50.0,
                    expected: palette::Hsv::new(90.0, 0.0, 0.0),
                },
                Test {
                    color: palette::Hsv::new(0.0, 0.0, 0.0),
                    hue: 100.0,
                    expected: palette::Hsv::new(180.0, 0.0, 0.0),
                },
                Test {
                    color: palette::Hsv::new(0.0, 0.0, 0.0),
                    hue: 200.0,
                    expected: palette::Hsv::new(360.0, 0.0, 0.0),
                },
            ];

            for (i, test) in tests.iter().enumerate() {
                let result = SetPenHueToNumber::set_hue(&test.color, test.hue);
                assert_eq!(result, test.expected, "{}", i);
            }
        }
    }
}
