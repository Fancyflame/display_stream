#![no_std]

pub use self::join::JoinExt;
use core::ops::RangeBounds;
pub use core::{fmt::*, format_args};
use split_at::IntoSpliter;

pub mod join;
pub mod replace;
pub mod simple;
pub mod slice;
pub mod split_at;

pub struct UsePipe<D, T> {
    source: D,
    create_pipe: T,
}

impl<D, T> Display for UsePipe<D, T>
where
    D: Display,
    T: CreateDisplayPipe,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        struct Helper<F>(F);
        impl<F> Write for Helper<F>
        where
            F: FnMut(&str) -> Result,
        {
            fn write_str(&mut self, s: &str) -> Result {
                (self.0)(s)
            }
        }

        let mut pipe = self.create_pipe.create_pipe();
        let mut helper = Helper(|s: &str| pipe.handle(f, s));
        write!(helper, "{}", self.source)?;
        pipe.end(f)
    }
}

pub trait CreateDisplayPipe {
    fn create_pipe<'a>(&'a self) -> impl DisplayPipe + 'a;
    fn to_display<T>(self, disp: T) -> UsePipe<T, Self>
    where
        Self: Sized,
        T: Display,
    {
        UsePipe {
            source: disp,
            create_pipe: self,
        }
    }

    fn pipe<T>(self, other: T) -> impl CreateDisplayPipe
    where
        Self: Sized,
        T: CreateDisplayPipe,
    {
        simple::Piped(self, other)
    }

    fn terminated<T: Display>(self, d: T) -> impl CreateDisplayPipe
    where
        Self: Sized,
        T: Display,
    {
        self.pipe(simple::terminated(d))
    }
}

impl<F, R> CreateDisplayPipe for F
where
    F: Fn() -> R,
    R: DisplayPipe + 'static,
{
    fn create_pipe<'a>(&'a self) -> impl DisplayPipe + 'a {
        self()
    }
}

pub trait DisplayPipe {
    fn handle(&mut self, w: &mut dyn Write, s: &str) -> Result;
    fn end(&mut self, w: &mut dyn Write) -> Result;
    fn pipe<T>(self, other: T) -> impl DisplayPipe
    where
        Self: Sized,
        T: DisplayPipe,
    {
        simple::Piped(self, other)
    }
}

impl<T> DisplayPipe for &mut T
where
    T: DisplayPipe,
{
    fn handle(&mut self, w: &mut dyn Write, s: &str) -> Result {
        T::handle(self, w, s)
    }
    fn end(&mut self, w: &mut dyn Write) -> Result {
        T::end(self, w)
    }
}

pub trait DisplayExt: Display + Sized {
    fn pipe<P>(self, p: P) -> impl Display
    where
        P: CreateDisplayPipe,
    {
        p.to_display(self)
    }

    fn replace<'a, R>(self, replace: &'a str, with: R) -> impl Display
    where
        R: Display,
    {
        self.pipe(replace::replace(replace, with, None))
    }

    fn replacen<'a, R>(self, replace: &'a str, with: R, count: usize) -> impl Display
    where
        R: Display,
    {
        self.pipe(replace::replace(replace, with, Some(count)))
    }

    fn slice<R>(self, r: R) -> impl Display
    where
        R: RangeBounds<usize>,
    {
        self.pipe(slice::slice(r))
    }

    fn split_at<S, P1, P2>(self, split_at: S, pipe1: P1, pipe2: P2) -> impl Display
    where
        S: IntoSpliter + Clone,
        P1: CreateDisplayPipe,
        P2: CreateDisplayPipe,
    {
        self.pipe(split_at::split_at(split_at, pipe1, pipe2))
    }

    fn omitted<T>(self, exceed_chars: usize, exceed_display: T) -> impl Display
    where
        T: Display,
    {
        self.pipe(slice::omitted(exceed_chars, exceed_display))
    }

    fn omitted_with<F, T>(self, exceed_chars: usize, f: F) -> impl Display
    where
        F: Fn(usize) -> T,
        T: Display,
    {
        self.pipe(slice::omitted_with(exceed_chars, f))
    }

    fn compute_len(&self) -> usize {
        struct Counter(usize);
        impl Write for Counter {
            fn write_str(&mut self, s: &str) -> Result {
                self.0 += s.len();
                Ok(())
            }
        }

        let mut d = Counter(0);
        write!(&mut d, "{self}").unwrap();
        d.0
    }
}

impl<T: Display> DisplayExt for T {}

#[macro_export]
macro_rules! lazy_format {
    (move $($tt:tt)*)=>{
        $crate::__prv_lazy_format!(move, $($tt)*)
    };
    ($lit:literal $($tt:tt)*) => {
        $crate::__prv_lazy_format!(, $lit $($tt)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __prv_lazy_format {
    ($($move:ident)?, $($tt:tt)*) => {
        $crate::FormatterFn($($move)? |formatter: &mut $crate::Formatter| {
            formatter.write_fmt($crate::format_args!($($tt)*))
        })
    };
}

pub fn format_fn<F>(f: F) -> FormatterFn<F>
where
    F: Fn(&mut Formatter) -> Result,
{
    FormatterFn(f)
}

pub struct FormatterFn<F>(pub F);

impl<F> Display for FormatterFn<F>
where
    F: Fn(&mut Formatter) -> Result,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        (self.0)(f)
    }
}
