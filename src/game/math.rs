use std::ops;
use num;

#[derive(Copy, Clone)]
pub struct Rect<T> {
    pub min: V2<T>,
    pub max: V2<T>,
}

impl<T> Rect<T> where T: Copy
{
    pub fn get_min(&self) -> V2<T> {
        self.min
    }

    pub fn get_max(&self) -> V2<T> {
        self.max
    }

    #[allow(dead_code)]
    pub fn new(min: V2<T>, max: V2<T>) -> Rect<T> {
        Rect::<T> {
            min: min,
            max: max,
        }
    }
}

impl<T> Rect<T>
    where V2<T>: ops::Add<Output = V2<T>>,
          T: Copy
{
    #[allow(dead_code)]
    pub fn min_dim(min: V2<T>, dim: V2<T>) -> Rect<T> {
        Rect::<T> {
            min: min,
            max: min + dim,
        }
    }
}

impl<T> Rect<T> where T: num::Float
{
    pub fn center_dim(center: V2<T>, dim: V2<T>) -> Rect<T> {
        Rect::<T> {
            min: center - dim * num::traits::cast(0.5).unwrap(),
            max: center + dim * num::traits::cast(0.5).unwrap(),
        }
    }

    #[allow(dead_code)]
    pub fn get_center(&self) -> V2<T> {
        (self.min + self.max) * num::traits::cast(0.5).unwrap()
    }
}

impl<T> Rect<T> where T: PartialOrd
{
    pub fn p_inside(&self, p: V2<T>) -> bool {
        p.x >= self.min.x && p.y >= self.min.y && p.x < self.max.x && p.y < self.max.y
    }
}

#[derive(Copy, Clone, Default, PartialEq)]
pub struct V4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T> V4<T> where T: Copy
{
    #[allow(dead_code)]
    pub fn xyz(&self) -> V3<T> {
        V3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl<T> V4<T> where T: Copy + ops::Add<Output = T> + ops::Mul<Output = T>
{
    #[allow(dead_code)]
    pub fn length_sq(&self) -> T {
        dot_4(*self, *self)
    }
}

impl<T> V4<T> where T: num::Float
{
    #[allow(dead_code)]
    pub fn length(&self) -> T {
        let val = self.length_sq();
        val.sqrt()
    }

    #[allow(dead_code)]
    pub fn normalize(&self) -> V4<T> {
        let length = self.length();
        V4 {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
            w: self.w / length,
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq)]
pub struct V3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> V3<T> where T: Copy + ops::Add<Output = T> + ops::Mul<Output = T>
{
    #[allow(dead_code)]
    pub fn length_sq(&self) -> T {
        dot_3(*self, *self)
    }
}

impl<T> V3<T> where T: num::Float
{
    #[allow(dead_code)]
    pub fn length(&self) -> T {
        let val = self.length_sq();
        val.sqrt()
    }

    #[allow(dead_code)]
    pub fn normalize(&self) -> V3<T> {
        let length = self.length();
        V3 {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }
}

impl<T> ops::Mul<T> for V3<T> where T: Copy + ops::Mul<Output = T>
{
    type Output = V3<T>;

    fn mul(self, other: T) -> V3<T> {
        V3::<T> {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}


impl<T> ops::Add<V3<T>> for V3<T> where T: ops::Add<Output = T>
{
    type Output = V3<T>;

    fn add(self, other: V3<T>) -> V3<T> {
        V3::<T> {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl<T> ops::Sub<V3<T>> for V3<T> where T: ops::Sub<Output = T>
{
    type Output = V3<T>;

    fn sub(self, other: V3<T>) -> V3<T> {
        V3::<T> {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}


impl<T> ops::Neg for V3<T> where T: ops::Neg<Output = T>
{
    type Output = V3<T>;

    fn neg(self) -> V3<T> {
        V3::<T> {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq)]
pub struct V2<T> {
    pub x: T,
    pub y: T,
}

impl<T> V2<T> where T: Copy + ops::Add<Output = T> + ops::Mul<Output = T>
{
    pub fn length_sq(&self) -> T {
        dot_2(*self, *self)
    }
}

impl<T> V2<T> where T: num::Float
{
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

#[allow(dead_code)]
pub fn dot_4<T>(a: V4<T>, b: V4<T>) -> T
    where T: ops::Add<Output = T> + ops::Mul<Output = T>
{
    a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
}

#[allow(dead_code)]
pub fn dot_3<T>(a: V3<T>, b: V3<T>) -> T
    where T: ops::Add<Output = T> + ops::Mul<Output = T>
{
    a.x * b.x + a.y * b.y + a.z * b.z
}

pub fn dot_2<T>(a: V2<T>, b: V2<T>) -> T
    where T: ops::Add<Output = T> + ops::Mul<Output = T>
{
    a.x * b.x + a.y * b.y
}

impl<T> ops::Mul<T> for V2<T> where T: Copy + ops::Mul<Output = T>
{
    type Output = V2<T>;

    fn mul(self, other: T) -> V2<T> {
        V2::<T> {
            x: self.x * other,
            y: self.y * other,
        }
    }
}


impl<T> ops::Add<V2<T>> for V2<T> where T: ops::Add<Output = T>
{
    type Output = V2<T>;

    fn add(self, other: V2<T>) -> V2<T> {
        V2::<T> {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T> ops::Sub<V2<T>> for V2<T> where T: ops::Sub<Output = T>
{
    type Output = V2<T>;

    fn sub(self, other: V2<T>) -> V2<T> {
        V2::<T> {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}


impl<T> ops::Neg for V2<T> where T: ops::Neg<Output = T>
{
    type Output = V2<T>;

    fn neg(self) -> V2<T> {
        V2::<T> {
            x: -self.x,
            y: -self.y,
        }
    }
}
