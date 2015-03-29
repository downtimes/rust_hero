use std::ops;

#[derive(Copy)]
pub struct V2f { 
    pub x: f32,
    pub y: f32,

}

impl V2f {
    pub fn length_sq(&self) -> f32 {
        dot(*self, *self)
    }
}

pub fn dot(a: V2f, b: V2f) -> f32 {
    a.x * b.x + a.y * b.y
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
