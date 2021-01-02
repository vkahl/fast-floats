//! Experimental (unstable) “fast-math” wrappers for f32, f64
//!
//! These wrappers enable the [“fast-math”][1] flags for the operations
//! where there are intrinsics for this (add, sub, mul, div, rem).
//! The wrappers exist so that we have a quick & easy way **to experiment**
//! with fast math flags and further that feature in Rust.
//!
//! Note that as of this writing, the Rust instrinsics use the “fast” flag
//! documented in the langref; this enables all the float flags.
//!
//! [1]: http://llvm.org/docs/LangRef.html#fast-math-flags
//!
//! # Rust Version
//!
//! This crate is nightly only and experimental. Breaking changes can occur at
//! any time, if changes in Rust require it.
#![feature(core_intrinsics)]

use std::{
    intrinsics::{fadd_fast, fdiv_fast, fmul_fast, frem_fast, fsub_fast},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign, Neg},
};

use num_derive::{Float, One, Zero, Num, NumCast, ToPrimitive, FromPrimitive};

macro_rules! float_wrapper {
    ($name: ident ($t: ty)) => {
        #[derive(Copy, Clone, PartialEq, PartialOrd, Default, ToPrimitive, FromPrimitive, Num, NumCast, Zero, One, Float)]
        #[repr(transparent)]
        pub struct $name(pub $t);

        impl $name {
            /// Get the inner value
            #[inline(always)]
            pub fn get(self) -> $t {
                self.0
            }
        }

        impl From<$t> for $name {
            #[inline(always)]
            fn from(other: $t) -> Self {
                $name(other)
            }
        }

        impl From<$name> for $t {
            #[inline(always)]
            fn from(other: $name) -> $t {
                other.get()
            }
        }

        impl Neg for $name {
            type Output = $name;
        
            #[inline(always)]
            fn neg(self) -> Self::Output {
                self.0.neg().into()
            }
        }

        impl $name {
            #[inline(always)]
            pub fn round(self) -> Self {
                self.0.round().into()
            }
        }
    };
}

float_wrapper! { FF32(f32) }
float_wrapper! { FF64(f64) }

macro_rules! impl_op {
    ($($name:ident, $method:ident, $intrins:ident;)*) => {
        $(
            impl $name<f32> for FF32 {
                type Output = Self;
                #[inline(always)]
                fn $method(self, rhs: f32) -> Self::Output {
                    unsafe {
                        FF32($intrins(self.0, rhs))
                    }
                }
            }

            impl $name<FF32> for f32 {
                type Output = FF32;
                #[inline(always)]
                fn $method(self, rhs: FF32) -> Self::Output {
                    FF32(self).$method(rhs.0)
                }
            }

            impl $name for FF32 {
                type Output = Self;
                #[inline(always)]
                fn $method(self, rhs: Self) -> Self::Output {
                    self.$method(rhs.0)
                }
            }

            impl $name<f64> for FF64 {
                type Output = Self;
                #[inline(always)]
                fn $method(self, rhs: f64) -> Self::Output {
                    unsafe {
                        FF64($intrins(self.0, rhs))
                    }
                }
            }

            impl $name<FF64> for f64 {
                type Output = FF64;
                #[inline(always)]
                fn $method(self, rhs: FF64) -> Self::Output {
                    FF64(self).$method(rhs.0)
                }
            }

            impl $name for FF64 {
                type Output = Self;
                #[inline(always)]
                fn $method(self, rhs: Self) -> Self::Output {
                    self.$method(rhs.0)
                }
            }
        )*

    }
}

macro_rules! impl_assignop {
    ($($name:ident, $method:ident, $intrins:ident;)*) => {
        $(
            impl<Rhs> $name<Rhs> for FF32
                where Self: Add<Rhs, Output=Self> + Copy,
            {
                #[inline(always)]
                fn $method(&mut self, rhs: Rhs) {
                    *self = *self + rhs
                }
            }

            impl<Rhs> $name<Rhs> for FF64
            where Self: Add<Rhs, Output=Self> + Copy,
        {
            #[inline(always)]
            fn $method(&mut self, rhs: Rhs) {
                *self = *self + rhs
            }
        }
        )*
    }
}

impl_op! {
    Add, add, fadd_fast;
    Sub, sub, fsub_fast;
    Mul, mul, fmul_fast;
    Div, div, fdiv_fast;
    Rem, rem, frem_fast;
}

impl_assignop! {
    AddAssign, add_assign, fadd_fast;
    SubAssign, sub_assign, fsub_fast;
    MulAssign, mul_assign, fmul_fast;
    DivAssign, div_assign, fdiv_fast;
    RemAssign, rem_assign, frem_fast;
}

use std::fmt;
macro_rules! impl_format {
    ($($name:ident)+) => {
        $(
            impl fmt::$name for FF32 {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    self.0.fmt(f)
                }
            }

            impl fmt::$name for FF64 {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    self.0.fmt(f)
                }
            }
        )+
    }
}

impl_format!(Debug Display LowerExp UpperExp);

#[allow(unused)]
#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_op {
        ($($op:tt)+) => {
            $(
                assert_eq!(FF32(2.) $op FF32(1.), FF32(2. $op 1.));
            )+
        }
    }

    #[test]
    fn each_op() {
        test_op!(+ - * / %);
    }

    // for demonstration purposes
    fn fast_sum(xs: &[f64]) -> f64 {
        xs.iter()
            .map(|&x| FF64(x))
            .fold(FF64(0.), |acc, x| acc + x)
            .get()
    }

    // for demonstration purposes
    fn fast_dot(xs: &[f64], ys: &[f64]) -> f64 {
        xs.iter()
            .zip(ys)
            .fold(FF64(0.), |acc, (&x, &y)| acc + FF64(x) * FF64(y))
            .get()
    }

    fn regular_sum(xs: &[f64]) -> f64 {
        xs.iter().map(|&x| x).fold(0., |acc, x| acc + x)
    }
}
