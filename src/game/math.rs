use std::ops;

#[derive(Copy)]
pub struct Rectf {
    pub min: V2f,
    pub max: V2f,
}

impl Rectf {
    pub fn get_min(&self) -> V2f {
        self.min
    }

    pub fn get_max(&self) -> V2f {
        self.max
    }

    pub fn get_center(&self) -> V2f {
        (self.min + self.max) * 0.5
    }


    pub fn new(min: V2f, max: V2f) -> Rectf {
        Rectf {
            min: min,
            max: max,
        }
    }

    pub fn min_dim(min: V2f, dim: V2f) -> Rectf {
        Rectf {
            min: min,
            max: min + dim,
        }
    }

    pub fn center_dim(center: V2f, dim: V2f) -> Rectf {
        Rectf {
            min: center - dim * 0.5,
            max: center + dim * 0.5,
        }
    }

    pub fn p_inside(&self, p: V2f) -> bool {
        p.x >= self.min.x &&
        p.y >= self.min.y &&
        p.x < self.max.x &&
        p.y < self.max.y
    }
}


#[derive(Copy, Default, PartialEq)]
pub struct V2f { 
    pub x: f32,
    pub y: f32,

}

impl V2f {
    pub fn length_sq(&self) -> f32 {
        dot(*self, *self)
    }

    pub fn length(&self) -> f32 {
        let val = self.length_sq();
        val.sqrt()
    }

    pub fn normalize(&self) -> V2f {
        let length = self.length();
        V2f {
            x: self.x / length,
            y: self.y / length,
        }
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
