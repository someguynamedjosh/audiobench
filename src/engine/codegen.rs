use crate::engine::parts::*;
use crate::gui::module_widgets::FeedbackDataRequirement;
use crate::util::*;

pub struct CodegenResult {
    pub code: String,
    pub aux_data_collector: AuxDataCollector,
    pub feedback_displayer: FeedbackDisplayer,
}

// This packages changes made by the user to knobs and automation into a format that can be read
// by the nodespeak parameter, so that trivial changes don't necessitate a recompile.
pub struct AuxDataCollector {
    ordered_controls: Vec<Rcrc<Control>>,
    data_length: usize,
}

impl AuxDataCollector {
    pub fn collect_data(&self) -> Vec<f32> {
        let mut data = Vec::with_capacity(self.data_length);
        for control in &self.ordered_controls {
            let control_ref = control.borrow();
            if control_ref.automation.len() == 0 {
                data.push(control_ref.value);
            } else {
                for lane in &control_ref.automation {
                    // algebraic simplification of remapping value [-1, 1] -> [0, 1] -> [min, max]
                    let a = (lane.range.1 - lane.range.0) / 2.0;
                    let b = a + lane.range.0;
                    data.push(a);
                    data.push(b);
                }
            }
        }
        debug_assert!(data.len() == self.data_length);
        data
    }

    pub fn get_data_length(&self) -> usize {
        self.data_length
    }
}

pub struct FeedbackDisplayer {
    ordered_modules: Vec<Rcrc<Module>>,
    data_length: usize,
}

impl FeedbackDisplayer {
    pub fn display_feedback(&mut self, feedback_data: &[f32]) {
        assert!(feedback_data.len() == self.data_length);
        let mut data_pos = 0;
        for module in &self.ordered_modules {
            let module_ref = module.borrow_mut();
            let module_data_length = module_ref.template.borrow().feedback_data_len;
            if let Some(data_ptr) = &module_ref.feedback_data {
                let slice = &feedback_data[data_pos..data_pos + module_data_length];
                data_ptr.borrow_mut().clone_from_slice(slice);
            }
            data_pos += module_data_length;
        }
        debug_assert!(data_pos == self.data_length);
    }

    pub fn get_data_length(&self) -> usize {
        self.data_length
    }
}

pub fn generate_code(
    for_graph: &ModuleGraph,
    buffer_length: i32,
    sample_rate: i32,
) -> Result<CodegenResult, ()> {
    let execution_order = for_graph.compute_execution_order()?;
    let generator = CodeGenerator {
        graph: for_graph,
        execution_order,
        current_aux_data_item: 0,
        aux_data_control_order: Vec::new(),
        feedback_data_len: 0,
    };
    Ok(generator.generate_code(buffer_length, sample_rate))
}

struct CodeGenerator<'a> {
    graph: &'a ModuleGraph,
    execution_order: Vec<usize>,
    current_aux_data_item: usize,
    aux_data_control_order: Vec<Rcrc<Control>>,
    feedback_data_len: usize,
}

fn format_decimal(value: f32) -> String {
    let digits = 8i32;
    let digits = match value {
        v if v <= 0.0 => digits,
        _ => digits - (value.abs().log10().min(digits as f32 - 1.0) as i32),
    };
    let digits = digits as usize;
    format!("{:.*}", digits, value)
}

fn snake_case_to_pascal_case(snake_case: &str) -> String {
    let mut result = "".to_owned();
    let mut capitalize = true;
    for c in snake_case.chars() {
        if c == '_' {
            capitalize = true;
        } else if capitalize {
            capitalize = false;
            result.push(c.to_ascii_uppercase());
        } else {
            result.push(c);
        }
    }
    result
}

impl<'a> CodeGenerator<'a> {
    fn next_aux_value(&mut self) -> String {
        self.current_aux_data_item += 1;
        format!("global_aux_data[{}]", self.current_aux_data_item - 1)
    }

    fn generate_code_for_lane(&mut self, lane: &AutomationLane) -> String {
        let mod_index = self
            .graph
            .index_of_module(&lane.connection.0)
            .unwrap_or(3999999);
        // The two values in the aux data are computed based on the min and max of the automation
        // channel such that mulitplying by the first and adding the second will generate the
        // appropriate transformation. See AuxDataCollector::collect_data for more.
        format!(
            "module_{}_output_{} * {} + {}",
            mod_index,
            lane.connection.1,
            self.next_aux_value(),
            self.next_aux_value(),
        )
    }

    fn generate_code_for_control(&mut self, control: &Rcrc<Control>) -> String {
        self.aux_data_control_order.push(Rc::clone(control));
        let control_ref = control.borrow();
        if control_ref.automation.len() == 0 {
            self.next_aux_value()
        } else {
            let mut code = self.generate_code_for_lane(&control_ref.automation[0]);
            for lane in &control_ref.automation[1..] {
                code.push_str(" + ");
                code.push_str(&self.generate_code_for_lane(lane));
            }
            code
        }
    }

    fn generate_code_for_input(&mut self, connection: &InputConnection, jack: &IOJack) -> String {
        match connection {
            InputConnection::Wire(module, output_index) => format!(
                "module_{}_output_{}",
                self.graph.index_of_module(&module).unwrap_or(2999999),
                output_index
            ),
            InputConnection::Default(index) => {
                jack.borrow_default_options()[*index].code.to_owned()
            }
        }
    }

