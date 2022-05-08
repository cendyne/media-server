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

#[derive(Debug, PartialEq, Clone)]
pub enum Transformation {
    Scale(f32),
    Resize(u32, u32),
    Background(u32),
    Blur(f32),
    Crop(u32, u32, u32, u32),
    Noop,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransformationList(Vec<Transformation>);

impl TransformationList {
    pub fn list(self) -> Vec<Transformation> {
        self.0
    }
    pub fn empty() -> TransformationList {
        TransformationList(Vec::with_capacity(0))
    }
}

impl From<Vec<Transformation>> for TransformationList {
    fn from(item: Vec<Transformation>) -> Self {
        TransformationList(item)
    }
}

impl fmt::Display for Transformation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Transformation::Scale(factor) => write!(f, "s{}", factor),
            Transformation::Resize(w, h) => write!(f, "r{}_{}", w, h),
            Transformation::Background(color) => write!(f, "bg{:06x}", color),
            Transformation::Blur(sigma) => write!(f, "bl{}", sigma),
            Transformation::Crop(x, y, w, h) => write!(f, "c{}_{}_{}_{}", x, y, w, h),
            Transformation::Noop => write!(f, "id"),
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

impl<'r> rocket::form::FromFormField<'r> for TransformationList {
    fn from_value(field: rocket::form::ValueField<'r>) -> rocket::form::Result<'r, Self> {
        field
            .value
            .parse::<TransformationList>()
            .map_err(|err| rocket::form::Errors::from(rocket::form::Error::validation(err)))
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
            Some('i') => {
                if let Some(second) = chars.next() {
                    match second {
                        'd' => Ok(Transformation::Noop),
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
            return Ok(TransformationList(Vec::with_capacity(0)));
        }

        let parts: Vec<&str> = s.split(',').collect();
        let mut result = Vec::with_capacity(parts.len());

        for part in parts {
            result.push(part.parse::<Transformation>()?);
        }

        Ok(TransformationList(result))
    }
}

impl<'de> serde::Serialize for TransformationList {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for TransformationList
where
    Self: std::str::FromStr,
    <Self as std::str::FromStr>::Err: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Helper<S>(std::marker::PhantomData<S>);

        impl<'de, S> serde::de::Visitor<'de> for Helper<S>
        where
            S: std::str::FromStr,
            <S as std::str::FromStr>::Err: std::fmt::Display,
        {
            type Value = S;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "string")
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value
                    .parse::<Self::Value>()
                    .map_err(serde::de::Error::custom)
            }

            fn visit_bytes<E>(self, value: &[u8]) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let utf8 = std::str::from_utf8(value).map_err(serde::de::Error::custom)?;
                self.visit_str(utf8)
            }
        }

        deserializer.deserialize_str(Helper(std::marker::PhantomData))
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
    fn noop_encodes_as_expected() {
        assert_eq!("id", Transformation::Noop.to_string());
    }

    #[test]
    fn noop_decodes_as_expected() {
        assert_eq!(Ok(Transformation::Noop), "id".parse::<Transformation>());
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
