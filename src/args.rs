use crate::canvas::Mode;
use eyre::Context;
use lexopt::prelude::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Opt {
    pub(crate) log_x: bool,
    pub(crate) log_y: bool,
    pub(crate) x_is_row: bool,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) mode: Mode,
    pub(crate) cdf: bool,
}

impl Opt {
    pub fn parse_from_env(parser: &mut lexopt::Parser) -> eyre::Result<Self> {
        let mut opt = Opt {
            log_x: false,
            log_y: false,
            x_is_row: true,
            width: 72,
            height: 40,
            mode: Mode::Dot,
            cdf: false,
        };
        while let Some(arg) = parser.next().context("read next argument")? {
            match arg {
                Short('h') | Long("help") => {
                    unimplemented!();
                }
                Short('d') => {
                    let dim = parser.value().context("value for -d")?;
                    let Some(dim) = dim.to_str() else {
                        eyre::bail!("-d argument contains invalid characters");
                    };
                    if let Some((width, height)) = dim.split_once('x') {
                        opt.width = width.parse().context("parse width in -d argument")?;
                        opt.height = height.parse().context("parse height in -d argument")?;
                    } else {
                        eyre::bail!(
                            "-d must be specified as WxH (eg, 72x40, which is the default)"
                        );
                    }
                }
                Short('l') | Long("log") => {
                    let dim = parser.value().context("value for --log")?;
                    if dim == "x" {
                        opt.log_x = true;
                    } else if dim == "y" {
                        opt.log_y = true;
                    } else if dim == "c" {
                        eyre::bail!("--log c is not yet supported");
                    } else {
                        eyre::bail!("--log takes x, y, or c");
                    }
                }
                Short('m') | Long("mode") => {
                    let mode = parser.value().context("value for --mode")?;
                    if mode == "dot" {
                        opt.mode = Mode::Dot;
                    } else if mode == "count" {
                        opt.mode = Mode::Count;
                    } else {
                        eyre::bail!("--mode takes dot (the default) or count");
                    }
                }
                Short('x') => {
                    opt.x_is_row = false;
                }
                Long("cdf") => {
                    opt.cdf = true;
                }
                arg => return Err(arg.unexpected().into()),
            }
        }

        if opt.cdf {
            eyre::ensure!(
                opt.x_is_row,
                "CDF is only over the Y value; an explicit X value will be ignored"
            );
            eyre::ensure!(
            !opt.log_x,
            "CDF is only over the Y value and changes the axes; logarithmic X would have no effet"
        );
            // NOTE: log y is interpreted as log of the _input_ not _output_
        }

        Ok(opt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    #[cfg(unix)]
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    #[cfg(windows)]
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    fn bad_string(text: &str) -> OsString {
        #[cfg(any(unix, all(target_os = "wasi", target_env = "p1")))]
        {
            let mut text = text.as_bytes().to_vec();
            for ch in &mut text {
                if *ch == b'@' {
                    *ch = b'\xFF';
                }
            }
            OsString::from_vec(text)
        }
    }
    #[test]
    fn d_dims_ok() -> eyre::Result<()> {
        let opt = Opt::parse_from_env(&mut lexopt::Parser::from_args(&["-d50x50"]))
            .context("parse command-line arguments")?;
        assert_eq!(
            opt,
            Opt {
                log_x: false,
                log_y: false,
                x_is_row: true,
                width: 50,
                height: 50,
                mode: Mode::Dot,
                cdf: false,
            }
        );
        Ok(())
    }
    #[test]
    #[should_panic(expected = "-d must be specified as WxH (eg, 72x40, which is the default)")]
    fn d_dims_missing() {
        Opt::parse_from_env(&mut lexopt::Parser::from_args(&["-dd"]))
            .context("parse command-line arguments")
            .unwrap();
    }
    #[test]
    #[should_panic(expected = "-d argument contains invalid characters")]
    fn d_dims_inv_char() {
        Opt::parse_from_env(&mut lexopt::Parser::from_args(&[bad_string("-d@")]))
            .context("parse command-line arguments")
            .unwrap();
    }
}
