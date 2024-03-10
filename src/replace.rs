use core::{
    fmt::{Display, Write},
    str::CharIndices,
};

use crate::{CreateDisplayPipe, DisplayPipe};

pub struct Replace<'a, R> {
    replace: &'a str,
    with: R,
    count: Option<usize>,
    first_char: char,
}

pub struct ReplaceRt<'a, R> {
    this: &'a Replace<'a, R>,
    temp_iter: CharIndices<'a>,
    count: Option<usize>,
}

impl<R> CreateDisplayPipe for Replace<'_, R>
where
    R: Display,
{
    fn create_pipe<'a>(&'a self) -> impl DisplayPipe + 'a {
        ReplaceRt {
            this: self,
            temp_iter: self.replace.char_indices(),
            count: self.count,
        }
    }
}

impl<R> DisplayPipe for ReplaceRt<'_, R>
where
    R: Display,
{
    fn handle(&mut self, w: &mut dyn Write, s: &str) -> core::fmt::Result {
        let Replace {
            replace,
            with,
            count: _,
            first_char,
        } = self.this;

        let mut chars = s.chars();
        if let Some(0) = self.count {
            w.write_str(chars.as_str())?;
            return Ok(());
        }

        while let Some(input_char) = chars.next() {
            let (temp_offset, temp_char) = self.temp_iter.next().unwrap();

            if input_char == temp_char {
                if self.temp_iter.as_str().is_empty() {
                    write!(w, "{}", with)?;
                    self.temp_iter = replace.char_indices();
                    if let Some(rest_count) = &mut self.count {
                        *rest_count -= 1;
                        if *rest_count == 0 {
                            w.write_str(chars.as_str())?;
                            break;
                        }
                    }
                }
            } else {
                write!(w, "{}", &replace[..temp_offset])?;
                self.temp_iter = replace.char_indices();
                if input_char == *first_char {
                    // template_iter can never be fully matched
                    self.temp_iter.next();
                } else {
                    w.write_char(input_char)?;
                }
            }
        }

        Ok(())
    }

    fn end(&mut self, w: &mut dyn Write) -> core::fmt::Result {
        let bound = self.temp_iter.next().unwrap().0;
        self.temp_iter = self.this.replace.char_indices();
        write!(w, "{}", &self.this.replace[..bound])
    }
}

pub fn replace<'a, R>(replace: &'a str, with: R, count: Option<usize>) -> Replace<'a, R>
where
    R: Display,
{
    Replace {
        replace,
        with,
        count,
        first_char: replace
            .chars()
            .next()
            .expect("template string cannot be an empty string"),
    }
}
