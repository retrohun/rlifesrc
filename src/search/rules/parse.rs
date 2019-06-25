use super::isotropic;
use super::life;

/// std::str::Chars 加上一个缓冲的字节
/// 以下的 parser 中会从 &str 中一个字符一个字符地读取，
/// 如果这个字符不对，可以把它塞回到缓冲区。
struct Chars<'a> {
    chars: std::str::Chars<'a>,
    buffer: Option<char>,
}

impl<'a> Chars<'a> {
    fn new(s: &'a str) -> Self {
        Chars {
            chars: s.chars(),
            buffer: None,
        }
    }

    fn push(&mut self, c: char) {
        self.buffer = Some(c)
    }
}

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.buffer.take() {
            Some(c) => Some(c),
            None => self.chars.next(),
        }
    }
}

fn parse_bs_life(chars: &mut Chars) -> Result<Vec<u8>, String> {
    let mut bs = Vec::new();

    while let Some(c) = chars.next() {
        match c {
            c if c.is_digit(9) => bs.push(c.to_string().parse::<u8>().unwrap()),
            '/' | 'S' | 's' => {
                chars.push(c);
                return Ok(bs);
            }
            _ => return Err(String::from("Missing number in rule")),
        }
    }
    Ok(bs)
}

pub fn parse_life(input: &str) -> Result<life::Life, String> {
    let mut chars = Chars::new(input);
    match chars.next() {
        Some('B') | Some('b') => (),
        _ => return Err(String::from("Expected B at start of rule")),
    }
    let b = parse_bs_life(&mut chars)?;
    match chars.next() {
        Some('/') => (),
        Some(c) => chars.push(c),
        None => return Err(String::from("Missing expected slash between b and s")),
    }
    match chars.next() {
        Some('S') | Some('s') => (),
        _ => return Err(String::from("Expected S after slash")),
    }
    let s = parse_bs_life(&mut chars)?;
    match chars.next() {
        None => Ok(life::Life::new(b, s)),
        _ => Err(String::from("Extra unparsed junk at end of rule string")),
    }
}

