use crate::high_level::compiler::SourceSet;
use colored::*;
use pest::error::InputLocation;
use pest::iterators::Pair;
use pest::RuleType;
use std::cmp;
use std::iter::FromIterator;
use std::ops::Add;

#[derive(Clone, Debug)]
pub struct FilePosition {
    start_pos: usize,
    end_pos: usize,
    file: usize,
}

impl PartialEq for FilePosition {
    // This allows us to derive ParialEq implementations for other types that have a child of type
    // FilePosition. The equality of these tyeps are never dependent on their position in the file.
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl FilePosition {
    pub fn from_pair<R: RuleType>(pair: &Pair<R>, file: usize) -> FilePosition {
        let span = pair.as_span();
        FilePosition {
            start_pos: span.start(),
            end_pos: span.end(),
            file,
        }
    }

    pub fn from_input_location(location: InputLocation, file: usize) -> FilePosition {
        let (start_pos, end_pos) = match location {
            InputLocation::Span(poss) => poss,
            InputLocation::Pos(start) => (start, start + 1),
        };
        FilePosition {
            start_pos,
            end_pos,
            file,
        }
    }

    pub fn for_builtin(start: usize, end: usize) -> FilePosition {
        FilePosition {
            start_pos: start,
            end_pos: end,
            file: 0,
        }
    }

    pub fn placeholder() -> FilePosition {
        FilePosition {
            start_pos: 0,
            end_pos: 0,
            file: 0,
        }
    }

    pub fn union(parts: &[&FilePosition]) -> FilePosition {
        assert!(parts.len() > 0);
        let mut result = parts[0].clone();
        for part in &parts[1..] {
            result.include_other(part);
        }
        result
    }

    pub fn include<R: RuleType>(&mut self, pair: &Pair<R>) {
        let span = pair.as_span();
        self.start_pos = cmp::min(self.start_pos, span.start());
        self.end_pos = cmp::max(self.end_pos, span.end());
    }

    pub fn include_other(&mut self, other: &Self) {
        self.start_pos = cmp::min(self.start_pos, other.start_pos);
        self.end_pos = cmp::max(self.end_pos, other.end_pos);
    }

    pub fn create_line_column_ref(&self, sources: &SourceSet) -> String {
        let filename = &sources.borrow_source(self.file).0;
        // Everything that comes before the start of this position in the file.
        let before_text = &sources.borrow_source(self.file).1[0..self.start_pos];

        let mut line = 1;
        let mut last_newline_pos = 0;
        while let Some(value) = (&before_text[last_newline_pos..]).find("\n") {
            line += 1;
            last_newline_pos += value + 1;
        }
        let column = before_text.len() - last_newline_pos + 1;
        format!("{}:{}:{}", filename, line, column)
    }
}

pub(crate) enum ProblemType {
    Error,
    Hint,
}

pub(crate) struct ProblemDescriptor {
    position: FilePosition,
    ptype: ProblemType,
    caption: String,
}

impl ProblemDescriptor {
    pub(crate) fn new(
        position: FilePosition,
        ptype: ProblemType,
        caption: &str,
    ) -> ProblemDescriptor {
        ProblemDescriptor {
            position,
            ptype,
            caption: caption.to_owned(),
        }
    }
}

pub struct CompileProblem {
    descriptors: Vec<ProblemDescriptor>,
}

fn wrap_text(input: &str, width: usize, offset: usize) -> String {
    let mut output = "".to_owned();
    let mut word = "".to_owned();
    let mut line_length = offset;

    for ch in input.chars() {
        if ch == ' ' {
            let word_length = word.len();
            if line_length + word_length > width {
                output.push('\n');
                line_length = 0;
            }
            line_length += word_length + 1;
            output.push_str(&word);
            output.push(' ');
            word = "".to_owned();
        } else if ch == '\n' {
            output.push_str(&word);
            word = "".to_owned();
            output.push('\n');
            line_length = 0;
        } else {
            word.push(ch)
        }
    }
    if word.len() > 0 {
        let word_length = word.len();
        if line_length + word_length > width {
            output.push('\n');
        }
        output.push_str(&word);
    }
    output
}

struct GrabResult {
    prefix: String,
    highlighted: String,
    suffix: String,

