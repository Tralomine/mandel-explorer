use std::ops::*;

pub trait Op
where
    Self: Add<Output=Self>,
    Self: AddAssign,
    Self: Sub<Output=Self>,
    Self: SubAssign,
    Self: Mul<Output=Self>,
    Self: MulAssign,
    Self: Div<Output=Self>,
    Self: DivAssign,
    Self: Neg<Output=Self>,
    Self: Copy,
{}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Vec2<T: Op>
{
    pub x: T,
    pub y: T,
}

impl<T: Op> std::ops::Add<Vec2<T>> for Vec2<T> {
    fn add(self, rhx: Self) -> Self {
        Vec2{x:self.x+rhx.x, y:self.y+rhx.y}
    }
    type Output = Vec2<T>;
}

impl<T: Op> std::ops::AddAssign<Vec2<T>> for Vec2<T> {
    fn add_assign(&mut self, rhs: Vec2<T>) {
        self.x+=rhs.x;
        self.y+=rhs.y;
    }
}

impl<T: Op> std::ops::Neg for Vec2<T> {
    fn neg(self) -> Self::Output {
        Vec2{x:-self.x, y:-self.y}
    }
    type Output = Vec2<T>;
}

impl<T: Op> std::ops::Sub<Vec2<T>> for Vec2<T> {
    fn sub(self, rhx: Self) -> Self {
        self + - rhx
    }
    type Output = Vec2<T>;
}

impl<T: Op> std::ops::SubAssign<Vec2<T>> for Vec2<T> {
    fn sub_assign(&mut self, rhs: Vec2<T>) {
        *self += -rhs
    }
}

impl<T: Op> std::ops::Mul<T> for Vec2<T> {
    fn mul(self, rhx: T) -> Self {
        Vec2{x:self.x*rhx, y:self.y*rhx}
    }
    type Output = Vec2<T>;
}

impl<T: Op> std::ops::MulAssign<T> for Vec2<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<T: Op> std::ops::Div<T> for Vec2<T> {
    fn div(self, rhx: T) -> Self {
        Vec2{x:self.x/rhx, y:self.y/rhx}
    }
    type Output = Vec2<T>;
}

impl<T: Op> std::ops::DivAssign<T> for Vec2<T> {
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
