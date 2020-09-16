use super::{problems, VagueIngester};
use crate::ast::structure as i;
use crate::high_level::problem::{CompileProblem, FilePosition};
use crate::vague::structure as o;

impl<'a> VagueIngester<'a> {
    pub(super) fn convert_vc_index(
        &mut self,
        node: i::Node,
    ) -> Result<o::VCExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::vc_index);
        let position = self.make_position(&node);
        let mut children = node.into_inner();
        let base_node = children.next().expect("bad AST");
        let base = self.convert_vc_identifier(base_node)?;

        let mut indexes = Vec::new();
        for child in children {
            if child.as_rule() == i::Rule::vpe {
                indexes.push((self.convert_vpe(child)?, false));
            } else if child.as_rule() == i::Rule::optional_index_indicator {
                // Turns out the previous index is actually optional.
                let last = indexes.len() - 1;
                indexes[last].1 = true;
            } else {
                unreachable!("bad AST");
            }
        }
        Ok(o::VCExpression::Index {
            base: Box::new(base),
            indexes,
            position,
        })
    }

    pub(super) fn convert_var_dec(
        &mut self,
        node: i::Node,
    ) -> Result<o::VCExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::var_dec);
        let position = self.make_position(&node);
        let mut children = node.into_inner();

        let var_type = self.convert_vpe(children.next().expect("bad AST"))?;
        let name = children.next().expect("bad AST").as_str();
        let var_id = self.create_variable(var_type, name, position.clone());
        Ok(o::VCExpression::Variable(var_id, position))
    }

    pub(super) fn convert_vc_identifier(
        &mut self,
        node: i::Node,
    ) -> Result<o::VCExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::vc_identifier);
        let position = self.make_position(&node);
        let child = node.into_inner().next().expect("bad AST");
        let var_id = self.lookup_identifier(&child)?;
        if self.target[var_id].is_read_only() {
            return Err(problems::write_to_read_only_variable(
                position,
                child.as_str(),
            ));
        }
        Ok(o::VCExpression::Variable(var_id, position))
    }

    pub(super) fn convert_vce(&mut self, node: i::Node) -> Result<o::VCExpression, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::vce);
        let child = node.into_inner().next().expect("bad AST");
        match child.as_rule() {
            i::Rule::vc_index => self.convert_vc_index(child),
            i::Rule::var_dec => self.convert_var_dec(child),
            i::Rule::vc_identifier => self.convert_vc_identifier(child),
            _ => unreachable!("bad AST"),
        }
    }
}
