use alloc::{format, string::String, vec::Vec};
use core::fmt::Write;
use stylus_sdk::alloy_primitives::FixedBytes;

const SVG_WIDTH: i32 = 1000;
const SVG_HEIGHT: i32 = 1000;
const BACKGROUND_COLOR: &str = "#1a1a1a";

const MIN_OSCILLATIONS: i32 = 4;
const MAX_OSCILLATIONS: i32 = 15;

const MIN_STROKE_WIDTH: i32 = 10;
const MAX_STROKE_WIDTH: i32 = 80;

const MIN_PERIOD: i32 = 20;
const MAX_PERIOD: i32 = 100;

const MIN_AMPLITUDE: i32 = 100;
const MAX_AMPLITUDE: i32 = 600;

/// Configuration for a squiggle SVG using a single seed
pub struct SquiggleGenerator {
    /// 32-bytes used as seed for all randomization
    pub seed: [u8; 32],
}

impl Default for SquiggleGenerator {
    fn default() -> Self {
        let seed_bytes = [0; 32];
        Self { seed: seed_bytes }
    }
}

impl SquiggleGenerator {
    /// Creates a new SquiggleConfig with the provided seed
    pub fn new(seed: FixedBytes<32>) -> Self {
        Self { seed: seed.0 }
    }

    /// Maps a byte value to a range
    fn map_to_range(byte: u8, min: i32, max: i32) -> i32 {
        min + ((byte as i32 * (max - min)) / 255)
    }

    /// Generates all parameters needed for the SVG from the seed
    fn generate_params(&self) -> (Vec<i32>, Vec<i32>, i32, u8) {
        // Bytes 0-2 are used for the number of oscillations, stroke width, and gradient type
        let oscillations = Self::map_to_range(self.seed[0], MIN_OSCILLATIONS, MAX_OSCILLATIONS);
        let stroke_width = Self::map_to_range(self.seed[1], MIN_STROKE_WIDTH, MAX_STROKE_WIDTH);
        let gradient_type = self.seed[2] % 3;

        // Generate x-offsets (periods) using bytes 3-17
        let mut x_offsets = [0i32; MAX_OSCILLATIONS as usize];
        for i in 0..oscillations as usize {
            let byte_index = 3 + i as usize;
            let period = Self::map_to_range(self.seed[byte_index], MIN_PERIOD, MAX_PERIOD);
            x_offsets[i] = period;
        }

        // Generate y-offsets (amplitudes) using bytes 18-32
        let mut y_offsets = [0i32; MAX_OSCILLATIONS as usize];
        for i in 0..oscillations as usize {
            let byte_index = 15 + i as usize;
            let amplitude = Self::map_to_range(self.seed[byte_index], MIN_AMPLITUDE, MAX_AMPLITUDE);
            let sign = if i % 2 == 0 { -1 } else { 1 };
            y_offsets[i] = sign * amplitude;
        }

        (
            x_offsets[..oscillations as usize].to_vec(),
            y_offsets[..oscillations as usize].to_vec(),
            stroke_width,
            gradient_type,
        )
    }

    /// Creates a smooth path using cubic Bézier curves
    fn generate_smooth_path(&self, x_offsets: &[i32], y_offsets: &[i32]) -> String {
        let mut path = String::new();

        // Calculate total width for centering
        let total_width: i32 = x_offsets.iter().sum();
        let start_x = (1000 - total_width) / 2;
        let center_y = 500;
        let mut current_x = start_x;

        // Start path at the beginning point
        write!(path, "M {},{} ", current_x, center_y).unwrap();

        // Create a smooth curve for each oscillation
        for (&x_offset, &y_offset) in x_offsets.iter().zip(y_offsets.iter()) {
            let end_x = current_x + x_offset;

            // Control points calculation - use integer division
            let cp1_x = current_x + (x_offset / 3);
            let cp1_y = center_y + y_offset;

            let cp2_x = current_x + (2 * x_offset / 3);
            let cp2_y = center_y + y_offset;

            // Add cubic Bézier curve command
            write!(
                path,
                "C {},{} {},{} {},{} ",
                cp1_x, cp1_y, cp2_x, cp2_y, end_x, center_y
            )
            .unwrap();

            current_x = end_x;
        }

        path
    }

    fn base64_encode(data: &str) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        const PAD: u8 = b'=';

        let bytes = data.as_bytes();
        let len = bytes.len();
        let pad_len = (3 - (len % 3)) % 3;
        let output_len = ((len + pad_len) / 3) * 4;
        let mut output = Vec::with_capacity(output_len);

