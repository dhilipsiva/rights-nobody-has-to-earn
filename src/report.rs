// SPDX-License-Identifier: MIT OR Apache-2.0

use std::io::{self, Write};

use crate::cli::Error;

pub(crate) struct Reporter<W> {
    output: W,
    diagnostics: Option<crate::diagnostics::Recorder>,
}

impl<W: Write> Reporter<W> {
    pub(crate) fn new(output: W) -> Self {
        Self::with_recorder(output, None)
    }

    /// Attach an optional run recorder so each step boundary is also a
    /// measured phase boundary. The recorder only observes: the bytes this
    /// reporter writes are identical with and without it, which the
    /// run-diagnostics self-test holds as a watched control.
    pub(crate) fn with_recorder(
        output: W,
        diagnostics: Option<crate::diagnostics::Recorder>,
    ) -> Self {
        Self {
            output,
            diagnostics,
        }
    }

    pub(crate) fn step(&mut self, name: &str) -> Result<(), Error> {
        if let Some(recorder) = &self.diagnostics {
            recorder.begin_phase(name);
        }
        self.write_line(format_args!("\n\x1b[1m{name}\x1b[0m"))
    }

    pub(crate) fn pass(&mut self, message: impl AsRef<str>) -> Result<(), Error> {
        self.write_line(format_args!("  \x1b[32mok\x1b[0m   {}", message.as_ref()))
    }

    pub(crate) fn warning(&mut self, message: impl AsRef<str>) -> Result<(), Error> {
        self.write_line(format_args!("  \x1b[33mwarn\x1b[0m {}", message.as_ref()))
    }

    pub(crate) fn line(&mut self, message: impl AsRef<str>) -> Result<(), Error> {
        self.write_line(format_args!("{}", message.as_ref()))
    }

    pub(crate) fn bytes(&mut self, body: &[u8]) -> Result<(), Error> {
        self.output.write_all(body).map_err(Error::from)
    }

    pub(crate) fn flush(&mut self) -> Result<(), Error> {
        self.output.flush().map_err(Error::from)
    }

    pub(crate) fn into_inner(self) -> W {
        self.output
    }

    fn write_line(&mut self, arguments: std::fmt::Arguments<'_>) -> Result<(), Error> {
        self.output
            .write_fmt(arguments)
            .and_then(|()| self.output.write_all(b"\n"))
            .and_then(|()| self.output.flush())
            .map_err(Error::from)
    }
}

pub(crate) struct TeeWriter<A, B> {
    first: A,
    second: B,
}

impl<A, B> TeeWriter<A, B> {
    pub(crate) fn new(first: A, second: B) -> Self {
        Self { first, second }
    }

    pub(crate) fn into_parts(self) -> (A, B) {
        (self.first, self.second)
    }
}

impl<A: Write, B: Write> Write for TeeWriter<A, B> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.first.write_all(buffer)?;
        self.second.write_all(buffer)?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.first.flush()?;
        self.second.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct CountingWriter {
        bytes: Vec<u8>,
        flushes: usize,
    }

    impl Write for CountingWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }

    #[test]
    fn reporter_and_tee_preserve_exact_bytes() {
        let mut left = CountingWriter::default();
        let mut right = CountingWriter::default();
        {
            let tee = TeeWriter::new(&mut left, &mut right);
            let mut report = Reporter::new(tee);
            report.step("one").unwrap();
            report.pass("two").unwrap();
            report.flush().unwrap();
        }
        assert_eq!(left.bytes, right.bytes);
        assert_eq!(left.flushes, 3);
        assert_eq!(right.flushes, 3);
        assert_eq!(
            left.bytes,
            b"\n\x1b[1mone\x1b[0m\n  \x1b[32mok\x1b[0m   two\n"
        );
    }
}
