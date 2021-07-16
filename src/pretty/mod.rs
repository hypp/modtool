
use serde_json::ser::Formatter;
use std::io::{self, Write};

fn indent<W>(wr: &mut W, n: usize, s: &[u8]) -> io::Result<()>
where
    W: ?Sized + io::Write,
{
    for _ in 0..n {
        wr.write_all(s)?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub struct PrettyFormatter2<'a> {
    current_indent: usize,
    has_value: bool,
    indent: &'a [u8],
    item_count: usize,
}

impl<'a> PrettyFormatter2<'a> {
    /// Construct a pretty printer formatter that defaults to using two spaces for indentation.
    pub fn new() -> Self {
        PrettyFormatter2::with_indent(b"  ")
    }

    /// Construct a pretty printer formatter that uses the `indent` string for indentation.
    pub fn with_indent(indent: &'a [u8]) -> Self {
        PrettyFormatter2 {
            current_indent: 0,
            has_value: false,
            indent,
            item_count: 0,
        }
    }
}

impl<'a> Default for PrettyFormatter2<'a> {
    fn default() -> Self {
        PrettyFormatter2::new()
    }
}
impl<'a> Formatter for PrettyFormatter2<'a> {
    fn begin_array<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.item_count = 0;
        self.current_indent += 1;
        self.has_value = false;
        w.write_all(b"[")
    }
    fn end_array<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.current_indent -= 1;

        if self.has_value {
            w.write_all(b"\n")?;
            indent(w, self.current_indent, self.indent)?;
        }

        w.write_all(b"]")
    }
    fn begin_array_value<W: ?Sized + Write>(&mut self, w: &mut W, first: bool) -> io::Result<()> {
        if self.item_count == 0 {
            if first {
                w.write_all(b"\n")?;
            } else {
                w.write_all(b",\n")?;
            }
            indent(w, self.current_indent, self.indent)?;
        } else {
            w.write_all(b", ")?;
        }
        self.item_count = (self.item_count + 1) % 16;
        Ok(())       
    }
    fn end_array_value<W: ?Sized + Write>(&mut self, _w: &mut W) -> io::Result<()> {
        self.has_value = true;
        Ok(())
    }
    fn begin_object<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.current_indent += 1;
        self.has_value = false;
        w.write_all(b"{")
    }
    fn end_object<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.current_indent -= 1;

        if self.has_value {
            w.write_all(b"\n")?;
            indent(w, self.current_indent, self.indent)?;
        }

        w.write_all(b"}")
    }
    fn begin_object_key<W: ?Sized + Write>(&mut self, w: &mut W, first: bool) -> io::Result<()> {
        if first {
            w.write_all(b"\n")?;
        } else {
            w.write_all(b",\n")?;
        }
        indent(w, self.current_indent, self.indent)
    }
    fn begin_object_value<W: ?Sized + Write>(&mut self, w: &mut W) -> io::Result<()> {
        w.write_all(b": ")
    }
    fn end_object_value<W: ?Sized + Write>(&mut self, _w: &mut W) -> io::Result<()> {
        self.has_value = true;
        Ok(())
    }
}