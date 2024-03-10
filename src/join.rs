use core::fmt::Display;

pub trait JoinExt: Sized {
    fn lazy_join<S: Display>(self, sep: S) -> Join<Self, S> {
        Join {
            disp_iter: self,
            sep,
        }
    }
}

impl<T> JoinExt for T
where
    T: Iterator + Clone,
    T::Item: Display,
{
}

#[derive(Clone)]
pub struct Join<I, S> {
    pub(super) disp_iter: I,
    pub(super) sep: S,
}

impl<I, S> Display for Join<I, S>
where
    I: Iterator + Clone,
    I::Item: Display,
    S: Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let iter = self.disp_iter.clone();
        let mut is_first = true;

        for item in iter {
            if is_first {
                is_first = false;
            } else {
                self.sep.fmt(f)?;
            }
            item.fmt(f)?;
        }
        Ok(())
    }
}
