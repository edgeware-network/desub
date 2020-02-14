// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of substrate-desub.
//
// substrate-desub is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// substrate-desub is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-desub.  If not, see <http://www.gnu.org/licenses/>.

use core::RustTypeMarker;
use regex::{Regex, RegexSet};

/// Internal C-Like enum for indexing into a RegexSet
enum RegexType {
    Array = 0,
    Vec = 1,
    Compact = 2,
    Option = 3,
    Generic = 4,
    Tuple = 5
}

/// Match a rust array
pub fn rust_array_decl() -> Regex {
    // width of number and unsigned/signed are all in their own capture group
    // size of array is in the last capture group
    Regex::new(r"^\[(?P<type>[uif]{1})(?P<bit8>8)?(?P<bit16>16)?(?P<bit32>32)?(?P<bit64>64)?(?P<bit128>128)?;\s?(?P<size>[\d]*)]$")
        .expect("Regex expression invalid")
}

/// Match a rust vector
/// allowed to be nested within, or have other (ie Option<>) nested within
pub fn rust_vec_decl() -> Regex {
    Regex::new(r"^Vec<(?P<type>[\w><]+)>$")
        .expect("Regex expression should be infallible; qed")
}

/// Match a Rust Option
/// Allowed to be nested within another type, or have other (ie Vec<>) nested
/// within
pub fn rust_option_decl() -> Regex {
    Regex::new(r"^Option<(?P<type>[\w><]+)>$")
        .expect("Regex expression should be infallible; qed")
}

/// Match a parity-scale-codec Compact<T> type
pub fn rust_compact_decl() -> Regex {
    Regex::new(r"^Compact<(?P<type>[\w><]+)>$")
        .expect("Regex expression should be infallible; qed")
}

/// Match a Rust Generic Type Declaration
/// Excudes types Vec/Option/Compact from matches
pub fn rust_generic_decl() -> Regex {
    Regex::new(r"\b(?!(?:Vec|Option|Compact)\b)(?P<outer_type>\w+)<(?P<inner_type>[\w<>]+)>")
        .expect("Regex expressions should be infallible; qed")
}

/// Only captures text within the tuples,
/// need to use 'Matches' (ie `find_iter`) iterator to get all matches
pub fn rust_tuple_decl() -> Regex {
    Regex::new(r"[\w><]+").expect("Regex expression should be infallible; qed")
}

pub fn rust_regex_set() -> RegexSet {
    RegexSet::new(&[
        rust_array_decl().as_str(),
        rust_vec_decl().as_str(),
        rust_compact_decl().as_str(),
        rust_option_decl().as_str(),
        rust_generic_decl().as_str(),
        rust_tuple_decl().as_str(),
    ])
    .expect("Regex expression should be infallible; qed")
}

