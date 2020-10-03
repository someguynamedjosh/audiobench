use super::problems;
use super::VagueIngester;
use crate::ast::structure as i;
use crate::high_level::problem::{CompileProblem, FilePosition};
use crate::vague::structure as o;

impl<'a> VagueIngester<'a> {
    pub(super) fn convert_macro_signature<'n>(
        &mut self,
        node: i::Node<'n>,
    ) -> Result<(Vec<o::VariableId>, Vec<i::Node<'n>>), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::macro_signature);
        let mut children = node.into_inner();
        let inputs_node = children.next().expect("bad AST");
        let outputs_node = children.next();
        let outputs = outputs_node
            .map(|node| node.into_inner().collect())
            .unwrap_or_default();
        let input_ids = inputs_node
            .into_inner()
            .map(|child| {
                debug_assert!(child.as_rule() == i::Rule::identifier);
                let name = child.as_str();
                let var = o::Variable::variable(self.make_position(&child), None);
                self.target
                    .adopt_and_define_symbol(self.current_scope, name, var)
            })
            .collect();
        Ok((input_ids, outputs))
    }

    pub(super) fn convert_macro_definition(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::macro_definition);
        let position = self.make_position(&node);
        let mut children = node.into_inner();
        let macro_name_node = children.next().expect("bad AST");
        let signature_node = children.next().expect("bad AST");
        let header_pos = {
            let mut pos = self.make_position(&macro_name_node);
            pos.include(&signature_node);
            pos
        };
        let body_node = children.next().expect("bad AST");

        let body_scope = self.target.create_child_scope(self.current_scope);
        let old_current_scope = self.current_scope;
        self.current_scope = body_scope;

        let (input_ids, output_nodes) = self.convert_macro_signature(signature_node)?;
        for id in input_ids {
            self.target[self.current_scope].add_input(id);
        }
        self.convert_code_block(body_node)?;
        let macro_name = macro_name_node.as_str();
        for node in output_nodes {
            let name = node.as_str();
            if let Some(id) = self.lookup_identifier_without_error(name) {
                self.target[self.current_scope].add_output(id);
            } else {
                let pos = self.make_position(&node);
                return Err(problems::missing_output_definition(pos, macro_name, &name));
            };
        }

        self.current_scope = old_current_scope;
        let var = o::Variable::macro_def(o::MacroData::new(body_scope, header_pos));
        let var_id = self
            .target
            .adopt_and_define_symbol(self.current_scope, macro_name, var);
        self.add_statement(o::Statement::CreationPoint {
            var: var_id,
            var_type: Box::new(o::VPExpression::Literal(
                o::KnownData::DataType(o::SpecificDataType::Macro.into()),
                FilePosition::placeholder(),
            )),
            position,
        });
        Ok(())
    }

    pub(super) fn convert_code_block(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        self.enter_scope();
        debug_assert!(node.as_rule() == i::Rule::code_block);
        for child in node.into_inner() {
            self.convert_statement(child)?;
        }
        self.exit_scope();
        Ok(())
    }

    pub(super) fn convert_code_block_in_new_scope(
        &mut self,
        node: i::Node,
    ) -> Result<o::ScopeId, CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::code_block);
        self.enter_scope();
        let new_scope = self.target.create_child_scope(self.current_scope);
        let old_scope = self.current_scope;
        self.current_scope = new_scope;
        for child in node.into_inner() {
            self.convert_statement(child)?;
        }
        self.current_scope = old_scope;
        self.exit_scope();
        Ok(new_scope)
    }

    pub(super) fn convert_return_statement(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::return_statement);
        let position = self.make_position(&node);
        if self.current_scope == self.target.get_entry_point() {
            Err(problems::return_from_root(position))
        } else {
            self.add_statement(o::Statement::Return(position));
            Ok(())
        }
    }

    pub(super) fn convert_assert_statement(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::assert_statement);
        let position = self.make_position(&node);
        let mut children = node.into_inner();
        let condition_node = children.next().expect("bad AST");
        let condition = self.convert_vpe(condition_node)?;
        self.add_statement(o::Statement::Assert(Box::new(condition), position));
        Ok(())
    }

    pub(super) fn convert_if_statement(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::if_statement);
        let position = self.make_position(&node);
        let mut children = node.into_inner();
        let primary_condition = self.convert_vpe(children.next().expect("bad AST"))?;
        let primary_body_node = children.next().expect("bad AST");
        let primary_body = self.convert_code_block_in_new_scope(primary_body_node)?;

        let mut clauses = vec![(primary_condition, primary_body)];
        let mut else_clause = None;
        for child in children {
            match child.as_rule() {
                i::Rule::else_if_clause => {
                    // Else if clauses should only show up before the else clause.
                    debug_assert!(else_clause.is_none(), "bad AST");
                    let mut children = child.into_inner();
                    let condition_node = children.next().expect("bad AST");
                    let condition = self.convert_vpe(condition_node)?;
                    let body_node = children.next().expect("bad AST");
                    let body = self.convert_code_block_in_new_scope(body_node)?;
                    clauses.push((condition, body));
                }
                i::Rule::else_clause => {
                    debug_assert!(else_clause.is_none(), "bad AST");
                    let mut children = child.into_inner();
                    let body_node = children.next().expect("bad AST");
                    let body = self.convert_code_block_in_new_scope(body_node)?;
                    else_clause = Some(body);
                }
                _ => unreachable!("bad AST"),
            }
        }

        self.add_statement(o::Statement::Branch {
            clauses,
            else_clause,
            position,
        });
        Ok(())
    }

    pub(super) fn convert_for_loop_statement(
        &mut self,
        node: i::Node,
    ) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::for_loop_statement);
        let position = self.make_position(&node);
        let mut children = node.into_inner();
        let counter_node = children.next().expect("bad AST");
        let counter_pos = self.make_position(&counter_node);
        let counter_name = counter_node.as_str();
        let start = self.convert_vpe(children.next().expect("bad AST"))?;
        let end = self.convert_vpe(children.next().expect("bad AST"))?;
        let body_scope = self.target.create_child_scope(self.current_scope);
        let counter = o::Variable::variable(counter_pos.clone(), None);
        let counter_id = self
            .target
            .adopt_and_define_symbol(body_scope, counter_name, counter);
        let old_current_scope = self.current_scope;
        self.current_scope = body_scope;

        let possibly_body = children.next().expect("bad AST");
        let (body, allow_unroll) = if possibly_body.as_rule() == i::Rule::no_unroll_keyword {
            (children.next().expect("bad AST"), false)
        } else {
            (possibly_body, true)
        };
        self.convert_code_block(body)?;
        self.current_scope = old_current_scope;
        self.add_statement(o::Statement::ForLoop {
            allow_unroll,
            counter: counter_id,
            start: Box::new(start),
            end: Box::new(end),
            body: body_scope,
            position,
        });
        Ok(())
    }

    pub(super) fn convert_input_variable_statement(
        &mut self,
        node: i::Node,
    ) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::input_variable_statement);
        if self.current_scope != self.target.get_entry_point() {
            return Err(problems::io_inside_macro(self.make_position(&node)));
        }
        let mut children = node.into_inner();
        let data_type = self.convert_vpe(children.next().expect("bad AST"))?;
        for child in children {
            let pos = self.make_position(&child);
            let name = child.as_str();
            let var_id = self.create_variable(data_type.clone(), name, pos);
            self.target[self.current_scope].add_input(var_id);
        }
        Ok(())
    }

    pub(super) fn convert_output_variable_statement(
        &mut self,
        node: i::Node,
    ) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::output_variable_statement);
        if self.current_scope != self.target.get_entry_point() {
            return Err(problems::io_inside_macro(self.make_position(&node)));
        }
        let mut children = node.into_inner();
        let data_type = self.convert_vpe(children.next().expect("bad AST"))?;
        for child in children {
            let pos = self.make_position(&child);
            let name = child.as_str();
            let var_id = self.create_variable(data_type.clone(), name, pos);
            self.target[self.current_scope].add_output(var_id);
        }
        Ok(())
    }

    // TODO: This will result in multiple static variables being created if a function is called
    // multiple times. Or maybe don't fix it? Maybe make it a feature?
    pub(super) fn convert_static_variable_statement(
        &mut self,
        node: i::Node,
    ) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::static_variable_statement);
        let mut exported_vars = Vec::new();
        for child in node.into_inner() {
            if child.as_rule() == i::Rule::identifier {
                let pos = FilePosition::from_pair(&child, self.current_file_id);
                let name = child.as_str().to_owned();
                exported_vars.push((name, pos));
            } else if child.as_rule() == i::Rule::code_block {
                let static_scope = self.target.create_child_scope(self.current_scope);
                let old_scope = self.current_scope;
                self.current_scope = static_scope;
                self.convert_code_block(child)?;
                let mut exported_ids = Vec::new();
                for (name, pos) in exported_vars {
                    if let Some(id) = self.lookup_identifier_without_error(&name) {
                        self.target[old_scope].define_symbol(&name, id);
                        exported_ids.push(id);
                    } else {
                        return Err(problems::missing_export_definition(pos, &name));
                    }
                }
                self.current_scope = old_scope;
                self.add_statement(o::Statement::StaticInit {
                    body: static_scope,
                    exports: exported_ids,
                    position: FilePosition::placeholder(), // TODO: Better position.
                });
                return Ok(());
            } else {
                unreachable!("bad AST");
            }
        }
        unreachable!("bad AST");
    }

    pub(super) fn convert_assign_statement(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::assign_statement);
        let position = self.make_position(&node);
        let mut children = node.into_inner();
        let vce = self.convert_vce(children.next().expect("bad AST"))?;
        let vpe = self.convert_vpe(children.next().expect("bad AST"))?;
        self.add_statement(o::Statement::Assign {
            target: Box::new(vce),
            value: Box::new(vpe),
            position,
        });
        Ok(())
    }

    fn convert_string_literal(&mut self, node: i::Node) -> String {
        debug_assert!(node.as_rule() == i::Rule::string);
        let text = node.as_str();
        snailquote::unescape(text).expect("bad AST")
    }

    fn convert_include_statement(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::include_statement);
        let position = self.make_position(&node);
        let mut children = node.into_inner();
        let filename_node = children.next().expect("bad AST");
        let filename = self.convert_string_literal(filename_node);
        if let Some(file_index) = self.source_set.find_source(&filename) {
            if self.aux_scope_data.included_files.contains(&file_index) {
                // Don't include the file a second time.
                return Ok(());
            }
            self.aux_scope_data.included_files.insert(file_index);
            let (_name, content) = self.source_set.borrow_source(file_index);
            let timer = std::time::Instant::now();
            let mut ast = match crate::ast::ingest(content, file_index) {
                Ok(ast) => ast,
                Err(mut problem) => {
                    problems::hint_encountered_while_including(&mut problem, position);
                    return Err(problem);
                }
            };

            self.perf_counters.ast.time += timer.elapsed();
            self.perf_counters.ast.num_invocations += 1;

            let old_file_id = self.current_file_id;
            self.current_file_id = file_index;
            if let Err(mut problem) = self.execute(&mut ast) {
                problems::hint_encountered_while_including(&mut problem, position);
                return Err(problem);
            }
            self.current_file_id = old_file_id;
        } else {
            return Err(problems::nonexistant_include(position, &filename));
        }
        Ok(())
    }

    pub(super) fn convert_statement(&mut self, node: i::Node) -> Result<(), CompileProblem> {
        debug_assert!(node.as_rule() == i::Rule::statement);
        let child = node.into_inner().next().expect("bad AST");
        match child.as_rule() {
            i::Rule::macro_definition => self.convert_macro_definition(child)?,
            i::Rule::code_block => self.convert_code_block(child)?,
            i::Rule::return_statement => self.convert_return_statement(child)?,
            i::Rule::assert_statement => self.convert_assert_statement(child)?,
            i::Rule::if_statement => self.convert_if_statement(child)?,
            i::Rule::for_loop_statement => self.convert_for_loop_statement(child)?,
            i::Rule::input_variable_statement => self.convert_input_variable_statement(child)?,
            i::Rule::output_variable_statement => self.convert_output_variable_statement(child)?,
            i::Rule::static_variable_statement => self.convert_static_variable_statement(child)?,
            i::Rule::assign_statement => self.convert_assign_statement(child)?,
            i::Rule::macro_call => {
                let expr = self.convert_macro_call(child, false)?;
                self.add_statement(o::Statement::RawVPExpression(Box::new(expr)));
            }
            i::Rule::var_dec => {
                self.convert_var_dec(child)?;
            }
            i::Rule::include_statement => self.convert_include_statement(child)?,
            _ => unreachable!("bad AST"),
        }
        Ok(())
    }
}
