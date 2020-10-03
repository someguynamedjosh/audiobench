use super::data_format::IOType;
use super::data_routing::{AutoconDynDataCollector, FeedbackDisplayer, StaticonDynDataCollector};
use super::data_transfer::{DataFormat, HostFormat};
use super::static_controls::Staticon;
use crate::engine::parts::*;
use crate::gui::module_widgets::FeedbackDataRequirement;
use shared_util::prelude::*;

pub(super) struct CodeGenResult {
    pub(super) code: String,
    pub(super) autocon_dyn_data_collector: AutoconDynDataCollector,
    pub(super) staticon_dyn_data_collector: StaticonDynDataCollector,
    pub(super) feedback_displayer: FeedbackDisplayer,
    pub(super) data_format: DataFormat,
}

pub(super) fn generate_code(
    for_graph: &ModuleGraph,
    host_format: &HostFormat,
) -> Result<CodeGenResult, ()> {
    let execution_order = for_graph.compute_execution_order()?;
    let generator = CodeGenerator {
        graph: for_graph,
        execution_order,
        current_autocon_dyn_data_item: 0,
        autocon_dyn_data_control_order: Vec::new(),
        staticon_input_code: Vec::new(),
        staticon_dyn_data_control_order: Vec::new(),
        staticon_dyn_data_types: Vec::new(),
        feedback_data_len: 0,
    };
    Ok(generator.generate_code(host_format))
}

struct CodeGenerator<'a> {
    graph: &'a ModuleGraph,
    execution_order: Vec<usize>,
    current_autocon_dyn_data_item: usize,
    autocon_dyn_data_control_order: Vec<Rcrc<Autocon>>,
    staticon_input_code: Vec<String>,
    staticon_dyn_data_control_order: Vec<Rcrc<Staticon>>,
    staticon_dyn_data_types: Vec<IOType>,
    feedback_data_len: usize,
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
        self.current_autocon_dyn_data_item += 1;
        format!(
            "global_autocon_dyn_data[{}]",
            self.current_autocon_dyn_data_item - 1
        )
    }

    fn generate_code_for_lane(&mut self, lane: &AutomationLane) -> String {
        let mod_index = self
            .graph
            .index_of_module(&lane.connection.0)
            .unwrap_or(3999999);
        // The two values in the aux data are computed based on the min and max of the automation
        // channel such that mulitplying by the first and adding the second will generate the
        // appropriate transformation. See AutoconDynDataCollector::collect_data for more.
        format!(
            "module_{}_output_{} * {} + {}",
            mod_index,
            lane.connection.1,
            self.next_aux_value(),
            self.next_aux_value(),
        )
    }

    fn generate_code_for_control(&mut self, control: &Rcrc<Autocon>) -> String {
        self.autocon_dyn_data_control_order.push(Rc::clone(control));
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

    fn generate_code_for_staticon(&mut self, control: &Rcrc<Staticon>) -> String {
        let control_ref = control.borrow();
        if control_ref.is_static_only() {
            format!("    {}\n", control_ref.generate_static_code())
        } else {
            let unique_input_name = format!(
                "staticon_dyn_data_{}",
                self.staticon_dyn_data_control_order.len(),
            );
            let (input_code, body_code) = control_ref.generate_dynamic_code(&unique_input_name);
            self.staticon_dyn_data_control_order
                .push(Rc::clone(control));
            self.staticon_dyn_data_types.push(control_ref.get_io_type());
            self.staticon_input_code.push(input_code);
            format!("    {}\n", body_code)
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
        for control in module_ref.autocons.iter() {
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
                FeedbackDataRequirement::Autocon { control_index } => {
                    control_feedback_code.push_str(&format!(
                        "        global_feedback_data[{}] = {}[0?][0?];\n",
                        self.feedback_data_len,
                        &module_ref.autocons[control_index].borrow().code_name
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
        for control in &module_ref.staticons {
            code.push_str(&self.generate_code_for_staticon(control));
        }

        code.push_str(&format!(
            "    include \"{}\";\n",
            template_ref.code_resource
        ));
        code.push_str("}\n\n");
    }

    fn generate_code(mut self, host_format: &HostFormat) -> CodeGenResult {
        let mut header = "".to_owned();
        let buffer_length = host_format.buffer_len;
        let sample_rate = host_format.sample_rate;
        header.push_str(&format!("INT BUFFER_LENGTH = {};\n", buffer_length,));
        header.push_str(&format!("FLOAT SAMPLE_RATE = {}.0;\n", sample_rate,));
        header.push_str(concat!(
            "input FLOAT global_pitch, global_velocity, global_note_status, global_should_update, global_bpm;\n",
            "input [BUFFER_LENGTH]FLOAT global_note_time, global_note_beats, global_song_time, global_song_beats;\n",
            "input [128]FLOAT global_midi_controls;\n",
            "output [BUFFER_LENGTH][2]FLOAT global_audio_out;\n",
        ));

        let mut code = "".to_owned();
        code.push_str("include \"!:lib.ns\";\n\n");
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
            for control in &module_ref.autocons {
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

        if self.current_autocon_dyn_data_item > 0 {
            header.push_str(&format!(
                "input [{}]FLOAT global_autocon_dyn_data;\n",
                self.current_autocon_dyn_data_item
            ));
        }
        if self.feedback_data_len > 0 {
            header.push_str(&format!(
                "output [{}]FLOAT global_feedback_data;\n",
                self.feedback_data_len
            ));
        }
        for line in &self.staticon_input_code {
            header.push_str(line);
            header.push_str("\n");
        }
        let Self {
            autocon_dyn_data_control_order,
            current_autocon_dyn_data_item,
            staticon_dyn_data_control_order,
            staticon_dyn_data_types,
            feedback_data_len,
            ..
        } = self;
        let data_format = DataFormat {
            host_format: HostFormat {
                buffer_len: buffer_length,
                sample_rate,
            },
            autocon_dyn_data_len: current_autocon_dyn_data_item,
            staticon_dyn_data_types,
            feedback_data_len,
        };
        let autocon_dyn_data_collector = AutoconDynDataCollector::new(
            autocon_dyn_data_control_order,
            data_format.autocon_dyn_data_len,
        );
        let staticon_dyn_data_collector =
            StaticonDynDataCollector::new(staticon_dyn_data_control_order);
        let feedback_displayer =
            FeedbackDisplayer::new(ordered_modules, data_format.feedback_data_len);

        CodeGenResult {
            code: format!("{}\n{}", header, code),
            autocon_dyn_data_collector,
            staticon_dyn_data_collector,
            feedback_displayer,
            data_format,
        }
    }
}
