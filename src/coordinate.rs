#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

/// Center = 0, 0
/// Left = -x, right = +x
/// Top = -y, bottom = +y
pub type SpriteCoordinate = Coordinate;

/// Left = 0, right = +x
/// Top = 0, bottom + y
pub type CanvasCoordinate = Coordinate;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size {
    pub width: f64,
    pub length: f64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rectangle {
    center: SpriteCoordinate,
    size: Size,
}

impl Rectangle {
    pub fn with_center(center: SpriteCoordinate, size: Size) -> Self {
        Self { center, size }
    }

    pub fn center(&self) -> SpriteCoordinate {
        self.center
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn contains(&self, coordinate: &SpriteCoordinate) -> bool {
        let top_left = self.center.add(&SpriteCoordinate {
            x: self.size.width / -2.0,
            y: self.size.length / -2.0,
        });
        coordinate.x >= top_left.x
            && coordinate.y >= top_left.y
            && coordinate.x <= top_left.x + self.size.width
            && coordinate.y <= top_left.y + self.size.length
    }

    pub fn translate(&self, coordinate: &SpriteCoordinate) -> Rectangle {
        Rectangle {
            center: self.center.add(coordinate),
            size: self.size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        struct Test {
            rect: Rectangle,
            coordinate: SpriteCoordinate,
            expected: bool,
        }

        let tests = vec![
            Test {
                rect: Rectangle::with_center(
                    SpriteCoordinate { x: 0.0, y: 0.0 },
                    Size {
                        width: 0.0,
                        length: 0.0,
                    },
                ),
                coordinate: SpriteCoordinate { x: 0.0, y: 0.0 },
                expected: true,
            },
            Test {
                rect: Rectangle::with_center(
                    SpriteCoordinate { x: 0.0, y: 0.0 },
                    Size {
                        width: 1.0,
                        length: 1.0,
                    },
                ),
                coordinate: SpriteCoordinate { x: 0.0, y: 0.0 },
                expected: true,
            },
            Test {
                rect: Rectangle::with_center(
                    SpriteCoordinate { x: 0.0, y: 0.0 },
                    Size {
                        width: 2.0,
                        length: 2.0,
                    },
                ),
                coordinate: SpriteCoordinate { x: 1.0, y: 1.0 },
                expected: true,
            },
            Test {
                rect: Rectangle::with_center(
                    SpriteCoordinate { x: 0.0, y: 0.0 },
                    Size {
                        width: 1.0,
                        length: 1.0,
                    },
                ),
                coordinate: SpriteCoordinate { x: 1.0, y: 1.0 },
                expected: false,
            },
            Test {
                rect: Rectangle::with_center(
                    SpriteCoordinate { x: 0.0, y: 0.0 },
                    Size {
                        width: 1.0,
                        length: 1.0,
                    },
                ),
                coordinate: SpriteCoordinate { x: -1.0, y: -1.0 },
                expected: false,
            },
            Test {
                rect: Rectangle::with_center(
                    SpriteCoordinate { x: 0.0, y: 0.0 },
                    Size {
                        width: 1.0,
                        length: 1.0,
                    },
                ),
                coordinate: SpriteCoordinate { x: -2.0, y: 0.0 },
                expected: false,
            },
            Test {
                rect: Rectangle::with_center(
                    SpriteCoordinate { x: 1.0, y: 1.0 },
                    Size {
                        width: 1.0,
                        length: 1.0,
                    },
                ),
                coordinate: SpriteCoordinate { x: 1.0, y: 0.0 },
                expected: false,
            },
            Test {
                rect: Rectangle::with_center(
                    SpriteCoordinate { x: 0.0, y: 0.0 },
                    Size {
                        width: 1.0,
                        length: 1.0,
                    },
                ),
                coordinate: SpriteCoordinate { x: 1.0, y: 2.0 },
                expected: false,
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            assert_eq!(test.rect.contains(&test.coordinate), test.expected, "{}", i);
        }
    }
}