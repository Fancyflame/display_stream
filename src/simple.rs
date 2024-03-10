use core::fmt::{Display, Result, Write};

use crate::{CreateDisplayPipe, DisplayPipe};

pub fn discard() -> Discard {
    Discard
}

pub fn origin() -> Origin {
    Origin
}

pub fn create_pipe_fn<F, R>(f: F) -> PipeFn<F>
where
    F: Fn() -> R,
    R: DisplayPipe,
{
    PipeFn(f)
}

pub fn pipe_fn<F>(f: F) -> PipeFn<F>
where
    F: FnMut(&mut dyn Write, &str) -> Result,
{
    PipeFn(f)
}

pub fn optioned<T, F1, F2, D1, D2>(
    opt: Option<T>,
    if_some: F1,
    if_none: F2,
) -> impl CreateDisplayPipe
where
    F1: FnOnce(T) -> D1,
    F2: FnOnce() -> D2,
    D1: CreateDisplayPipe,
    D2: CreateDisplayPipe,
{
    match opt {
        Some(v) => Branched::A(if_some(v)),
        None => Branched::B(if_none()),
    }
}

pub fn terminated<T: Display>(d: T) -> Terminated<T> {
    Terminated(d)
}

pub struct Discard;

impl DisplayPipe for Discard {
    fn handle(&mut self, _: &mut dyn Write, _: &str) -> Result {
        Ok(())
    }

    fn end(&mut self, _: &mut dyn Write) -> Result {
        Ok(())
    }
}

pub struct Origin;

impl DisplayPipe for Origin {
    fn handle(&mut self, f: &mut dyn Write, s: &str) -> Result {
        f.write_str(s)
    }

    fn end(&mut self, _w: &mut dyn Write) -> Result {
        Ok(())
    }
}

pub struct PipeFn<F>(F);

impl<F, R> CreateDisplayPipe for PipeFn<F>
where
    F: Fn() -> R,
    R: DisplayPipe + 'static,
{
    fn create_pipe<'a>(&'a self) -> impl DisplayPipe + 'a {
        (self.0)()
    }
}

impl<F> DisplayPipe for PipeFn<F>
where
    F: FnMut(&mut dyn Write, &str) -> Result,
{
    fn handle(&mut self, f: &mut dyn Write, s: &str) -> Result {
        (self.0)(f, s)
    }

    fn end(&mut self, _w: &mut dyn Write) -> Result {
        Ok(())
    }
}

pub enum Branched<A, B> {
    A(A),
    B(B),
}

impl<A, B> CreateDisplayPipe for Branched<A, B>
where
    A: CreateDisplayPipe,
    B: CreateDisplayPipe,
{
    fn create_pipe<'a>(&'a self) -> impl DisplayPipe + 'a {
        match self {
            Self::A(p) => Branched::A(p.create_pipe()),
            Self::B(p) => Branched::B(p.create_pipe()),
        }
    }
}

impl<A, B> DisplayPipe for Branched<A, B>
where
    A: DisplayPipe,
    B: DisplayPipe,
{
    fn handle(&mut self, f: &mut dyn Write, s: &str) -> Result {
        match self {
            Self::A(p) => p.handle(f, s),
            Self::B(p) => p.handle(f, s),
        }
    }

    fn end(&mut self, w: &mut dyn Write) -> Result {
        match self {
            Self::A(p) => p.end(w),
            Self::B(p) => p.end(w),
        }
    }
}

pub struct Piped<A, B>(pub(super) A, pub(super) B);

struct PipeHelper<F>(F);
impl<F> Write for PipeHelper<F>
where
    F: FnMut(&str) -> Result,
{
    fn write_str(&mut self, s: &str) -> Result {
        (self.0)(s)
    }
}

impl<A, B> CreateDisplayPipe for Piped<A, B>
where
    A: CreateDisplayPipe,
    B: CreateDisplayPipe,
{
    fn create_pipe<'a>(&'a self) -> impl DisplayPipe + 'a {
        Piped(self.0.create_pipe(), self.1.create_pipe())
    }
}

impl<A, B> DisplayPipe for Piped<A, B>
where
    A: DisplayPipe,
    B: DisplayPipe,
{
    fn handle(&mut self, w: &mut dyn Write, s: &str) -> Result {
        self.0
            .handle(&mut PipeHelper(|s2: &str| self.1.handle(w, s2)), s)
    }

    fn end(&mut self, w: &mut dyn Write) -> Result {
        self.0
            .end(&mut PipeHelper(|s2: &str| self.1.handle(w, s2)))?;
        self.1.end(w)
    }
}

pub struct Terminated<T>(T);

impl<T> CreateDisplayPipe for Terminated<T>
where
    T: Display,
{
    fn create_pipe<'a>(&'a self) -> impl DisplayPipe + 'a {
        Terminated(&self.0)
    }
}

impl<T> DisplayPipe for Terminated<T>
where
    T: Display,
{
    fn handle(&mut self, w: &mut dyn Write, s: &str) -> Result {
        w.write_str(s)
    }

    fn end(&mut self, w: &mut dyn Write) -> Result {
        write!(w, "{}", self.0)
    }
}

pub struct Preceded<T>(T);

impl<T> CreateDisplayPipe for Preceded<T>
where
    T: Display,
{
    fn create_pipe<'a>(&'a self) -> impl DisplayPipe + 'a {
        Preceded(Some(&self.0))
    }
}

impl<T> DisplayPipe for Preceded<Option<T>>
where
    T: Display,
{
    fn handle(&mut self, w: &mut dyn Write, s: &str) -> Result {
        if let Some(p) = self.0.take() {
            write!(w, "{p}")?;
        }
        w.write_str(s)
    }

    fn end(&mut self, w: &mut dyn Write) -> Result {
        if let Some(p) = self.0.take() {
            write!(w, "{p}")
        } else {
            Ok(())
        }
    }
}
