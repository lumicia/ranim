use super::{AnimationSpan, EvalDynamic};
use crate::animation::Evaluator;
use crate::items::vitem::DEFAULT_STROKE_WIDTH;
use crate::traits::{Empty, FillColor, Interpolatable, Partial, StrokeColor, StrokeWidth};
use crate::utils::rate_functions::smooth;
use log::warn;

// MARK: Creation

/// The requirement of [`Create`] and [`UnCreate`]
pub trait CreationRequirement: Clone + Partial + Empty + Interpolatable {}
impl<T: Clone + Partial + Empty + Interpolatable> CreationRequirement for T {}

/// The methods to create animations for `T` that satisfies [`CreationRequirement`]
pub trait CreationAnim<T: CreationRequirement + 'static> {
    /// Create a [`Create`] anim for `T`.
    fn create(self) -> AnimationSpan<T>;
    /// Create an [`UnCreate`] anim for `T`.
    fn uncreate(self) -> AnimationSpan<T>;
}

impl<T: CreationRequirement + 'static> CreationAnim<T> for T {
    fn create(self) -> AnimationSpan<T> {
        AnimationSpan::from_evaluator(Evaluator::new_dynamic(Create::new(self)))
            .with_rate_func(smooth)
    }
    fn uncreate(self) -> AnimationSpan<T> {
        AnimationSpan::from_evaluator(Evaluator::new_dynamic(UnCreate::new(self)))
            .with_rate_func(smooth)
    }
}

// MARK: Writing
/// The requirement of [`Write`] and [`Unwrite`]
pub trait WritingRequirement: CreationRequirement + StrokeWidth + StrokeColor + FillColor {}
impl<T: CreationRequirement + StrokeWidth + StrokeColor + FillColor> WritingRequirement for T {}

/// The methods to create animations for `T` that satisfies [`WritingRequirement`]
pub trait WritingAnim<T: WritingRequirement + 'static> {
    /// Create a [`Write`] anim for `T`.
    fn write(self) -> AnimationSpan<T>;
    /// Create a [`Unwrite`] anim for `T`.
    fn unwrite(self) -> AnimationSpan<T>;
}

impl<T: WritingRequirement + 'static> WritingAnim<T> for T {
    fn write(self) -> AnimationSpan<T> {
        AnimationSpan::from_evaluator(Evaluator::new_dynamic(Write::new(self)))
            .with_rate_func(smooth)
    }
    fn unwrite(self) -> AnimationSpan<T> {
        AnimationSpan::from_evaluator(Evaluator::new_dynamic(Unwrite::new(self)))
            .with_rate_func(smooth)
    }
}

// /// The Trait for `I: IntoIterator<Item = T>` where `T` satisfies [`WritingRequirement`]
// pub trait GroupWritingAnim<T: WritingRequirement + 'static> {
//     /// Create a [`Write`] anim for a group.
//     fn group_write(self, lag_ratio: f64) -> AnimationSpan<Group<T>>;
//     /// Create an [`Unwrite`] anim for a group.
//     fn group_unwrite(self, lag_ratio: f64) -> AnimationSpan<Group<T>>;
// }

// impl<T: WritingRequirement + 'static, I> GroupWritingAnim<T> for I
// where
//     I: IntoIterator<Item = T>,
// {
//     fn group_write(self, lag_ratio: f64) -> AnimationSpan<Group<T>> {
//         AnimationSpan::from_evaluator(Evaluator::new_dynamic(GroupWrite::new(self, lag_ratio)))
//             .with_rate_func(smooth)
//     }
//     fn group_unwrite(self, lag_ratio: f64) -> AnimationSpan<Group<T>> {
//         AnimationSpan::from_evaluator(Evaluator::new_dynamic(GroupWrite::new(self, lag_ratio)))
//             .with_rate_func(smooth)
//     }
// }

// ---------------------------------------------------- //

// MARK: Impl
/// The create anim.
///
/// This anim uses [`Partial::get_partial_closed`] to create the item
pub struct Create<T: CreationRequirement> {
    /// The original object
    pub original: T,
}

impl<T: CreationRequirement> Create<T> {
    /// Constructor
    pub fn new(target: T) -> Self {
        Self { original: target }
    }
}

impl<T: CreationRequirement> EvalDynamic<T> for Create<T> {
    fn eval_alpha(&self, alpha: f64) -> T {
        if alpha == 0.0 {
            T::empty()
        } else if 0.0 < alpha && alpha < 1.0 {
            self.original.get_partial_closed(0.0..alpha)
        } else if alpha == 1.0 {
            self.original.clone()
        } else {
            unreachable!()
        }
    }
}

/// The uncreate anim
///
/// This anim uses [`Partial::get_partial_closed`] to uncreate the item
pub struct UnCreate<T: CreationRequirement> {
    /// The original object
    pub original: T,
}

