use core::fmt::{Result, Write};

use crate::{CreateDisplayPipe, DisplayPipe};

pub trait IntoSpliter {
    fn into_spliter(self) -> impl FnMut(char) -> bool;
}

impl IntoSpliter for usize {
    fn into_spliter(self) -> impl FnMut(char) -> bool {
        let mut cursor = 0;
        move |_| {
            if cursor == self {
                true
            } else {
                cursor += 1;
                false
            }
        }
    }
}

impl IntoSpliter for char {
    fn into_spliter(self) -> impl FnMut(char) -> bool {
        move |ch| ch == self
    }
}

impl<F> IntoSpliter for F
where
    F: FnMut(char) -> bool,
{
    fn into_spliter(self) -> impl FnMut(char) -> bool {
        self
    }
}

#[derive(Clone)]
pub struct SplitAt<S, P1, P2> {
    spliter: S,
    pipe1: P1,
    pipe2: P2,
}

impl<F, P1, P2> CreateDisplayPipe for SplitAt<F, P1, P2>
where
    F: IntoSpliter + Clone,
    P1: CreateDisplayPipe,
    P2: CreateDisplayPipe,
{
    fn create_pipe<'a>(&'a self) -> impl DisplayPipe + 'a {
        SplitAt {
            spliter: Some(self.spliter.clone().into_spliter()),
            pipe1: self.pipe1.create_pipe(),
            pipe2: self.pipe2.create_pipe(),
        }
    }
}

impl<S, P1, P2> DisplayPipe for SplitAt<Option<S>, P1, P2>
where
    S: FnMut(char) -> bool,
    P1: DisplayPipe,
    P2: DisplayPipe,
{
    fn handle(&mut self, w: &mut dyn Write, s: &str) -> Result {
        let Some(spliter) = &mut self.spliter else {
            return self.pipe2.handle(w, s);
        };

        let mut byte_split_at = 0;
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.peek().copied() {
            if spliter(ch) {
                self.pipe1.end(w)?;
                self.spliter = None;
                break;
            } else {
                byte_split_at += ch.len_utf8();
                chars.next();
            }
        }

        let (s1, s2) = s.split_at(byte_split_at);
        if !s1.is_empty() {
            self.pipe1.handle(w, s1)?;
        }
        if !s2.is_empty() {
            self.pipe2.handle(w, s2)?;
        }

        Ok(())
    }

    fn end(&mut self, w: &mut dyn Write) -> Result {
        self.pipe1.end(w)?;
        self.pipe2.end(w)
    }
}

pub fn split_at<S, P1, P2>(split_at: S, pipe1: P1, pipe2: P2) -> impl CreateDisplayPipe
where
    S: IntoSpliter + Clone,
    P1: CreateDisplayPipe,
    P2: CreateDisplayPipe,
{
    SplitAt {
        spliter: split_at,
        pipe1,
        pipe2,
    }
}
