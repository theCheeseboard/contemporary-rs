const NUMBER_EXTENSIONS: &[&str] = &[
    "twenty",
    "nineteen",
    "eighteen",
    "seventeen",
    "sixteen",
    "fifteen",
    "fourteen",
    "thirteen",
    "twelve",
    "eleven",
    "ten",
    "nine",
    "eight",
    "seven",
    "six",
    "five",
    "four",
    "three",
    "two",
    "one",
];

pub fn mangle(string: &str) -> String {
    string
        .chars()
        .map(|c| match c {
            'A' => 'Å',
            'B' => 'β',
            'C' => 'Ç',
            'D' => 'Đ',
            'E' => 'Ê',
            'G' => 'Ĝ',
            'H' => 'Ĥ',
            'I' => 'Î',
            'J' => 'Ĵ',
            'K' => 'Ķ',
            'L' => 'Ļ',
            'M' => 'Ḿ',
            'N' => 'Ñ',
            'O' => 'Ö',
            'P' => 'Þ',
            'R' => 'Ŕ',
            'S' => 'Š',
            'T' => 'Ţ',
            'U' => 'Û',
            'V' => 'V',
            'v' => 'v',
            'W' => 'Ŵ',
            'Y' => 'Ý',
            'a' => 'å',
            'c' => 'ç',
            'd' => 'ð',
            'e' => 'é',
            'g' => 'ĝ',
            'h' => 'ĥ',
            'i' => 'î',
            'j' => 'ĵ',
            'k' => 'ķ',
            'l' => 'ļ',
            'm' => 'ḿ',
            'n' => 'ñ',
            'o' => 'ö',
            'p' => 'þ',
            'r' => 'ŕ',
            's' => 'š',
            't' => 'ţ',
            'u' => 'û',
            'w' => 'ŵ',
            'y' => 'ý',
            'z' => 'ƶ',
            _ => c,
        })
        .collect::<String>()
}

pub fn contain(string: &String, len: usize) -> String {
    let mut extension_length = ((len * 7) / 10).min(1);
    let mut all_extensions = NUMBER_EXTENSIONS.to_vec();
    let mut extensions = Vec::new();

    while extension_length > 0 {
        let next_extension = all_extensions.pop().unwrap();
        extension_length = extension_length.saturating_sub(next_extension.len());
        extensions.push(next_extension);
        if all_extensions.is_empty() {
            all_extensions = NUMBER_EXTENSIONS.to_vec();
        }
    }

    format!("[{} {}]", string, extensions.join(" "))
}

pub fn contain_variable(string: &String) -> String {
    format!("»{}«", string)
}
