//! Experimental (unstable) “fast-math” wrappers for f32, f64
//!
//! These wrappers enable the [“fast-math”][1] flags for the operations
//! where there are intrinsics for this (add, sub, mul, div, rem).
//! The wrappers exist so that we have a quick & easy way to experiment
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
#![no_std]
#![feature(core_intrinsics)]

#[cfg(feature = "num-traits")]
extern crate num_traits;

#[cfg(feature = "num-traits")]
use num_traits::Zero;

extern crate core as std;

use std::intrinsics::{fadd_fast, fsub_fast, fmul_fast, fdiv_fast, frem_fast};
use std::ops::{
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
};

/// “fast-math” wrapper for f32 and f64.
///
/// The `Fast` type enforces no invariant and can hold any f32, f64 values.
/// See crate docs for more details.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Default)]
pub struct Fast<F>(pub F);

impl<F> Fast<F> {
    /// Get the inner value
    pub fn get(self) -> F { self.0 }
}

impl<F> From<F> for Fast<F> {
    fn from(x: F) -> Self { Fast(x) }
}

// for demonstration purposes
#[cfg(test)]
pub fn fast_sum(xs: &[f64]) -> f64 {
    xs.iter().map(|&x| Fast(x)).fold(Fast(0.), |acc, x| acc + x).get()
}

// for demonstration purposes
#[cfg(test)]
pub fn fast_dot(xs: &[f64], ys: &[f64]) -> f64 {
    xs.iter().zip(ys).fold(Fast(0.), |acc, (&x, &y)| acc + Fast(x) * Fast(y)).get()
}

#[cfg(test)]
pub fn regular_sum(xs: &[f64]) -> f64 {
    xs.iter().map(|&x| x).fold(0., |acc, x| acc + x)
}

macro_rules! impl_op {
    ($($name:ident, $method:ident, $intrins:ident;)*) => {
        $(
        impl $name for Fast<f64> {
            type Output = Self;
            #[inline(always)]
            fn $method(self, rhs: Self) -> Self::Output {
                unsafe {
                    Fast($intrins(self.0, rhs.0))
                }
            }
        }

        impl $name for Fast<f32> {
            type Output = Self;
            #[inline(always)]
            fn $method(self, rhs: Self) -> Self::Output {
                unsafe {
                    Fast($intrins(self.0, rhs.0))
                }
            }
        }
        )*

    }
}

macro_rules! impl_assignop {
    ($($name:ident, $method:ident, $intrins:ident;)*) => {
        $(
        impl $name for Fast<f64> {
            #[inline(always)]
            fn $method(&mut self, rhs: Self) {
                *self = *self + rhs
            }
        }

        impl $name for Fast<f32> {
            #[inline(always)]
            fn $method(&mut self, rhs: Self) {
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

/*
impl<Z> Zero for Fast<Z> where Z: Zero {
    fn zero() -> Self { Fast(Z::zero()) }
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}
*/
#[cfg(feature = "num-traits")]
impl Zero for Fast<f64> {
    fn zero() -> Self { Fast(<_>::zero()) }
    fn is_zero(&self) -> bool { self.get().is_zero() }
}
#[cfg(feature = "num-traits")]
impl Zero for Fast<f32> {
    fn zero() -> Self { Fast(<_>::zero()) }
    fn is_zero(&self) -> bool { self.get().is_zero() }
}


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_op {
        ($($op:tt)+) => {
            $(
                assert_eq!(Fast(2.) $op Fast(1.), Fast(2. $op 1.));
            )+
        }
    }

    #[test]
    fn each_op() {
        test_op!(+ - * / %);
    }
}