    line_number: usize,
    column_number: usize,
}

impl CompileProblem {
    pub(crate) fn from_descriptors(descriptors: Vec<ProblemDescriptor>) -> CompileProblem {
        CompileProblem { descriptors }
    }

    pub(crate) fn add_descriptor(&mut self, descriptor: ProblemDescriptor) {
        self.descriptors.push(descriptor)
    }

    fn grab_text<'a>(from: &'a str, at: &FilePosition) -> GrabResult {
        let start_char = at.start_pos;
        let end_char = at.end_pos;
        let mut result = GrabResult {
            prefix: String::new(),
            highlighted: String::new(),
            suffix: String::new(),

            line_number: 0,
            column_number: 0,
        };
        let mut line_buffer: Vec<char> = Vec::new();
        let mut line = 1;
        let mut column = 0;

        for (index, character) in from.chars().enumerate() {
            column += 1;
            if index < start_char {
                line_buffer.push(character);
            } else if index == start_char {
                result.prefix = String::from_iter(line_buffer.iter());
                result.highlighted.push(character);
                result.line_number = line;
                result.column_number = column;
            } else if index > start_char && index < end_char {
                result.highlighted.push(character);
            } else if character != '\n' {
                result.suffix.push(character);
            }
            if character == '\n' {
                line_buffer.clear();
                line += 1;
                column = 0;
                if index >= end_char {
                    return result;
                }
            }
        }

        result
    }

    // This whole thing is a mess but it doesn't need to run fast.
    pub fn format(&self, width: usize, source_set: &SourceSet) -> String {
        let mut output = "".to_owned();
        for descriptor in self.descriptors.iter() {
            output.push_str(&match descriptor.ptype {
                ProblemType::Error => "ERROR: ".bright_red().to_string(),
                // ProblemType::Warning => "WARNING: ".bright_yellow().to_string(),
                ProblemType::Hint => "HINT: ".bright_cyan().to_string(),
            });
            output.push_str(&wrap_text(&descriptor.caption, width, 10));
            output.push_str("\n");

            let position = &descriptor.position;
            let source_name = &source_set.borrow_source(position.file).0;
            let grabbed = Self::grab_text(&source_set.borrow_source(position.file).1, position);
            let spacing = grabbed.line_number.to_string().len();
            let spaces = &format!("{: ^1$}", "", spacing + 2);
            output.push_str(&format!("{:-^1$}\n", "", width).blue().to_string());
            output.push_str(&format!(
                "{}{}{}{}:{}:{}\n",
                "|".blue().to_string(),
                spaces,
                "| ".blue().to_string(),
                source_name,
                grabbed.line_number,
                grabbed.column_number,
            ));
            output.push_str(&format!("{:-^1$}", "", width).blue().to_string());
            let highlight_start = grabbed.prefix.len();
            let highlight_end = highlight_start + grabbed.highlighted.len();
            let start_x = spacing + 5;
            let mut x = start_x;
            let mut line = grabbed.line_number;
            for (index, ch) in grabbed
                .prefix
                .add(&grabbed.highlighted.add(&grabbed.suffix))
                .chars()
                .enumerate()
            {
                if ch == '\n' || index == 0 {
                    output.push_str(
                        &format!("\n| {: >1$} | ", (line), spacing)
                            .blue()
                            .to_string(),
                    );
                    x = start_x;
                    line += 1;
                }
                if ch != '\n' {
                    if index < highlight_start || index >= highlight_end {
                        output.push(ch);
                    } else {
                        output.push_str(&match descriptor.ptype {
                            ProblemType::Error => ch.to_string().bright_red().to_string(),
                            // ProblemType::Warning => ch.to_string().bright_yellow().to_string(),
                            ProblemType::Hint => ch.to_string().bright_cyan().to_string(),
                        });
                    }
                    x += 1;
                    if x >= width {
                        output.push_str(&format!("\n|{}| ", spaces).blue().to_string());
                        x = start_x;
                    }
                }
            }
            output.push_str(&format!("\n{:-^1$}\n\n", "", width).blue().to_string());
        }
        output
    }
}
