// Copyright (C) 2022 Cendyne.
// This file is part of Cendyne Media-Server.

// Cendyne Media-Server is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// Cendyne Media-Server is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Transformation {
    Scale(f32),
    Resize(u32, u32),
    Background(u32),
    Blur(f32),
    Crop(u32, u32, u32, u32),
}

#[derive(Debug, PartialEq)]
pub struct TransformationList(pub Vec<Transformation>);

impl fmt::Display for Transformation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Transformation::Scale(factor) => write!(f, "s{}", factor),
            Transformation::Resize(w, h) => write!(f, "r{}_{}", w, h),
            Transformation::Background(color) => write!(f, "bg{:06x}", color),
            Transformation::Blur(sigma) => write!(f, "bl{}", sigma),
            Transformation::Crop(x, y, w, h) => write!(f, "c{}_{}_{}_{}", x, y, w, h),
        }
    }
}

impl fmt::Display for TransformationList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut has_first = false;
        self.0.iter().fold(Ok(()), |acc, t| {
            if let Ok(()) = acc {
                if has_first {
                    write!(f, ",{}", t)
                } else {
                    has_first = true;
                    write!(f, "{}", t)
                }
            } else {
                acc
            }
        })
    }
}

impl FromStr for Transformation {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("Cannot parse an empty string".to_string());
        }
        let mut chars = s.chars();
        let first = chars.next();
        match first {
            Some('s') => {
                let factor = s[1..].parse::<f32>().map_err(|e| format!("{}", e))?;
                Ok(Transformation::Scale(factor))
            }
            Some('r') => {
                if let Some(first_underscore) = s.find('_') {
                    let w = s[1..first_underscore]
                        .parse::<u32>()
                        .map_err(|e| format!("{}", e))?;
                    let h = s[first_underscore + 1..]
                        .parse::<u32>()
                        .map_err(|e| format!("{}", e))?;
                    Ok(Transformation::Resize(w, h))
                } else {
                    Err(format!("Could not parse {} into a transformation", s))
                }
            }
            Some('b') => {
                if let Some(second) = chars.next() {
                    match second {
                        'g' => {
                            let color =
                                u32::from_str_radix(&s[2..], 16).map_err(|e| format!("{}", e))?;
                            Ok(Transformation::Background(color))
                        }
                        'l' => {
                            let factor = s[2..].parse::<f32>().map_err(|e| format!("{}", e))?;
                            Ok(Transformation::Blur(factor))
                        }
                        _ => Err(format!("Could not parse {} into a transformation", s)),
                    }
                } else {
                    Err(format!("Could not parse {} into a transformation", s))
                }
            }
            Some('c') => {
                if let Some(a) = s.find('_') {
                    let x = s[1..a].parse::<u32>().map_err(|e| format!("{}", e))?;
                    let rest = &s[a + 1..];
                    if let Some(b) = rest.find('_') {
                        let y = rest[..b].parse::<u32>().map_err(|e| format!("{}", e))?;
                        let rest = &rest[b + 1..];
                        if let Some(c) = rest.find('_') {
                            let w = rest[..c].parse::<u32>().map_err(|e| format!("{}", e))?;
                            let h = rest[c + 1..].parse::<u32>().map_err(|e| format!("{}", e))?;
                            return Ok(Transformation::Crop(x, y, w, h));
                        }
                    }
                }
                Err(format!("Could not parse {} into a transformation", s))
            }
            _ => Err(format!("Could not parse {} into a transformation", s)),
        }
    }
}

impl FromStr for TransformationList {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("Cannot parse an empty string".to_string());
        }

        let parts: Vec<&str> = s.split(',').collect();
        let mut result = Vec::with_capacity(parts.len());

        for part in parts {
            result.push(part.parse::<Transformation>()?);
        }

        Ok(TransformationList(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn background_color_encodes_as_expected() {
        assert_eq!("bgdeb836", Transformation::Background(0xdeb836).to_string());
    }

    #[test]
    fn background_color_decodes_as_expected() {
        assert_eq!(
            Ok(Transformation::Background(0xdeb836)),
            "bgdeb836".parse::<Transformation>()
        );
    }

    #[test]
    fn blur_encodes_as_expected() {
        assert_eq!("bl5", Transformation::Blur(5.0).to_string());
    }

    #[test]
    fn blur_decodes_as_expected() {
        assert_eq!(
            Ok(Transformation::Blur(5.0)),
            "bl5".parse::<Transformation>()
        );
    }

    #[test]
    fn scale_encodes_as_expected() {
        assert_eq!("s50", Transformation::Scale(50.0).to_string());
    }

    #[test]
    fn scale_decodes_as_expected() {
        assert_eq!(
            Ok(Transformation::Scale(50.0)),
            "s50".parse::<Transformation>()
        );
    }

    #[test]
    fn resize_encodes_as_expected() {
        assert_eq!("r128_256", Transformation::Resize(128, 256).to_string());
    }

    #[test]
    fn resize_decodes_as_expected() {
        assert_eq!(
            Ok(Transformation::Resize(128, 256)),
            "r128_256".parse::<Transformation>()
        );
    }

    #[test]
    fn crop_encodes_as_expected() {
        assert_eq!(
            "c0_1_128_256",
            Transformation::Crop(0, 1, 128, 256).to_string()
        )
    }

    #[test]
    fn crop_decodes_as_expected() {
        assert_eq!(
            Ok(Transformation::Crop(0, 1, 128, 256)),
            "c0_1_128_256".parse::<Transformation>()
        );
    }

    #[test]
    fn list_encodes_as_expected() {
        assert_eq!(
            "s10",
            TransformationList(vec![Transformation::Scale(10.0)]).to_string()
        );
        assert_eq!(
            "s10,s20",
            TransformationList(vec![
                Transformation::Scale(10.0),
                Transformation::Scale(20.0)
            ])
            .to_string()
        );
        assert_eq!(
            "s10,s20,s30",
            TransformationList(vec![
                Transformation::Scale(10.0),
                Transformation::Scale(20.0),
                Transformation::Scale(30.0)
            ])
            .to_string()
        );
    }

    #[test]
    fn list_decodes_as_expected() {
        assert_eq!(
            Ok(TransformationList(vec![Transformation::Scale(10.0)])),
            "s10".parse::<TransformationList>()
        );
        assert_eq!(
            Ok(TransformationList(vec![
                Transformation::Scale(10.0),
                Transformation::Scale(20.0)
            ])),
            "s10,s20".parse::<TransformationList>()
        );
        assert_eq!(
            Ok(TransformationList(vec![
                Transformation::Scale(10.0),
                Transformation::Scale(20.0),
                Transformation::Scale(30.0)
            ])),
            "s10,s20,s30".parse::<TransformationList>()
        );
    }
    #[test]
    fn many_tests_combined() {
        assert_eq!(
            Ok(TransformationList(vec![
                Transformation::Scale(50.0),
                Transformation::Blur(2.0),
                Transformation::Crop(0, 0, 128, 128),
                Transformation::Resize(256, 256)
            ])),
            "s50,bl2,c0_0_128_128,r256_256".parse::<TransformationList>()
        );
        assert_eq!(
            "s50,bl2,c0_0_128_128,r256_256",
            TransformationList(vec![
                Transformation::Scale(50.0),
                Transformation::Blur(2.0),
                Transformation::Crop(0, 0, 128, 128),
                Transformation::Resize(256, 256)
            ])
            .to_string()
        );
    }
}