impl<T: CreationRequirement> UnCreate<T> {
    /// Constructor
    pub fn new(target: T) -> Self {
        Self { original: target }
    }
}

impl<T: CreationRequirement> EvalDynamic<T> for UnCreate<T> {
    fn eval_alpha(&self, mut alpha: f64) -> T {
        if !(0.0..=1.0).contains(&alpha) {
            warn!("the alpha is out of range: {alpha}, clampped to 0.0..=1.0");
            alpha = alpha.clamp(0.0, 1.0)
        }
        // trace!("{alpha}");
        if alpha == 0.0 {
            self.original.clone()
        } else if 0.0 < alpha && alpha < 1.0 {
            self.original.get_partial_closed(0.0..1.0 - alpha)
        } else if alpha == 1.0 {
            T::empty()
        } else {
            panic!("the alpha is out of range: {alpha}");
        }
    }
}

/// Write
///
/// First update with partial from 0.0..0.0 to 0.0..1.0, then lerp fill_opacity to 1.0
pub struct Write<T: WritingRequirement> {
    pub(crate) original: T,
    pub(crate) outline: T,
}

impl<T: WritingRequirement> Write<T> {
    /// Constructor
    pub fn new(target: T) -> Self {
        let mut outline = target.clone();
        outline
            .set_fill_opacity(0.0)
            .set_stroke_width(DEFAULT_STROKE_WIDTH)
            .set_stroke_opacity(1.0);
        Self {
            original: target,
            outline,
        }
    }
}

impl<T: WritingRequirement> EvalDynamic<T> for Write<T> {
    fn eval_alpha(&self, alpha: f64) -> T {
        let alpha = alpha * 2.0;
        if (0.0..1.0).contains(&alpha) {
            self.outline.get_partial(0.0..alpha)
        } else if alpha == 1.0 {
            self.outline.clone()
        } else if (1.0..2.0).contains(&alpha) {
            self.outline.lerp(&self.original, alpha - 1.0)
        } else if alpha == 2.0 {
            self.original.clone()
        } else {
            unreachable!()
        }
    }
}

// /// [`Write`] for a group of items.
// ///
// /// with a `lag_ratio`
// pub struct GroupWrite<T: WritingRequirement> {
//     anims: Vec<Write<T>>,
//     lag_ratio: f64,
// }

// impl<T: WritingRequirement> GroupWrite<T> {
//     /// Constructor
//     pub fn new<I>(target: I, lag_ratio: f64) -> Self
//     where
//         I: IntoIterator<Item = T>,
//     {
//         Self {
//             anims: target.into_iter().map(Write::new).collect(),
//             lag_ratio,
//         }
//     }
// }

// impl<T: WritingRequirement> EvalDynamic<Group<T>> for GroupWrite<T> {
//     fn eval_alpha(&self, alpha: f64) -> Group<T> {
//         // -|--
//         //  -|--
//         //   -|--
//         // total_time - unit_time * (1.0 - lag_ratio)  = unit_time * lag_ratio * n
//         // total_time = unit_time * (1.0 + (n - 1) lag_ratio)
//         let unit_time = 1.0 / (1.0 + (self.anims.len() - 1) as f64 * self.lag_ratio);
//         let unit_lagged_time = unit_time * self.lag_ratio;
//         self.anims
//             .iter()
//             .enumerate()
//             .map(|(i, anim)| {
//                 let start = unit_lagged_time * i as f64;

//                 let alpha = (alpha - start) / unit_time;
//                 let alpha = alpha.clamp(0.0, 1.0);
//                 anim.eval_alpha(alpha)
//             })
//             .collect()
//     }
// }

/// Unwrite
///
/// First lerp fill_opacity to 0.0, then update with partial from 0.0..1.0 to 0.0..0.0
pub struct Unwrite<T: WritingRequirement> {
    pub(crate) original: T,
    pub(crate) outline: T,
}

impl<T: WritingRequirement> Unwrite<T> {
    /// Constructor
    pub fn new(target: T) -> Self {
        let mut outline = target.clone();
        outline
            .set_fill_opacity(0.0)
            .set_stroke_width(DEFAULT_STROKE_WIDTH)
            .set_stroke_opacity(1.0);
        Self {
            original: target,
            outline,
        }
    }
}

impl<T: WritingRequirement> EvalDynamic<T> for Unwrite<T> {
    fn eval_alpha(&self, alpha: f64) -> T {
        let alpha = alpha * 2.0;
        if (0.0..1.0).contains(&alpha) {
            self.original.lerp(&self.outline, alpha)
        } else if alpha == 1.0 {
            self.outline.clone()
        } else if (1.0..2.0).contains(&alpha) {
            self.outline.get_partial(0.0..2.0 - alpha)
        } else if alpha == 2.0 {
            T::empty()
        } else if alpha == 0.0 {
            self.original.clone()
        } else {
            panic!("the alpha is out of range: {alpha}");
        }
    }
}
