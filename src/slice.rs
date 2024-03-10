use core::{
    fmt::Display,
    ops::{Bound, RangeBounds},
};

use crate::{
    simple::{discard, optioned, origin, pipe_fn},
    split_at::split_at,
    CreateDisplayPipe, DisplayPipe,
};

pub fn omitted<T>(exceed_chars: usize, exceed_display: T) -> impl CreateDisplayPipe
where
    T: Display,
{
    split_at(exceed_chars, origin, discard.terminated(exceed_display))
}

pub fn omitted_with<T, F>(exceed_chars: usize, exceed_fn: F) -> impl CreateDisplayPipe
where
    F: Fn(usize) -> T,
    T: Display,
{
    split_at(exceed_chars, origin, Omitted(exceed_fn))
}

pub fn slice<R>(range: R) -> impl CreateDisplayPipe
where
    R: RangeBounds<usize>,
{
    let start = match range.start_bound().cloned() {
        Bound::Excluded(x) => x.checked_sub(1).expect("invalid excluded lower bound 0"),
        Bound::Included(x) => x,
        Bound::Unbounded => 0,
    };

    let end = match range.end_bound().cloned() {
        Bound::Excluded(x) => Some(x),
        Bound::Included(x) => Some(x + 1),
        Bound::Unbounded => None,
    };

    let display_len = end.map(|end| end.checked_sub(start).unwrap_or(0));

    split_at(
        start,
        discard,
        optioned(display_len, |len| split_at(len, origin, discard), || origin),
    )
}

pub fn count_chars(r: &mut usize) -> impl DisplayPipe + '_ {
    pipe_fn(|w, s| {
        *r += s.chars().count();
        w.write_str(s)
    })
}

struct Omitted<F>(F);

impl<F, T> CreateDisplayPipe for Omitted<F>
where
    F: Fn(usize) -> T,
    T: Display,
{
    fn create_pipe<'a>(&'a self) -> impl DisplayPipe + 'a {
        OmittedTerm {
            len: 0,
            display: &self.0,
        }
    }
}

struct OmittedTerm<F> {
    len: usize,
    display: F,
}

impl<F, T> DisplayPipe for OmittedTerm<F>
where
    F: Fn(usize) -> T,
    T: Display,
{
    fn handle(&mut self, _: &mut dyn core::fmt::Write, s: &str) -> core::fmt::Result {
        self.len += s.chars().count();
        Ok(())
    }

    fn end(&mut self, w: &mut dyn core::fmt::Write) -> core::fmt::Result {
        write!(w, "{}", (self.display)(self.len))
    }
}