#[allow(clippy::cognitive_complexity)]
fn parse_bs_isotropic(chars: &mut Chars) -> Result<Vec<u8>, String> {
    let mut bs = Vec::new();

    // 原来宏时可以放到函数里边的。
    macro_rules! parse_keys {
        ( $( $key: expr => $value: expr, )* ) => {
            {
                let all_keys = vec![$( $key, )*];
                let keys = match chars.next() {
                    Some('-') => {
                        let mut keys = Vec::new();
                        while let Some(c) = chars.next() {
                            if all_keys.contains(&c) {
                                keys.push(c);
                            } else {
                                chars.push(c);
                                break;
                            }
                        }
                        all_keys.into_iter().filter(|c| !keys.contains(c)).collect()
                    }
                    Some(c) if all_keys.contains(&c) => {
                        chars.push(c);
                        let mut keys = Vec::new();
                        while let Some(c) = chars.next() {
                            if all_keys.contains(&c) {
                                keys.push(c);
                            } else {
                                chars.push(c);
                                break;
                            }
                        }
                        keys
                    }
                    Some(c) => {
                        chars.push(c);
                        all_keys
                    }
                    None => all_keys
                };
                for &c in keys.iter() {
                    match c {
                        $(
                            $key => {
                                for &i in $value.iter() {
                                    bs.push(i);
                                }
                            }
                        )*
                        _ => unreachable!(),
                    }
                }
            }
        };
    }

    while let Some(c) = chars.next() {
        match c {
            '0' => bs.push(0x00),
            '1' => parse_keys! {
                'c' => [0x01, 0x04, 0x20, 0x80],
                'e' => [0x02, 0x08, 0x10, 0x40],
            },
            '2' => parse_keys! {
                'c' => vec![0x05, 0x21, 0x84, 0xa0],
                'e' => vec![0x0a, 0x12, 0x48, 0x50],
                'k' => vec![0x0c, 0x11, 0x22, 0x30, 0x41, 0x44, 0x82, 0x88],
                'a' => vec![0x03, 0x06, 0x09, 0x14, 0x28, 0x60, 0x90, 0xc0],
                'i' => vec![0x18, 0x42],
                'n' => vec![0x24, 0x81],
            },
            '3' => parse_keys! {
                'c' => vec![0x25, 0x85, 0xa1, 0xa4],
                'e' => vec![0x1a, 0x4a, 0x52, 0x58],
                'k' => vec![0x32, 0x4c, 0x51, 0x8a],
                'a' => vec![0x0b, 0x16, 0x68, 0xd0],
                'i' => vec![0x07, 0x29, 0x94, 0xe0],
                'n' => vec![0x0d, 0x15, 0x23, 0x61, 0x86, 0xa8, 0xb0, 0xc4],
                'y' => vec![0x31, 0x45, 0x8c, 0xa2],
                'q' => vec![0x26, 0x2c, 0x34, 0x64, 0x83, 0x89, 0x91, 0xc1],
                'j' => vec![0x0e, 0x13, 0x2a, 0x49, 0x54, 0x70, 0x92, 0xc8],
                'r' => vec![0x19, 0x1c, 0x38, 0x43, 0x46, 0x62, 0x98, 0xc2],
            },
            '4' => parse_keys! {
                'c' => vec![0xa5],
                'e' => vec![0x5a],
                'k' => vec![0x33, 0x4d, 0x55, 0x71, 0x8e, 0xaa, 0xb2, 0xcc],
                'a' => vec![0x0f, 0x17, 0x2b, 0x69, 0x96, 0xd4, 0xe8, 0xf0],
                'i' => vec![0x1d, 0x63, 0xb8, 0xc6],
                'n' => vec![0x27, 0x2d, 0x87, 0x95, 0xa9, 0xb4, 0xe1, 0xe4],
                'y' => vec![0x35, 0x65, 0x8d, 0xa3, 0xa6, 0xac, 0xb1, 0xc5],
                'q' => vec![0x36, 0x6c, 0x8b, 0xd1],
                'j' => vec![0x3a, 0x4e, 0x53, 0x59, 0x5c, 0x72, 0x9a, 0xca],
                'r' => vec![0x1b, 0x1e, 0x4b, 0x56, 0x6a, 0x78, 0xd2, 0xd8],
                't' => vec![0x39, 0x47, 0x9c, 0xe2],
                'w' => vec![0x2e, 0x74, 0x93, 0xc9],
                'z' => vec![0x3c, 0x66, 0x99, 0xc3],
            },
            '5' => parse_keys! {
                'c' => vec![0x5b, 0x5e, 0x7a, 0xda],
                'e' => vec![0xa7, 0xad, 0xb5, 0xe5],
                'k' => vec![0x75, 0xae, 0xb3, 0xcd],
                'a' => vec![0x2f, 0x97, 0xe9, 0xf4],
                'i' => vec![0x1f, 0x6b, 0xd6, 0xf8],
                'n' => vec![0x3b, 0x4f, 0x57, 0x79, 0x9e, 0xdc, 0xea, 0xf2],
                'y' => vec![0x5d, 0x73, 0xba, 0xce],
                'q' => vec![0x3e, 0x6e, 0x76, 0x7c, 0x9b, 0xcb, 0xd3, 0xd9],
                'j' => vec![0x37, 0x6d, 0x8f, 0xab, 0xb6, 0xd5, 0xec, 0xf1],
                'r' => vec![0x3d, 0x67, 0x9d, 0xb9, 0xbc, 0xc7, 0xe3, 0xe6],
            },
            '6' => parse_keys! {
                'c' => vec![0x5f, 0x7b, 0xde, 0xfa],
                'e' => vec![0xaf, 0xb7, 0xed, 0xf5],
                'k' => vec![0x77, 0x7d, 0xbb, 0xbe, 0xcf, 0xdd, 0xee, 0xf3],
                'a' => vec![0x3f, 0x6f, 0x9f, 0xd7, 0xeb, 0xf6, 0xf9, 0xfc],
                'i' => vec![0xbd, 0xe7],
                'n' => vec![0x7e, 0xdb],
            },
            '7' => parse_keys! {
                'c' => vec![0x7f, 0xdf, 0xfb, 0xfe],
                'e' => vec![0xbf, 0xef, 0xf7, 0xfd],
            },
            '8' => bs.push(0xff),
            '/' | 'S' | 's' => {
                chars.push(c);
                return Ok(bs);
            }
            _ => return Err(String::from("Missing number in rule")),
        }
    }

    Ok(bs)
}

pub fn parse_isotropic(input: &str) -> Result<isotropic::Life, String> {
    let mut chars = Chars::new(input);
    match chars.next() {
        Some('B') | Some('b') => (),
        _ => return Err(String::from("Expected B at start of rule")),
    }
    let b = parse_bs_isotropic(&mut chars)?;
    match chars.next() {
        Some('/') => (),
        Some(c) => chars.push(c),
        None => return Err(String::from("Missing expected slash between b and s")),
    }
    match chars.next() {
        Some('S') | Some('s') => (),
        _ => return Err(String::from("Expected S after slash")),
    }
    let s = parse_bs_isotropic(&mut chars)?;
    match chars.next() {
        None => Ok(isotropic::Life::new(b, s)),
        _ => Err(String::from("Extra unparsed junk at end of rule string")),
    }
}