        let mut i = 0;
        while i < len {
            let mut n = bytes[i] as u32;
            n = (n << 8) | if i + 1 < len { bytes[i + 1] as u32 } else { 0 };
            n = (n << 8) | if i + 2 < len { bytes[i + 2] as u32 } else { 0 };

            output.push(ALPHABET[((n >> 18) & 0x3F) as usize]);
            output.push(ALPHABET[((n >> 12) & 0x3F) as usize]);
            output.push(if i + 1 < len {
                ALPHABET[((n >> 6) & 0x3F) as usize]
            } else {
                PAD
            });
            output.push(if i + 2 < len {
                ALPHABET[(n & 0x3F) as usize]
            } else {
                PAD
            });

            i += 3;
        }

        // Safe to unwrap as we know the output contains only valid ASCII
        String::from_utf8(output).unwrap()
    }

    /// Writes the gradient definition to the SVG
    fn write_gradient(&self, svg: &mut String, gradient_type: u8) {
        let rainbow_gradient = [
            ("0.00", (255, 0, 0)),    // Red
            ("16.67", (255, 142, 0)), // Orange
            ("33.33", (255, 239, 0)), // Yellow
            ("50.00", (0, 241, 29)),  // Green
            ("66.67", (0, 255, 255)), // Cyan
            ("83.33", (0, 64, 255)),  // Blue
            ("100.0", (128, 0, 255)), // Purple
        ];

        let sunset_gradient = [
            ("0.00", (255, 95, 109)),
            ("25.00", (255, 140, 105)),
            ("50.00", (255, 160, 122)),
            ("75.00", (255, 182, 193)),
            ("100.0", (255, 192, 203)),
        ];

        let ocean_gradient = [
            ("0.00", (30, 144, 255)),
            ("25.00", (0, 206, 209)),
            ("50.00", (32, 178, 170)),
            ("75.00", (72, 209, 204)),
            ("100.0", (0, 255, 255)),
        ];

        let gradient: &[(&str, (u8, u8, u8))] = match gradient_type {
            0 => &rainbow_gradient,
            1 => &sunset_gradient,
            2 => &ocean_gradient,
            _ => &rainbow_gradient,
        };

        writeln!(
            svg,
            r#"    <defs>\n        <linearGradient id="gradient" x1="0%" y1="0%" x2="100%" y2="0%">"#
        )
        .unwrap();

        for (offset, (x, y, z)) in gradient {
            writeln!(
                svg,
                r#"            <stop offset="{offset}%" style="stop-color:rgb({x},{y},{z})"/>"#,
            )
            .unwrap();
        }

        writeln!(svg, r#"        </linearGradient>\n    </defs>"#).unwrap();
    }

    /// Generates the complete SVG with a random squiggle
    pub fn generate_svg(&self) -> String {
        let (x_offsets, y_offsets, stroke_width, gradient_type) = self.generate_params();
        let mut svg = String::new();

        // Start SVG document
        writeln!(svg, r#"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#, SVG_WIDTH, SVG_HEIGHT, SVG_WIDTH, SVG_HEIGHT).unwrap();
        writeln!(
            svg,
            r#"    <rect width="100%" height="100%" fill="{}"/>"#,
            BACKGROUND_COLOR
        )
        .unwrap();

        // Generate smooth path
        let path_data = self.generate_smooth_path(&x_offsets, &y_offsets);
        writeln!(svg, r#"    <path "#).unwrap();
        writeln!(svg, r#"        d="{}""#, path_data).unwrap();
        writeln!(svg, r#"        fill="none""#).unwrap();
        writeln!(svg, r#"        stroke="url(#gradient)""#).unwrap();
        writeln!(svg, r#"        stroke-width="{}""#, stroke_width).unwrap();
        writeln!(svg, r#"        stroke-linecap="round""#).unwrap();
        writeln!(svg, r#"    />"#).unwrap();

        // Write gradient
        self.write_gradient(&mut svg, gradient_type);

        writeln!(svg, r#"</svg>"#).unwrap();
        svg
    }

    /// Generates the complete onchain NFT metadata
    pub fn generate_metadata(&self) -> String {
        let svg = self.generate_svg();
        let base64_svg = Self::base64_encode(&svg);
        let metadata = format!(
            r#"{{"name":"Stylus Squiggle","description":"A squiggle generated by Stylus","image":"data:image/svg+xml;base64,{}"}}"#,
            base64_svg
        );
        let base64_metadata = Self::base64_encode(&metadata);

        let final_metadata = format!(r#"data:application/json;base64,{}"#, base64_metadata);
        final_metadata
    }
}

#[cfg(test)]
mod tests {
    use hex::FromHex;

    use super::*;

    #[test]
    fn test_svg_generation() {
        let seed = FixedBytes::<32>::from_hex(
            "1234592349abcdef1234567890abcdef0001567890abcdef1234567890abcdef",
        )
        .unwrap();
        let config = SquiggleGenerator::new(seed);
        let svg = config.generate_svg();

        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>\n"));
        assert!(svg.contains("path"));
        assert!(svg.contains("gradient"));

        // println!("{}", svg);
    }
}
