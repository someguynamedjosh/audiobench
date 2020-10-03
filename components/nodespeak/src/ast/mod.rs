use crate::high_level::problem::{CompileProblem, FilePosition};

use pest::error::ErrorVariant;
use pest::Parser;

#[derive(Parser)]
#[grammar = "ast/grammar.pest"]
struct NodespeakParser;

pub mod structure {
    pub use super::Rule;
    use pest::iterators::{Pair, Pairs};

    pub type Program<'a> = Pairs<'a, Rule>;
    pub type Node<'a> = Pair<'a, Rule>;

    pub fn rule_name(rule: &Rule) -> &'static str {
        match rule {
            Rule::WHITESPACE => "whitespace",
            Rule::COMMENT | Rule::block_comment | Rule::line_comment => "comment",
            Rule::EOI => "end of file",

            Rule::dec_int => "integer literal",
            Rule::hex_int => "hexadecimal literal",
            Rule::oct_int => "octal literal",
            Rule::legacy_oct_int => "c-style octal literal",
            Rule::bin_int => "binary literal",
            Rule::dec_digit => "digit",
            Rule::float => "float literal",
            Rule::int => "int literal",
            Rule::literal => "literal value",
            Rule::identifier => "identifier",

            Rule::vp_var => "variable",
            Rule::build_array => "array data",
            Rule::vpe_part_1 | Rule::vpe_part_2 | Rule::vpe_part_3 | Rule::vpe_part | Rule::vpe => {
                "value-producing expression"
            }
            Rule::build_array_type => "array type",
            Rule::optional_index_indicator => "?",
            Rule::vp_index => "index expression",
            Rule::get_property => "property access",
            Rule::negate => "negate",
            Rule::not => "not",
            Rule::operator => "binary operator",

            Rule::var_dec => "variable declaration",
            Rule::vc_identifier => "variable",
            Rule::vc_index => "index expression",
            Rule::vce => "value-consuming expression",

            Rule::macro_call_input_list => "input list for macro call",
            Rule::inline_output => "inline keyword",
            Rule::macro_call_output => "output for macro call",
            Rule::macro_call_output_list => "output list for macro call",
            Rule::macro_call => "single-output macro callession",

            Rule::macro_inputs => "input list for macro definition",
            Rule::macro_outputs => "output list for macro definition",
            Rule::single_macro_output => "single output for macro definition",
            Rule::macro_signature => "signature for macro definition",
            Rule::macro_definition => "macro definition",

            Rule::else_if_clause => "else if clause",
            Rule::else_clause => "else clause",
            Rule::if_statement => "if statement",
            Rule::no_unroll_keyword => "no_unroll (keyword)",
            Rule::for_loop_statement => "for loop",

            Rule::raw_string => "string segment",
            Rule::escape_sequence => "string escape sequence",
            Rule::string => "string literal",

            Rule::input_variable_statement => "input declaration",
            Rule::output_variable_statement => "output declaration",
            Rule::static_variable_statement => "static variable declaration",
            Rule::assign_statement => "assignment statement",
            Rule::macro_call_statement => "macro call as statement",
            Rule::var_dec_statement => "variable declaration as statement",
            Rule::return_statement => "return statement",
            Rule::assert_statement => "assert statement",
            Rule::include_statement => "include statement",
            Rule::statement => "statement",

            Rule::code_block => "code block",

            Rule::root => "program",
        }
    }
}

pub(self) mod problems {
    use crate::high_level::problem::{
        CompileProblem, FilePosition, ProblemDescriptor, ProblemType,
    };

    pub fn bad_syntax(pos: FilePosition, message: String) -> CompileProblem {
        CompileProblem::from_descriptors(vec![
            ProblemDescriptor::new(pos, ProblemType::Error, &format!("Bad Syntax\n{}", message)),
        ])
    }
}

pub fn ingest(text: &str, file_id: usize) -> Result<structure::Program, CompileProblem> {
    NodespeakParser::parse(Rule::root, text).map_err(|parse_err| {
        problems::bad_syntax(
            FilePosition::from_input_location(parse_err.location, file_id),
            match parse_err.variant {
                ErrorVariant::ParsingError {
                    positives,
                    negatives,
                } => format!(
                    "Expected {}... but found {}.",
                    {
                        positives
                            .iter()
                            .map(|rule| structure::rule_name(rule))
                            .collect::<Vec<&str>>()
                            .join(", ")
                    },
                    {
                        if negatives.len() == 0 {
                            "unknown syntax".to_owned()
                        } else {
                            negatives
                                .iter()
                                .map(|rule| structure::rule_name(rule))
                                .collect::<Vec<&str>>()
                                .join(", ")
                        }
                    }
                ),
                ErrorVariant::CustomError { message: _message } => {
                    unreachable!("Only parsing errors are encountered in the parser.")
                }
            },
        )
    })
}
