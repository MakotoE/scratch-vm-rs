/// Center = 0, 0
/// Left = -240, right = +240
/// Top = -180, bottom = +180
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SpriteCoordinate {
    pub x: f64,
    pub y: f64,
}

impl SpriteCoordinate {
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl From<CanvasCoordinate> for SpriteCoordinate {
    fn from(c: CanvasCoordinate) -> Self {
        Self {
            x: c.x - 240.0,
            y: -c.y + 180.0,
        }
    }
}

/// Left = 0, right = +480
/// Top = 0, bottom +360
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CanvasCoordinate {
    pub x: f64,
    pub y: f64,
}

impl CanvasCoordinate {
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn scale(&self, scale: &Scale) -> Self {
        Self {
            x: self.x * scale.x,
            y: self.y * scale.y,
        }
    }
}

impl Default for CanvasCoordinate {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl From<SpriteCoordinate> for CanvasCoordinate {
    fn from(sprite_coordinate: SpriteCoordinate) -> Self {
        Self {
            x: 240.0 + sprite_coordinate.x,
            y: 180.0 - sprite_coordinate.y,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size {
    pub width: f64,
    pub length: f64,
}

impl Size {
    pub fn multiply(&self, scale: &Scale) -> Self {
        Self {
            width: self.width * scale.x,
            length: self.length * scale.y,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scale {
    pub x: f64,
    pub y: f64,
}

impl Scale {
    pub fn multiply(&self, scale: &Scale) -> Self {
        Self {
            x: self.x * scale.x,
            y: self.y * scale.y,
        }
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SpriteRectangle {
    pub center: SpriteCoordinate,
    pub size: Size,
}

impl SpriteRectangle {
    pub fn contains(&self, coordinate: &SpriteCoordinate) -> bool {
        let top_left = self.top_left();
        coordinate.x >= top_left.x
            && coordinate.y >= top_left.y
            && coordinate.x <= top_left.x + self.size.width
            && coordinate.y <= top_left.y + self.size.length
    }

    fn top_left(&self) -> SpriteCoordinate {
        self.center.add(&SpriteCoordinate {
            x: self.size.width / -2.0,
            y: self.size.length / -2.0,
        })
    }

    fn bottom_right(&self) -> SpriteCoordinate {
        self.center.add(&SpriteCoordinate {
            x: self.size.width / 2.0,
            y: self.size.length / 2.0,
        })
    }

    pub fn intersects(&self, other: &SpriteRectangle) -> bool {
        let self_top_left = self.top_left();
        let self_bottom_right = self.bottom_right();
        let other_top_left = other.top_left();
        let other_bottom_right = other.bottom_right();
        !(self_top_left.x > other_bottom_right.x
            || self_bottom_right.x < other_top_left.x
            || self_top_left.y > other_bottom_right.y
            || self_bottom_right.y < other_top_left.y)
    }

    #[allow(dead_code)]
    pub fn translate(&self, coordinate: &SpriteCoordinate) -> SpriteRectangle {
        SpriteRectangle {
            center: self.center.add(coordinate),
            size: self.size,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CanvasRectangle {
    pub top_left: CanvasCoordinate,
    pub size: Size,
}

impl CanvasRectangle {
    pub fn translate(&self, coordinate: &CanvasCoordinate) -> CanvasRectangle {
        CanvasRectangle {
            top_left: self.top_left.add(coordinate),
            size: self.size,
        }
    }

    pub fn scale(&self, scale: &Scale) -> CanvasRectangle {
        CanvasRectangle {
            top_left: self.top_left,
            size: self.size.multiply(scale),
        }
    }

    pub fn contains(&self, coordinate: &CanvasCoordinate) -> bool {
        coordinate.x >= self.top_left.x
            && coordinate.y >= self.top_left.y
            && coordinate.x <= self.top_left.x + self.size.width
            && coordinate.y <= self.top_left.y + self.size.length
    }
}

impl From<SpriteRectangle> for CanvasRectangle {
    fn from(s: SpriteRectangle) -> Self {
        Self {
            top_left: s.center.into(),
            size: s.size,
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Transformation {
    pub translate: CanvasCoordinate,
    pub scale: Scale,
}

impl Transformation {
    pub fn translate(translate: CanvasCoordinate) -> Self {
        Self {
            translate,
            scale: Scale::default(),
        }
    }

    #[allow(dead_code)]
    pub fn scale(scale: Scale) -> Self {
        Self {
            translate: CanvasCoordinate::default(),
            scale,
        }
    }

    pub fn apply_transformation(&self, other: &Transformation) -> Self {
        Self {
            translate: self.translate.add(&other.translate),
            scale: self.scale.multiply(&other.scale),
        }
    }

    pub fn apply_to_coordinate(&self, coordinate: &CanvasCoordinate) -> CanvasCoordinate {
        coordinate.add(&self.translate).scale(&self.scale)
    }

    pub fn apply_to_rectangle(&self, rectangle: &CanvasRectangle) -> CanvasRectangle {
        rectangle.translate(&self.translate).scale(&self.scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod coordinate {
        use super::*;

        #[test]
        fn conversion() {
            assert_eq!(
                CanvasCoordinate::from(SpriteCoordinate { x: 0.0, y: 0.0 }),
                CanvasCoordinate { x: 240.0, y: 180.0 }
            );
            assert_eq!(
                SpriteCoordinate::from(CanvasCoordinate { x: 240.0, y: 180.0 }),
                SpriteCoordinate { x: 0.0, y: 0.0 }
            );
            assert_eq!(
                CanvasCoordinate::from(SpriteCoordinate {
                    x: -240.0,
                    y: 180.0
                }),
                CanvasCoordinate { x: 0.0, y: 0.0 }
            );
            assert_eq!(
                SpriteCoordinate::from(CanvasCoordinate { x: 0.0, y: 0.0 }),
                SpriteCoordinate {
                    x: -240.0,
                    y: 180.0,
                }
            );
        }
    }

    mod sprite_rectangle {
        use super::*;

        #[test]
        fn test_contains() {
            struct Test {
                rect: SpriteRectangle,
                coordinate: SpriteCoordinate,
                expected: bool,
            }

            let tests = vec![
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 0.0,
                            length: 0.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: 0.0, y: 0.0 },
                    expected: true,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 1.0,
                            length: 1.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: 0.0, y: 0.0 },
                    expected: true,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 2.0,
                            length: 2.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: 1.0, y: 1.0 },
                    expected: true,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 1.0,
                            length: 1.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: 1.0, y: 1.0 },
                    expected: false,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 1.0,
                            length: 1.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: -1.0, y: -1.0 },
                    expected: false,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 1.0,
                            length: 1.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: -2.0, y: 0.0 },
                    expected: false,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 1.0, y: 1.0 },
                        size: Size {
                            width: 1.0,
                            length: 1.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: 1.0, y: 0.0 },
                    expected: false,
                },
                Test {
                    rect: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 1.0,
                            length: 1.0,
                        },
                    },
                    coordinate: SpriteCoordinate { x: 1.0, y: 2.0 },
                    expected: false,
                },
            ];

            for (i, test) in tests.iter().enumerate() {
                assert_eq!(test.rect.contains(&test.coordinate), test.expected, "{}", i);
            }
        }

        #[test]
        fn intersects() {
            struct Test {
                a: SpriteRectangle,
                b: SpriteRectangle,
                expected: bool,
            }

            let tests = vec![
                Test {
                    a: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 0.0,
                            length: 0.0,
                        },
                    },
                    b: SpriteRectangle {
                        center: SpriteCoordinate { x: 0.0, y: 0.0 },
                        size: Size {
                            width: 0.0,
                            length: 0.0,
                        },
                    },
                    expected: true,
                },
                Test {
                    a: SpriteRectangle {
                        center: SpriteCoordinate { x: 1.0, y: 1.0 },
                        size: Size {
                            width: 2.0,
                            length: 2.0,
                        },
                    },
                    b: SpriteRectangle {
                        center: SpriteCoordinate { x: 1.0, y: 1.0 },
                        size: Size {
                            width: 2.0,
                            length: 2.0,
                        },
                    },
                    expected: true,
                },
                Test {
                    a: SpriteRectangle {
                        center: SpriteCoordinate { x: 1.0, y: 1.0 },
                        size: Size {
                            width: 2.0,
                            length: 2.0,
                        },
                    },
                    b: SpriteRectangle {
                        center: SpriteCoordinate { x: 2.0, y: 2.0 },
                        size: Size {
                            width: 2.0,
                            length: 2.0,
                        },
                    },
                    expected: true,
                },
            ];

            for (i, test) in tests.iter().enumerate() {
                assert_eq!(test.a.intersects(&test.b), test.expected, "{}", i);
            }
        }
    }
}
