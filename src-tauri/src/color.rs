// Color formats for the tray's recent colors. Same order and output as
// FORMATS in src/lib/color.js: HEX, RGB, RGBA, HSL, HSLA.

pub fn format(hex: &str, index: usize) -> String {
    let n = u32::from_str_radix(hex, 16).unwrap_or(0);
    let (r, g, b) = ((n >> 16) & 255, (n >> 8) & 255, n & 255);
    match index {
        1 => format!("rgb({r},{g},{b})"),
        2 => format!("rgba({r},{g},{b},1)"),
        3 => {
            let (h, s, l) = hsl(r, g, b);
            format!("hsl({h},{s}%,{l}%)")
        }
        4 => {
            let (h, s, l) = hsl(r, g, b);
            format!("hsla({h},{s}%,{l}%,1)")
        }
        _ => format!("#{hex}"),
    }
}

pub fn rgb(hex: &str) -> [u8; 3] {
    let [_, r, g, b] = u32::from_str_radix(hex, 16).unwrap_or(0).to_be_bytes();
    [r, g, b]
}

fn hsl(r: u32, g: u32, b: u32) -> (i64, i64, i64) {
    let (r, g, b) = (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let (mut h, mut s) = (0.0, 0.0);

    if max != min {
        let d = max - min;
        s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        h = if max == r {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        } * 60.0;
    }
    (h.round() as i64, (s * 100.0).round() as i64, (l * 100.0).round() as i64)
}