/// Parse a known match to the array regular expression
///
/// # Panics
/// If the match is incorrect
pub fn parse_regex_array(s: &str) -> Option<RustTypeMarker> {
    let re = rust_array_decl();
    if !re.is_match(s) {
        return None;
    }
    let caps = re.captures(s).expect("checked for array declaration; ");

    let t = caps
        .name("type")
        .expect("type match should always exist")
        .as_str();
    let size = caps.name("size").expect("name match should always exist");
    let caps = caps
        .iter()
        .map(|c| c.map(|c| c.as_str()))
        .collect::<Vec<Option<&str>>>();

    let ty = if caps[2].is_some() {
        match t {
            "u" => RustTypeMarker::U8,
            "i" => RustTypeMarker::I8,
            "f" => panic!("type does not exist 'f8'"),
            _ => panic!("impossible match encountered"),
        }
    } else if caps[3].is_some() {
        match t {
            "u" => RustTypeMarker::U16,
            "i" => RustTypeMarker::I16,
            "f" => panic!("type does not exist 'f16'"),
            _ => panic!("impossible match encountered"),
        }
    } else if caps[4].is_some() {
        match t {
            "u" => RustTypeMarker::U32,
            "i" => RustTypeMarker::I32,
            "f" => RustTypeMarker::F32,
            _ => panic!("impossible match encountered"),
        }
    } else if caps[5].is_some() {
        match t {
            "u" => RustTypeMarker::U64,
            "i" => RustTypeMarker::I64,
            "f" => RustTypeMarker::F64,
            _ => panic!("impossible match encountered"),
        }
    } else if caps[6].is_some() {
        match t {
            "u" => RustTypeMarker::U128,
            "i" => RustTypeMarker::I128,
            "f" => panic!("type does not exist: 'f128'"),
            _ => panic!("impossible match encountered"),
        }
    } else {
        panic!("Couldn't determine size of array");
    };
    let ty = Box::new(ty);
    let size = size
        .as_str()
        .parse::<usize>()
        .expect("Should always be number");
    Some(RustTypeMarker::Array { size, ty })
}
/*
/// recursively parses a regex set
/// returning a RustTypeMarker with all matched types
pub fn parse_regex_set(s: &str) -> Option<RustTypeMarker> {
    let re = rust_regex_set();

    if !re.is_match(s) {
        return None;
    }
    let matches: Vec<_> = re.matches(s);

   

    let caps = re.captures(s).expect("Checked for match; qed");
    let t = caps.name("type").expect("type will always be present; qed");
    // this gotta be recursive
    let re_set = rust_regex_set();
    let matches: Vec<_> = re_set.matches(t);
    None
}
 */

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Captures;

    fn caps_to_vec_str<'a>(caps: Captures<'a>) -> Vec<Option<&'a str>> {
        caps.iter()
            .map(|c| c.map(|c| c.as_str()))
            .collect::<Vec<Option<&str>>>()
    }

    #[test]
    fn should_match_array_decl() {
        let re = rust_array_decl();
        assert!(re.is_match("[u8; 16]"));
        assert!(re.is_match("[u16; 16]"));
        assert!(re.is_match("[u32; 16]"));
        assert!(re.is_match("[u64; 16]"));
        assert!(re.is_match("[u128; 16]"));
        assert!(re.is_match("[i8; 16]"));
        assert!(re.is_match("[i16; 16]"));
        assert!(re.is_match("[i32; 16]"));
        assert!(re.is_match("[i64; 16]"));
        assert!(re.is_match("[i128; 16]"));
        assert!(re.is_match("[u16; 128]"));
        assert!(re.is_match("[u32; 64]"));
        assert!(re.is_match("[u64; 99999]"));
        assert!(re.is_match("[u128; 23]"));
    }

    #[test]
    fn should_seperate_args_in_capture_groups() {
        let re = rust_array_decl();

        let caps = caps_to_vec_str(re.captures("[u8; 16]").unwrap());
        assert_eq!(
            vec![
                Some("[u8; 16]"),
                Some("u"),
                Some("8"),
                None,
                None,
                None,
                None,
                Some("16")
            ],
            caps
        );

        let caps = caps_to_vec_str(re.captures("[i8; 16]").unwrap());
        assert_eq!(
            vec![
                Some("[i8; 16]"),
                Some("i"),
                Some("8"),
                None,
                None,
                None,
                None,
                Some("16")
            ],
            caps
        );

        let caps = caps_to_vec_str(re.captures("[u16; 16]").unwrap());
        assert_eq!(
            vec![
                Some("[u16; 16]"),
                Some("u"),
                None,
                Some("16"),
                None,
                None,
                None,
                Some("16")
            ],
            caps
        );

        let caps = caps_to_vec_str(re.captures("[i16; 16]").unwrap());
        assert_eq!(
            vec![
                Some("[i16; 16]"),
                Some("i"),
                None,
                Some("16"),
                None,
                None,
                None,
                Some("16")
            ],
            caps
        );

        let caps = caps_to_vec_str(re.captures("[u32; 16]").unwrap());
        assert_eq!(
            vec![
                Some("[u32; 16]"),
                Some("u"),
                None,
                None,
                Some("32"),
                None,
                None,
                Some("16")
            ],
            caps
        );

        let caps = caps_to_vec_str(re.captures("[i32; 16]").unwrap());
        assert_eq!(
            vec![
                Some("[i32; 16]"),
                Some("i"),
                None,
                None,
                Some("32"),
                None,
                None,
                Some("16")
            ],
            caps
        );

        let caps = caps_to_vec_str(re.captures("[u64; 16]").unwrap());
        assert_eq!(
            vec![
                Some("[u64; 16]"),
                Some("u"),
                None,
                None,
                None,
                Some("64"),
                None,
                Some("16")
            ],
            caps
        );

        let caps = caps_to_vec_str(re.captures("[i64; 16]").unwrap());
        assert_eq!(
            vec![
                Some("[i64; 16]"),
                Some("i"),
                None,
                None,
                None,
                Some("64"),
                None,
                Some("16")
            ],
            caps
        );

        let caps = caps_to_vec_str(re.captures("[u128; 16]").unwrap());
        assert_eq!(
            vec![
                Some("[u128; 16]"),
                Some("u"),
                None,
                None,
                None,
                None,
                Some("128"),
                Some("16")
            ],
            caps
        );

        let caps = caps_to_vec_str(re.captures("[i128; 16]").unwrap());
        assert_eq!(
            vec![
                Some("[i128; 16]"),
                Some("i"),
                None,
                None,
                None,
                None,
                Some("128"),
                Some("16")
            ],
            caps
        );

        let caps = caps_to_vec_str(re.captures("[f32; 16]").unwrap());
        assert_eq!(
            vec![
                Some("[f32; 16]"),
                Some("f"),
                None,
                None,
                Some("32"),
                None,
                None,
                Some("16")
            ],
            caps
        );

        let caps = caps_to_vec_str(re.captures("[f64; 16]").unwrap());
        assert_eq!(
            vec![
                Some("[f64; 16]"),
                Some("f"),
                None,
                None,
                None,
                Some("64"),
                None,
                Some("16")
            ],
            caps
        );

        let caps = caps_to_vec_str(re.captures("[i128; 9999]").unwrap());
        assert_eq!(
            vec![
                Some("[i128; 9999]"),
                Some("i"),
                None,
                None,
                None,
                None,
                Some("128"),
                Some("9999")
            ],
            caps
        );

        let caps = caps_to_vec_str(re.captures("[u128; 9999]").unwrap());
        assert_eq!(
            vec![
                Some("[u128; 9999]"),
                Some("u"),
                None,
                None,
                None,
                None,
                Some("128"),
                Some("9999")
            ],
            caps
        );
    }

    #[test]
    fn should_parse_vecs() {
        let re = rust_vec_decl();
        assert!(re.is_match("Vec<RuntimeVersionApi>"));
        assert!(re.is_match("Vec<BlockNumber>"));
        assert!(re.is_match("Vec<SomeStruct>"));
    }

    #[test]
    fn should_get_type_of_vec() {
        let re = rust_vec_decl();
        let caps = caps_to_vec_str(re.captures("Vec<RuntimeVersionApi>").unwrap());
        // first capture group is always entire expression
        assert!(caps[1] == Some("RuntimeVersionApi"));
    }

    #[test]
    fn should_parse_compact() {
        let re = rust_compact_decl();
        assert!(re.is_match("Compact<RuntimeVersionApi>"));
        assert!(re.is_match("Compact<BlockNumber>"));
        assert!(re.is_match("Compact<SomeStruct>"));
    }

    #[test]
    fn should_get_type_of_compact() {
        let re = rust_compact_decl();
        let caps = caps_to_vec_str(re.captures("Compact<RuntimeVersionApi>").unwrap());
        // first capture group is always entire expression
        assert!(caps[1] == Some("RuntimeVersionApi"));
    }

    #[test]
    fn should_parse_options() {
        let re = rust_option_decl();
        assert!(re.is_match("Option<RuntimeVersionApi>"));
        assert!(re.is_match("Option<BlockNumber>"));
        assert!(re.is_match("Option<SomeStruct>"));
        assert!(re.is_match("Option<Vec<SomeStruct>>"));
    }

    #[test]
    fn should_get_type_of_option() {
        let re = rust_option_decl();
        let caps = caps_to_vec_str(re.captures("Option<RuntimeVersionApi>").unwrap());
        // first capture group is always entire expression
        assert!(caps[1] == Some("RuntimeVersionApi"));

        let re = rust_option_decl();
        let caps =
            caps_to_vec_str(re.captures("Option<Vec<RuntimeVersionApi>>").unwrap());
        assert!(caps[1] == Some("Vec<RuntimeVersionApi>"));
    }

    #[test]
    fn should_match_arbitrary_types() {
        let re = rust_generic_decl();
        assert!(re.is_match("GenericOuterType<GenericInnerType>"));
        assert!(re.is_match("GenericOutT<GenericOutInT<InnerT>>"));
    }

    #[test]
    fn should_get_arbitrary_type() {
        let re = rust_generic_decl();
        let caps =
            caps_to_vec_str(re.captures("GenericOuterType<GenericInnerType>").unwrap());
        assert_eq!(
            vec![
                Some("GenericOuterType<GenericInnerType>"),
                Some("GenericOuterType"),
                Some("GenericInnerType")
            ],
            caps
        );
    }

    #[test]
    fn should_parse_tuples() {
        let re = rust_tuple_decl();
        assert!(re.is_match("(StorageKey, Option<StorageData>)"));
        assert!(re.is_match("(ApiKey, u32)"));
        assert!(re.is_match("(u32,ApiKey,AnotherType)"));
    }

    #[test]
    fn should_get_types_in_tuple() {
        let re = rust_tuple_decl();
        let match_str = "(StorageKey, Option<StorageData>)";
        let types = re
            .find_iter(match_str)
            .map(|m| match_str[m.start() .. m.end()].to_string())
            .collect::<Vec<String>>();
        assert_eq!(
            vec!["StorageKey".to_string(), "Option<StorageData>".to_string()],
            types
        );
    }

    #[test]
    fn should_match_with_regex_set() {
        let set = rust_regex_set();
        assert!(set.is_match("(StorageKey, Option<StorageData>)"));
        assert!(set.is_match("GenericOuterType<GenericInnerType>"));
        assert!(set.is_match("[u8; 16]"));
        assert!(set.is_match("Vec<InnerType>"));
        assert!(set.is_match("Option<InnerType>"));
    }

    #[test]
    fn should_get_all_matches() {
        let set = rust_regex_set();
        let matches: Vec<_> = set.matches("[u8; 16]").into_iter().collect();
        // matches array decl and tuple type (tuple type will match on anything, should
        // not be trusted) TODO: create better regex for tuple type
        assert_eq!(vec![0, 5], matches);
    }
}
