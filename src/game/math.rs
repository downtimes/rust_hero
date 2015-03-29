use std::ops;

#[derive(Copy)]
pub struct V2f { 
    pub x: f32,
    pub y: f32,
}

impl ops::Mul<f32> for V2f {
    type Output = V2f;

    fn mul(self, other: f32) -> V2f {
        V2f {
            x: self.x * other, 
            y: self.y * other
        }
    }
}

impl ops::Sub<f32> for V2f {
    type Output = V2f;

    fn sub(self, other: f32) -> V2f {
        V2f {
            x: self.x - other, 
            y: self.y - other
        }
    }
}


impl ops::Add<f32> for V2f {
    type Output = V2f;

    fn add(self, other: f32) -> V2f {
        V2f {
            x: self.x + other, 
            y: self.y + other
        }
    }
}

impl ops::Add for V2f {
    type Output = V2f;

    fn add(self, other: V2f) -> V2f {
        V2f {
            x: self.x + other.x, 
            y: self.y + other.y
        }
    }
}

impl ops::Sub for V2f {
    type Output = V2f;

    fn sub(self, other: V2f) -> V2f {
        V2f {
            x: self.x - other.x, 
            y: self.y - other.y
        }
    }
}


impl ops::Neg for V2f {
    type Output = V2f;

    fn neg(self) -> V2f {
        V2f {
            x: -self.x, 
            y: -self.y
        }
    }
}
