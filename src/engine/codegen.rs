use crate::engine::parts::*;
use crate::util::*;

pub fn generate_code(
    for_graph: &ModuleGraph,
    buffer_length: i32,
    sample_rate: i32,
) -> Result<String, ()> {
    let execution_order = for_graph.compute_execution_order()?;
    let generator = CodeGenerator {
        graph: for_graph,
        execution_order,
    };
    Ok(generator.generate_code(buffer_length, sample_rate))
}

struct CodeGenerator<'a> {
    graph: &'a ModuleGraph,
    execution_order: Vec<usize>,
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

impl<'a> CodeGenerator<'a> {
    fn generate_code_for_lane(&self, lane: &AutomationLane) -> String {
        let (target_min, target_max) = lane.range;
        // algebraic simplification of remapping value [-1, 1] -> [0, 1] -> [min, max]
        let a = (target_max - target_min) / 2.0;
        let b = a + target_min;
        let mod_index = self
            .graph
            .index_of_module(&lane.connection.0)
            .unwrap_or(3999999);
        format!(
            "module_{}_output_{} * {} + {}",
            mod_index,
            lane.connection.1,
            format_decimal(a),
            format_decimal(b)
        )
    }

    fn generate_code_for_control(&self, control: &Rcrc<Control>) -> String {
        let control_ref = control.borrow();
        if control_ref.automation.len() == 0 {
            format_decimal(control_ref.value)
        } else {
            let mut code = self.generate_code_for_lane(&control_ref.automation[0]);
            for lane in &control_ref.automation[1..] {
                code.push_str(" + ");
                code.push_str(&self.generate_code_for_lane(lane));
            }
            code
        }
    }

    fn generate_code_for_input(&self, input: &InputConnection, typ: JackType) -> String {
        match input {
            InputConnection::Wire(module, output_index) => format!(
                "module_{}_output_{}",
                self.graph.index_of_module(&module).unwrap_or(2999999),
                output_index
            ),
            InputConnection::Default => typ.default_value().to_owned(),
        }
    }

    fn generate_code(&self, buffer_length: i32, sample_rate: i32) -> String {
        let mut code = "".to_owned();
        code.push_str(&format!("INT BUFFER_LENGTH = {};\n", buffer_length,));
        code.push_str(&format!("FLOAT SAMPLE_RATE = {}.0;\n", sample_rate,));
        code.push_str(concat!(
            "input FLOAT global_pitch, global_velocity, global_note_status;\n",
            "input [BUFFER_LENGTH][1]FLOAT global_note_time;\n",
            "output [BUFFER_LENGTH][2]FLOAT global_audio_out;\n",
            "[BUFFER_LENGTH]BOOL global_release_trigger = FALSE;\n",
            "if global_note_status == 1.0 { global_release_trigger[0] = TRUE; }\n",
            "macro FlatWaveform(buffer_pos, phase):(value) { FLOAT value = 0.0; }\n",
            "\n",
        ));

        for (index, module) in self.graph.borrow_modules().iter().enumerate() {
            code.push_str(&format!("macro module_{}(\n", index));
            let module_ref = module.borrow();
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
            code.push_str(&format!(
                "    include \"{}\";\n",
                template_ref.code_resource
            ));
            code.push_str("}\n\n");
        }

        for index in &self.execution_order {
            let module_ref = self.graph.borrow_modules()[*index].borrow();
            let template_ref = module_ref.template.borrow();
            code.push_str(&format!("module_{}(\n", index));
            for (input, jack) in module_ref.inputs.iter().zip(template_ref.inputs.iter()) {
                code.push_str(&format!(
                    "    {}, // {}\n",
                    self.generate_code_for_input(input, jack.get_type()),
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

        code
    }
}
