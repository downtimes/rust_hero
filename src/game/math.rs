use std::ops;
use std::num;

#[derive(Copy)]
pub struct Rect<T> {
    pub min: V2<T>,
    pub max: V2<T>,
}

impl<T> Rect<T> where T: Copy {
    pub fn get_min(&self) -> V2<T> {
        self.min
    }

    pub fn get_max(&self) -> V2<T> {
        self.max
    }

    pub fn new(min: V2<T>, max: V2<T>) -> Rect<T> {
        Rect::<T> {
            min: min,
            max: max,
        }
    }
}

impl<T> Rect<T> where V2<T>: ops::Add<Output=V2<T>>,
                      T: Copy {
    pub fn min_dim(min: V2<T>, dim: V2<T>) -> Rect<T> {
        Rect::<T> {
            min: min,
            max: min + dim,
        }
    }
}

impl Rect<f32> {
   pub fn center_dim(center: V2<f32>, dim: V2<f32>) -> Rect<f32> {
       Rect::<f32> {
           min: center - dim * 0.5,
           max: center + dim * 0.5,
       }
   }

    pub fn get_center(&self) -> V2<f32> {
        (self.min + self.max) * 0.5
    }
}

impl<T> Rect<T> where T: PartialOrd {
    pub fn p_inside(&self, p: V2<T>) -> bool {
        p.x >= self.min.x &&
        p.y >= self.min.y &&
        p.x < self.max.x &&
        p.y < self.max.y
    }
}

#[derive(Copy, Default, PartialEq)]
pub struct V2<T> { 
    pub x: T,
    pub y: T,

}

impl<T> V2<T> where T: Copy + ops::Add<Output=T> + ops::Mul<Output=T> {
    pub fn length_sq(&self) -> T {
        dot(*self, *self)
    }

}

impl<T> V2<T> where T: num::Float {
    pub fn length(&self) -> T {
        let val = self.length_sq();
        val.sqrt()
    }

    pub fn normalize(&self) -> V2<T> {
        let length = self.length();
        V2 {
            x: self.x / length,
            y: self.y / length,
        }
    }
}

pub fn dot<T>(a: V2<T>, b: V2<T>) -> T where T: ops::Add<Output=T>
                                                + ops::Mul<Output=T> {
    a.x * b.x + a.y * b.y
}

impl<T> ops::Mul<T> for V2<T> where T: Copy + ops::Mul<Output=T> {
    type Output = V2<T>;

    fn mul(self, other: T) -> V2<T> {
        V2::<T> {
            x: self.x * other, 
            y: self.y * other
        }
    }
}


impl<T> ops::Add<V2<T>> for V2<T> where T: ops::Add<Output=T>{
    type Output = V2<T>;

    fn add(self, other: V2<T>) -> V2<T> {
        V2::<T> {
            x: self.x + other.x, 
            y: self.y + other.y
        }
    }
}

impl<T> ops::Sub<V2<T>> for V2<T> where T: ops::Sub<Output=T> {
    type Output = V2<T>;

    fn sub(self, other: V2<T>) -> V2<T> {
        V2::<T> {
            x: self.x - other.x, 
            y: self.y - other.y
        }
    }
}


impl<T> ops::Neg for V2<T> where T: ops::Neg<Output=T> {
    type Output = V2<T>;

    fn neg(self) -> V2<T> {
        V2::<T> {
            x: -self.x, 
            y: -self.y
        }
    }
}
