/*
Adapted from: https://gitlab.com/polwel/egui-colorgradient/-/tree/master

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
*/

use bevy::color::Color;
use bevy_egui::egui::emath::Float;
use bevy_egui::egui::epaint::util::OrderedFloat;
use bevy_egui::egui::epaint::{Color32, Hsva, Rgba};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

/// The method used for interpolating between two points
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InterpolationMethod {
    /// Use the nearest value to the left of the sample point. If there is no key point to the left
    /// of the sample, use the nearest point on the _right_ instead.
    Constant,
    /// Linearly interpolate between the two stops to the left and right of the sample. If the sample
    /// is outside the range of the stops, use the value of the single nearest stop.
    Linear,
}

impl Display for InterpolationMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Linear => "linear",
                Self::Constant => "constant",
            }
        )
    }
}

/// A ColorInterpolator can arbitrarily sample a gradient.
pub struct ColorInterpolator {
    method: InterpolationMethod,
    keys: Vec<(f32, Rgba)>,
}

impl ColorInterpolator {
    fn new(
        keys: impl IntoIterator<Item = (f32, impl Into<Rgba>)>,
        method: InterpolationMethod,
    ) -> Self {
        let keys: Vec<_> = keys.into_iter().map(|(k, v)| (k, v.into())).collect();
        let mut result = Self { keys, method };
        result.sort();
        result
    }

    fn sort(&mut self) {
        self.keys.sort_by_key(|(t, _)| t.ord());
        // self.keys.sort_by_key(|(t, _)| t.ord());
    }

    /// Find the insertion point for x to maintain order
    fn bisect(&self, x: f32) -> Option<usize> {
        let mut lo = 0;
        let mut hi = self.keys.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            match self.keys[mid].0.partial_cmp(&x)? {
                Ordering::Less => lo = mid + 1,
                Ordering::Equal => lo = mid + 1,
                Ordering::Greater => hi = mid,
            }
        }

        Some(lo)
    }

    /// Sample the gradient at the given position.
    ///
    /// Returns `None` if the gradient is empty.
    pub fn sample_at(&self, x: f32) -> Option<Rgba> {
        Some(match self.method {
            InterpolationMethod::Constant => {
                let insertion_point = self.bisect(x)?;
                match insertion_point {
                    0 => self.keys.first()?.1,
                    n => self.keys.get(n - 1)?.1,
                }
            }
            InterpolationMethod::Linear => {
                let insertion_point = self.bisect(x)?;
                match insertion_point {
                    0 => self.keys.first()?.1,
                    n if n == self.keys.len() => self.keys.last()?.1,
                    n => {
                        let (t0, c0) = *self.keys.get(n - 1)?;
                        let (t1, c1) = *self.keys.get(n)?;

                        c0 + (c1 + c0 * -1.0_f32) * ((x - t0) / (t1 - t0))
                    }
                }
            }
        })
    }
}

fn argsort_by_key<T, K, F>(data: &[T], mut f: F) -> Vec<usize>
where
    F: FnMut(&T) -> K,
    K: Ord,
{
    let mut indices = (0..data.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| f(&data[i]));
    indices
}

/// A color gradient, that will be interpolated between a number of fixed points, a.k.a. _stops_.
#[derive(Clone)]
pub struct Gradient {
    pub stops: Vec<(f32, Hsva)>,
    pub interpolation_method: InterpolationMethod,
}

impl Gradient {
    /// Create a new gradient from an iterator over key colors.
    pub fn new(
        interpolation_method: InterpolationMethod,
        stops: impl IntoIterator<Item = (f32, impl Into<Hsva>)>,
    ) -> Self {
        Self {
            interpolation_method,
            stops: stops.into_iter().map(|(k, v)| (k, v.into())).collect(),
        }
    }

    /// Create a [ColorInterpolator] to evaluate the gradient at any point.
    pub fn interpolator(&self) -> ColorInterpolator {
        ColorInterpolator::new(self.stops.iter().copied(), self.interpolation_method)
    }

    /// Create a [ColorInterpolator] that discards the alpha component of the color gradient and
    /// always produces an opaque color.
    pub fn interpolator_opaque(&self) -> ColorInterpolator {
        ColorInterpolator::new(
            self.stops.iter().map(|(t, c)| (*t, c.to_opaque())),
            self.interpolation_method,
        )
    }

    /// Produce a list of the indices of the gradient's stops that would place them in order.
    ///
    /// Use this to prepare for the upcoming reordering of the stops by [sort()](Gradient::sort).
    pub fn argsort(&self) -> Vec<usize> {
        argsort_by_key(&self.stops, |(t, _col)| t.ord())
    }

    /// Sort the gradient's stops by ascending position.
    pub fn sort(&mut self) {
        self.stops.sort_by_key(|(t, _col)| t.ord())
    }

    /// Return a vector of the gradient's color sampled on linearly spaced points between 0 and 1.
    ///
    /// The first and last samples correspond to the gradient's value at 0.0 and 1.0, respectively.
    ///
    /// This is useful for generating a texture.
    ///
    /// # Panics
    ///
    /// Will panic if the provided size `n` is smaller or equal to 1, or if the gradient is empty.
    pub fn linear_eval(&self, n: usize, opaque: bool) -> Vec<Color32> {
        let interp = match opaque {
            false => self.interpolator(),
            true => self.interpolator_opaque(),
        };
        (0..n)
            .map(|idx| (idx as f32) / (n - 1) as f32)
            .map(|t| interp.sample_at(t).unwrap().into())
            .collect()
    }
    pub fn linear_eval_bevy(&self, n: usize, opaque: bool) -> Vec<Color> {
        let interp = match opaque {
            false => self.interpolator(),
            true => self.interpolator_opaque(),
        };
        (0..n)
            .map(|idx| (idx as f32) / (n - 1) as f32)
            .map(|t| Color::from_egui(interp.sample_at(t).unwrap()))
            .collect()
    }
}

impl Default for Gradient {
    fn default() -> Self {
        Self {
            stops: vec![(0., Color32::BLACK.into()), (1., Color32::WHITE.into())],
            interpolation_method: InterpolationMethod::Linear,
        }
    }
}

trait from_egui {
    fn from_egui(color: bevy_egui::egui::Rgba) -> Self;
}




impl from_egui for bevy::color::Color {
    fn from_egui(color: bevy_egui::egui::Rgba) -> Self {
        Self::srgba(color.r(), color.g(), color.b(), color.a())
    }
}