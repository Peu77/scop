use std::fs;
use std::path::Path;

use crate::error::{AppError, Result};

#[derive(Debug)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub fn load(path: &Path) -> Result<Image> {
    let bytes = fs::read(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse(&bytes)
}

fn parse(bytes: &[u8]) -> Result<Image> {
    let mut scanner = HeaderScanner::new(bytes);
    if scanner.token()? != b"P6" {
        return Err(ppm_error("expected P6 magic number"));
    }

    let width = parse_dimension(scanner.token()?, "width")?;
    let height = parse_dimension(scanner.token()?, "height")?;
    let maximum = parse_number(scanner.token()?, "maximum color value")?;
    if maximum != 255 {
        return Err(ppm_error("maximum color value must be 255"));
    }

    let pixel_start = scanner.pixel_start()?;
    let pixel_count = (width as usize)
        .checked_mul(height as usize)
        .and_then(|count| count.checked_mul(3))
        .ok_or_else(|| ppm_error("image dimensions are too large"))?;
    let pixels = bytes
        .get(pixel_start..)
        .ok_or_else(|| ppm_error("missing pixel data"))?;
    if pixels.len() != pixel_count {
        return Err(ppm_error(format!(
            "expected {pixel_count} bytes of pixel data, found {}",
            pixels.len()
        )));
    }

    Ok(Image {
        width,
        height,
        pixels: pixels.to_vec(),
    })
}

fn parse_dimension(token: &[u8], label: &str) -> Result<u32> {
    let value = parse_number(token, label)?;
    if value == 0 {
        return Err(ppm_error(format!("{label} must be greater than zero")));
    }
    Ok(value)
}

fn parse_number(token: &[u8], label: &str) -> Result<u32> {
    let text =
        std::str::from_utf8(token).map_err(|_| ppm_error(format!("{label} is not valid ASCII")))?;
    text.parse()
        .map_err(|_| ppm_error(format!("invalid {label} '{text}'")))
}

fn ppm_error(message: impl Into<String>) -> AppError {
    AppError::Ppm(message.into())
}

struct HeaderScanner<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> HeaderScanner<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn token(&mut self) -> Result<&'a [u8]> {
        self.skip_whitespace_and_comments();
        let start = self.position;
        while self.position < self.bytes.len()
            && !self.bytes[self.position].is_ascii_whitespace()
            && self.bytes[self.position] != b'#'
        {
            self.position += 1;
        }
        if start == self.position {
            return Err(ppm_error("incomplete header"));
        }
        Ok(&self.bytes[start..self.position])
    }

    fn pixel_start(&mut self) -> Result<usize> {
        let Some(&separator) = self.bytes.get(self.position) else {
            return Err(ppm_error("missing whitespace before pixel data"));
        };
        if !separator.is_ascii_whitespace() {
            return Err(ppm_error("missing whitespace before pixel data"));
        }
        self.position += 1;
        if separator == b'\r' && self.bytes.get(self.position) == Some(&b'\n') {
            self.position += 1;
        }
        Ok(self.position)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            while self
                .bytes
                .get(self.position)
                .is_some_and(u8::is_ascii_whitespace)
            {
                self.position += 1;
            }
            if self.bytes.get(self.position) != Some(&b'#') {
                break;
            }
            while self
                .bytes
                .get(self.position)
                .is_some_and(|byte| *byte != b'\n')
            {
                self.position += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parse_accepts_p6_pixels_and_header_comments() {
        let image = parse(b"P6\n# palette\n2 1\n255\n\x00\x01\x02\xfd\xfe\xff").unwrap();

        assert_eq!(image.pixels, [0, 1, 2, 253, 254, 255]);
    }

    #[test]
    fn parse_preserves_whitespace_as_first_pixel_byte() {
        let image = parse(b"P6\n1 1\n255\n\x20\x00\xff").unwrap();

        assert_eq!(image.pixels, [32, 0, 255]);
    }

    #[test]
    fn parse_rejects_truncated_pixel_data() {
        let error = parse(b"P6\n1 1\n255\n\x00\x01").unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid PPM: expected 3 bytes of pixel data, found 2"
        );
    }

    #[test]
    fn parse_rejects_unsupported_maximum_color_value() {
        let error = parse(b"P6\n1 1\n100\n\x00\x01\x02").unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid PPM: maximum color value must be 255"
        );
    }
}
