use super::controls::{AnyControl, Control, AutomationSource};
use super::data_routing::{ControlDynDataCollector, FeedbackDisplayer};
use super::data_transfer::{DataFormat, GlobalParameters};
use crate::engine::parts::*;
use crate::gui::module_widgets::FeedbackDataRequirement;
use julia_helper::GeneratedCode;
use shared_util::prelude::*;

pub(super) struct CodeGenResult {
    pub code: GeneratedCode,
    pub dyn_data_collector: ControlDynDataCollector,
    pub feedback_displayer: FeedbackDisplayer,
    pub data_format: DataFormat,
}

pub struct AutomationCode {

}

impl AutomationCode {
    fn of(source: &AutomationSource) -> String {
        unimplemented!();
    }
}

struct CodeGenerator<'a> {
    graph: &'a ModuleGraph,
    execution_order: Vec<usize>,
    dyn_data_control_order: Vec<AnyControl>,
    dyn_data_types: Vec<()>, // Previously IOType
    dyn_data_parameter_defs: Vec<String>,
    feedback_data_len: usize,
}

pub(super) fn generate_code(
    for_graph: &ModuleGraph,
    global_params: &GlobalParameters,
) -> Result<CodeGenResult, ()> {
    let execution_order = for_graph.compute_execution_order()?;
    let generator = CodeGenerator {
        graph: for_graph,
        execution_order,
        dyn_data_types: Vec::new(),
        dyn_data_control_order: Vec::new(),
        dyn_data_parameter_defs: Vec::new(),
        feedback_data_len: 0,
    };
    Ok(generator.generate_code(global_params))
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
    fn generate_code_for_control(&mut self, control: &AnyControl) -> String {
        let control_ptr = control.as_dyn_ptr();
        let control_ref = control_ptr.borrow();
        unimplemented!();
        // if control_ref.is_static_only() {
        //     format!("    {}\n", control_ref.generate_static_code())
        // } else {
        //     let unique_input_name = format!(
        //         "control_dyn_data_{}",
        //         self.control_dyn_data_control_order.len(),
        //     );
        //     // let (input_code, body_code) = control_ref.generate_dynamic_code(&unique_input_name);
        //     self.dyn_data_control_order.push(Rc::clone(control));
        //     self.dyn_data_types.push(()); //control_ref.get_io_type());
        //     // self.control_input_code.push(input_code);
        //     format!("    {}\n", body_code)
        // }
    }

    fn generate_code_for_input(&mut self, connection: &(), jack: &IOJack) -> String {
        unimplemented!();
        // match connection {
        //     InputConnection::Wire(module, output_index) => format!(
        //         "module_{}_output_{}",
        //         self.graph.index_of_module(&module).unwrap_or(2999999),
        //         output_index
        //     ),
        //     InputConnection::Default(index) => {
        //         jack.borrow_default_options()[*index].code.to_owned()
        //     }
        // }
    }

    fn generate_code(mut self, global_params: &GlobalParameters) -> CodeGenResult {
        let buffer_length = global_params.buffer_length;
        let sample_rate = global_params.sample_rate;

        let mut code = "".to_owned();
        let mut ordered_modules = Vec::new();
        for module_ptr in self.graph.borrow_modules() {
            ordered_modules.push(Rc::clone(module_ptr));
        }

        code.push_str("module Generated\n\n  using Main.Registry.Factory.Lib\n\n");
        code.push_str("  mutable struct StaticData");
        for (index, module) in self.graph.borrow_modules().iter().enumerate() {
            let module_ref = module.borrow();
            let template_ref = module_ref.template.borrow();
            code.push_str("\n    for_module_");
            code.push_str(&format!(
                "{}::Main.Registry.{}.{}Module.StaticData",
                index, template_ref.lib_name, template_ref.module_name
            ));
        }
        code.push_str("  end\n\n");

        code.push_str("  const static_container = Vector{StaticData}()\n\n");
        code.push_str("  function static_init(index::Integer)\n");
        code.push_str("    data = StaticData(\n");
        for (index, module) in self.graph.borrow_modules().iter().enumerate() {
            let module_ref = module.borrow();
            let template_ref = module_ref.template.borrow();
            code.push_str(&format!(
                "      Main.Registry.{}.{}Module.static_init()",
                template_ref.lib_name, template_ref.module_name
            ));
            if index < self.graph.borrow_modules().len() - 1 {
                code.push_str(",\n");
            }
        }
        code.push_str("\n    )\n");
        code.push_str(concat!(
            "    if index >= length(static_container)\n",
            "      push!(static_container, data)\n",
            "    else\n",
            "      static_container[index + 1] = data\n",
            "    end\n",
        ));
        code.push_str("  end # function static_init\n\n");

        code.push_str(concat!(
            "  function exec(midi_controls::Vector{Float32}, pitch_wheel::Float32, bpm::Float32, ",
            "elapsed_time::Float32, elapsed_beats::Float32, do_update::Bool, ",
            "note_input::NoteInput, static_index::Integer)\n",
            "    static_index += 1\n", // grumble grumble
            "    global_input = GlobalInput(midi_controls, pitch_wheel, bpm, elapsed_time, ",
            "elapsed_beats, do_update)\n",
            "    global_start_trigger = Trigger(note_input.start_trigger, repeat([false], buffer_length - 1)...)\n",
            "    global_release_trigger = Trigger(note_input.release_trigger, repeat([false], buffer_length - 1)...)\n",
            "    global_autocon_dyn_data = SA_F32[0.25f0, 0.25f0, 0.25f0, 0.25f0, 0.25f0, 0.25f0, 0.25f0, 0.25f0, 0.25f0, 0.25f0, 0.25f0]\n",
            "    note_output = NoteOutput()\n",
            "    context = NoteContext(global_input, note_input, note_output)\n",
        ));
        for index in std::mem::replace(&mut self.execution_order, Vec::new()) {
            let module_ref = self.graph.borrow_modules()[index].borrow();
            let template_ref = module_ref.template.borrow();

            code.push_str("\n    ");
            for output_index in 0..template_ref.outputs.len() {
                code.push_str(&format!("module_{}_output_{}, ", index, output_index,));
            }
            code.push_str(&format!(
                "static_container[static_index].for_module_{}, = \n",
                index
            ));
            code.push_str(&format!(
                "    Main.Registry.{}.{}Module.exec(\n      context,\n",
                template_ref.lib_name, template_ref.module_name
            ));
            code.push_str(&format!(
                "      static_container[static_index].for_module_{}\n    )\n",
                index
            ));
        }
        code.push_str("    (Array(context.note_out.audio),)\n");
        code.push_str("  end # function exec\n\n");
        code.push_str("end # module Generated\n");
        let code = GeneratedCode::from_unique_source("Generated/note_graph.jl", &code);

        let Self {
            dyn_data_control_order,
            dyn_data_types,
            feedback_data_len,
            ..
        } = self;
        let data_format = DataFormat {
            global_params: global_params.clone(),
            dyn_data_types,
            feedback_data_len,
        };
        let dyn_data_collector =
            ControlDynDataCollector::new(dyn_data_control_order);
        let feedback_displayer =
            FeedbackDisplayer::new(ordered_modules, data_format.feedback_data_len);

        CodeGenResult {
            code,
            dyn_data_collector,
            feedback_displayer,
            data_format,
        }
    }
}
