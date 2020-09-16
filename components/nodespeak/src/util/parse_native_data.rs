extern crate pest;

use pest::iterators::Pair;
use pest::Parser;

use crate::trivial::structure::KnownData;
use crate::vague::structure::KnownData as VagueKnownData;

#[derive(Parser)]
#[grammar = "util/known_data.pest"]
struct KnownDataParser;

fn parse_literal(source: Pair<Rule>) -> KnownData {
    match source.as_rule() {
        Rule::neg_float => KnownData::Float(
            -parse_literal(source.into_inner().next().expect("Required by grammar."))
                .require_float(),
        ),
        Rule::neg_int => KnownData::Int(
            -parse_literal(source.into_inner().next().expect("Required by grammar.")).require_int(),
        ),
        Rule::float => KnownData::Float(
            source
                .as_str()
                .replace("_", "")
                .parse()
                .expect("Valid float required by grammar."),
        ),
        Rule::dec_int => KnownData::Int(
            source
                .as_str()
                .replace("_", "")
                .parse()
                .expect("Valid int required by grammar."),
        ),
        Rule::hex_int => KnownData::Int(
            i64::from_str_radix(&source.as_str().replace("_", "")[2..], 16)
                .expect("Grammar requires valid hexadecimal int."),
        ),
        Rule::oct_int => KnownData::Int(
            i64::from_str_radix(&source.as_str().replace("_", "")[2..], 8)
                .expect("Grammar requires valid octal int."),
        ),
        Rule::legacy_oct_int => KnownData::Int(
            i64::from_str_radix(&source.as_str().replace("_", "")[1..], 8)
                .expect("Grammar requires valid octal int."),
        ),
        Rule::bin_int => KnownData::Int(
            i64::from_str_radix(&source.as_str().replace("_", "")[2..], 2)
                .expect("Grammar requires valid binary int."),
        ),
        Rule::bool_true => KnownData::Bool(true),
        Rule::bool_false => KnownData::Bool(false),
        Rule::array_literal => {
            let children: Vec<_> = source
                .into_inner()
                .map(|child| parse_literal(child))
                .collect();
            if children.len() == 0 {
                panic!("TODO: Nice error, an array cannot have no children.");
            } else {
                KnownData::Array(children)
            }
        }
        _ => {
            eprintln!("{}", source);
            unreachable!("No other possible children in grammar.")
        }
    }
}

fn convert_native_to_vague(data: &KnownData) -> VagueKnownData {
    match data {
        KnownData::Bool(value) => VagueKnownData::Bool(*value),
        KnownData::Int(value) => VagueKnownData::Int(*value),
        KnownData::Float(value) => VagueKnownData::Float(*value),
        KnownData::Array(values) => {
            let new_values = values.iter().map(convert_native_to_vague).collect();
            VagueKnownData::Array(new_values)
        }
    }
}

pub fn parse_known_data(source: &str) -> Result<VagueKnownData, String> {
    let mut parse_result = match KnownDataParser::parse(Rule::root, source) {
        Result::Ok(result) => result,
        Result::Err(err) => {
            return Result::Err(format!("{}", err));
        }
    };
    let literal = parse_result
        .next()
        .expect("Grammar requires a result.")
        .into_inner()
        .next()
        .expect("Grammar requires a literal.");
    let native = parse_literal(literal);
    Result::Ok(convert_native_to_vague(&native))
}

pub fn parse_native_data(source: &str) -> Result<KnownData, String> {
    let mut parse_result = match KnownDataParser::parse(Rule::root, source) {
        Result::Ok(result) => result,
        Result::Err(err) => {
            return Result::Err(format!("{}", err));
        }
    };
    let literal = parse_result
        .next()
        .expect("Grammar requires a result.")
        .into_inner()
        .next()
        .expect("Grammar requires a literal.");
    Result::Ok(parse_literal(literal))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn int_literal() -> Result<(), String> {
        parse_known_data("40189")?;
        parse_known_data("94901")?;
        parse_known_data("-110298")?;
        parse_known_data("901489")?;
        parse_known_data("   909814  ")?;
        Result::Ok(())
    }

    #[test]
    fn float_literal() -> Result<(), String> {
        parse_known_data("4180.")?;
        parse_known_data(".9901824")?;
        parse_known_data("-1901824.490182e-42")?;
        parse_known_data("248e69")?;
        parse_known_data("   -0.94981  ")?;
        Result::Ok(())
    }
}