    fn generate_function_def_for_module(
        &mut self,
        code: &mut String,
        index: usize,
        module_ref: &Module,
    ) {
        code.push_str(&format!("macro module_{}(\n", index));
        let template_ref = module_ref.template.borrow();
        for input in template_ref.inputs.iter() {
            code.push_str(&format!("    {}, \n", input.borrow_code_name()));
        }
        for control in module_ref.controls.iter() {
            let control_ref = control.borrow();
            code.push_str(&format!("    {}, \n", control_ref.code_name));
        }
        code.push_str("):(\n");
        for output in template_ref.outputs.iter() {
            code.push_str(&format!("    {}, \n", output.borrow_code_name()));
        }
        code.push_str(") {\n");

        let template_ref = module_ref.template.borrow();
        let mut control_feedback_code = "".to_owned();
        let mut custom_feedback_code = "".to_owned();
        for wo in &template_ref.widget_outlines {
            match wo.get_feedback_data_requirement() {
                FeedbackDataRequirement::None => (),
                FeedbackDataRequirement::Control { control_index } => {
                    control_feedback_code.push_str(&format!(
                        "        global_feedback_data[{}] = {}[0?][0?];\n",
                        self.feedback_data_len,
                        &module_ref.controls[control_index].borrow().code_name
                    ));
                    self.feedback_data_len += 1;
                }
                FeedbackDataRequirement::Custom { code_name, size } => {
                    // TODO: Check for duplicate code names (preferably in registry code.)
                    let code_name = snake_case_to_pascal_case(&code_name);
                    custom_feedback_code
                        .push_str(&format!("    macro Set{}(data):() {{\n", code_name));
                    // TODO: No loop for single items?
                    custom_feedback_code.push_str(&format!("        for i = 0 to {} {{\n", size));
                    custom_feedback_code.push_str(&format!(
                        "            global_feedback_data[{} + i] = data[i?];\n",
                        self.feedback_data_len
                    ));
                    custom_feedback_code.push_str("        }\n    }\n");
                    self.feedback_data_len += size;
                }
            }
        }
        if control_feedback_code.len() > 0 {
            code.push_str("    if global_update_feedback_data {\n");
            code.push_str(&control_feedback_code);
            code.push_str("    }\n");
        }
        code.push_str(&custom_feedback_code);

        code.push_str(&format!(
            "    include \"{}\";\n",
            template_ref.code_resource
        ));
        code.push_str("}\n\n");
    }

    fn generate_code(mut self, buffer_length: i32, sample_rate: i32) -> CodegenResult {
        let mut header = "".to_owned();
        header.push_str(&format!("INT BUFFER_LENGTH = {};\n", buffer_length,));
        header.push_str(&format!("FLOAT SAMPLE_RATE = {}.0;\n", sample_rate,));
        header.push_str(concat!(
            "input FLOAT global_pitch, global_velocity, global_note_status, global_should_update;\n",
            "input [BUFFER_LENGTH][1]FLOAT global_note_time;\n",
            "output [BUFFER_LENGTH][2]FLOAT global_audio_out;\n",
        ));

        let mut code = "".to_owned();
        code.push_str(concat!(
            "[BUFFER_LENGTH]BOOL global_release_trigger = FALSE;\n",
            "if global_note_status == 1.0 { global_release_trigger[0] = TRUE; }\n",
            "[BUFFER_LENGTH]BOOL global_start_trigger = FALSE;\n",
            "if global_note_status == 2.0 { global_start_trigger[0] = TRUE; }\n",
            "BOOL global_update_feedback_data = global_should_update == 1.0;\n",
            "macro FlatWaveform(buffer_pos, phase):(value) { FLOAT value = 0.0; }\n",
            "\n",
        ));
        let mut ordered_modules = Vec::new();
        for (index, module) in self.graph.borrow_modules().iter().enumerate() {
            ordered_modules.push(Rc::clone(module));
            let module_ref = module.borrow();
            self.generate_function_def_for_module(&mut code, index, &*module_ref);
        }

        for index in std::mem::replace(&mut self.execution_order, Vec::new()) {
            let module_ref = &self.graph.borrow_modules()[index].borrow();
            let template_ref = module_ref.template.borrow();
            code.push_str(&format!("module_{}(\n", index));
            for (input, jack) in module_ref.inputs.iter().zip(template_ref.inputs.iter()) {
                code.push_str(&format!(
                    "    {}, // {}\n",
                    self.generate_code_for_input(input, jack),
                    jack.borrow_code_name()
                ));
            }
            for control in &module_ref.controls {
                code.push_str(&format!(
                    "    {}, // {}\n",
                    self.generate_code_for_control(control),
                    &control.borrow().code_name
                ));
            }
            code.push_str("):(\n");
            for output_index in 0..template_ref.outputs.len() {
                code.push_str(&format!(
                    "    AUTO module_{}_output_{},\n",
                    index, output_index,
                ));
            }
            code.push_str(");\n\n");
        }

        if self.current_aux_data_item > 0 {
            header.push_str(&format!(
                "input [{}]FLOAT global_aux_data;\n",
                self.current_aux_data_item
            ));
        }
        if self.feedback_data_len > 0 {
            header.push_str(&format!(
                "output [{}]FLOAT global_feedback_data;\n",
                self.feedback_data_len
            ));
        }
        let Self {
            aux_data_control_order,
            current_aux_data_item,
            feedback_data_len,
            ..
        } = self;
        let aux_data_collector = AuxDataCollector {
            ordered_controls: aux_data_control_order,
            data_length: current_aux_data_item,
        };
        let feedback_displayer = FeedbackDisplayer {
            ordered_modules,
            data_length: feedback_data_len,
        };

        CodegenResult {
            code: format!("{}\n{}", header, code),
            aux_data_collector,
            feedback_displayer,
        }
    }
}
