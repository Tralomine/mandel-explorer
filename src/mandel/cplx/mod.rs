#[derive(Copy, Clone)]
pub struct Cplx<T> {
    pub re: T,
    pub im: T,
}

impl<T: Copy + std::ops::Add<Output = T> + std::ops::Mul<Output = T>> Cplx<T> {
    pub fn sq_abs(&self) -> T {
        self.re * self.re + self.im * self.im
    }
}

impl<
        T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Mul<Output = T>,
    > std::ops::Mul<Cplx<T>> for Cplx<T>
{
    fn mul(self, rhs: Cplx<T>) -> Self::Output {
        Cplx {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.re * rhs.im,
        }
    }
    type Output = Cplx<T>;
}

impl<T: Copy + std::ops::Mul<Output = T>> std::ops::Mul<T> for Cplx<T>
{
    fn mul(self, rhs: T) -> Self::Output {
        Cplx {
            re: self.re * rhs,
            im: self.re * rhs,
        }
    }
    type Output = Cplx<T>;
}

impl<
        T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Mul<Output = T> + std::ops::Div<Output = T>,
    > std::ops::Div<Cplx<T>> for Cplx<T>
{
    fn div(self, rhs: Cplx<T>) -> Self::Output {
        // a+bi / c+di == (a+bi)(c-di)/(c+di)(c-di) == ac+bd+(bc-ad)i/cc+dd
        let ccdd = rhs.re*rhs.re+rhs.im*rhs.im;
        Cplx {
            re: (self.re * rhs.re + self.im * rhs.im)/ccdd,
            im: (self.im * rhs.re - self.re * rhs.im )/ccdd,
        }
    }
    type Output = Cplx<T>;
}


impl<T: Copy + std::ops::Div<Output = T>> std::ops::Div<T> for Cplx<T>
{
    fn div(self, rhs: T) -> Self::Output {
        Cplx {
            re: self.re / rhs,
            im: self.re / rhs,
        }
    }
    type Output = Cplx<T>;
}


impl<T: Copy + std::ops::Add<Output = T>> std::ops::Add<Cplx<T>> for Cplx<T> {
    fn add(self, rhs: Cplx<T>) -> Self::Output {
        Cplx {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
    type Output = Cplx<T>;
}

impl<T: Copy + std::ops::Neg<Output = T> + std::ops::Add<Output = T>> std::ops::Sub<Cplx<T>>
    for Cplx<T>
{
    fn sub(self, rhs: Cplx<T>) -> Self::Output {
        self + (-rhs)
    }
    type Output = Cplx<T>;
}

impl<T: Copy + std::ops::Neg<Output = T>> std::ops::Neg for Cplx<T> {
    fn neg(self) -> Self::Output {
        Cplx {
            re: -self.re,
            im: -self.im,
        }
    }
    type Output = Cplx<T>;
}

impl Cplx<f64> {
    pub fn square(&self) -> Cplx<f64> {
        // re*re-im*im, 2*re*im
        Cplx {
            re: self.re * self.re - self.im * self.im,
            im: 2. * self.re * self.im,
        }
    }
    pub fn abs(&self) -> f64 {
        self.sq_abs().sqrt()
    }
}
