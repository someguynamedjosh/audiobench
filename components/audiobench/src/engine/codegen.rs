use super::data_routing::{AutoconDynDataCollector, FeedbackDisplayer, StaticonDynDataCollector};
use super::data_transfer::{DataFormat, GlobalParameters};
use super::static_controls::Staticon;
use crate::engine::parts::*;
use crate::gui::module_widgets::FeedbackDataRequirement;
use julia_helper::GeneratedCode;
use shared_util::prelude::*;

pub(super) struct CodeGenResult {
    pub code: GeneratedCode,
    pub autocon_dyn_data_collector: AutoconDynDataCollector,
    pub staticon_dyn_data_collector: StaticonDynDataCollector,
    pub feedback_displayer: FeedbackDisplayer,
    pub data_format: DataFormat,
}

pub(super) fn generate_code(
    for_graph: &ModuleGraph,
    global_params: &GlobalParameters,
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
    Ok(generator.generate_code(global_params))
}

struct CodeGenerator<'a> {
    graph: &'a ModuleGraph,
    execution_order: Vec<usize>,
    current_autocon_dyn_data_item: usize,
    autocon_dyn_data_control_order: Vec<Rcrc<Autocon>>,
    staticon_input_code: Vec<String>,
    staticon_dyn_data_control_order: Vec<Rcrc<Staticon>>,
    staticon_dyn_data_types: Vec<()>, // Previously IOType
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
            self.staticon_dyn_data_types.push(()); //control_ref.get_io_type());
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
                "{}::Main.Registry.{}.{}.StaticData",
                index, template_ref.lib_name, template_ref.module_name
            ));
        }
        code.push_str("\n  end\n\n");

        code.push_str("  const static_container = Vector{StaticData}()\n\n");
        code.push_str("  function init_static(index::Integer)\n");
        code.push_str("    data = StaticData(\n");
        for (index, module) in self.graph.borrow_modules().iter().enumerate() {
            let module_ref = module.borrow();
            let template_ref = module_ref.template.borrow();
            code.push_str(&format!(
                "      Main.Registry.{}.{}.init_static()",
                template_ref.lib_name, template_ref.module_name
            ));
            if index < self.graph.borrow_modules().len() - 1 {
                code.push_str(",\n");
            }
        }
        code.push_str("\n    )\n");
        code.push_str(concat!(
            "    if index > length(static_container)\n",
            "      push!(static_container, data)\n",
            "    else\n",
            "      static_container[index] = data\n",
            "    end\n",
        ));
        code.push_str("  end # function init_static\n\n");

        code.push_str(concat!(
            "  function exec(midi_controls::Vector{Float32}, pitch_wheel::Float32, bpm::Float32, ",
            "elapsed_time::Float32, elapsed_beats::Float32, do_update::Bool, ",
            "note_input::NoteInput, static_index::Integer)\n",
            "    global_input = GlobalInput(midi_controls, pitch_wheel, bpm, elapsed_time, ",
            "elapsed_beats, do_update)\n",
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
                "static_container[static_index].for_module_{} = \n",
                index
            ));
            code.push_str(&format!(
                "    Main.Registry.{}.{}Module.exec(\n      context,\n",
                template_ref.lib_name, template_ref.module_name
            ));
            for (input, jack) in module_ref.inputs.iter().zip(template_ref.inputs.iter()) {
                code.push_str(&format!(
                    "      {}, # {}\n",
                    self.generate_code_for_input(input, jack),
                    jack.borrow_code_name()
                ));
            }
            for control in &module_ref.autocons {
                code.push_str(&format!(
                    "      {}, # {}\n",
                    self.generate_code_for_control(control),
                    &control.borrow().code_name
                ));
            }
            code.push_str(&format!(
                "      static_container[static_index].for_module_{}\n    )\n",
                index
            ));
        }
        code.push_str("  end # function exec\n\n");
        code.push_str("end # module Generated\n");
        let code = GeneratedCode::from_unique_source("Generated/note_graph.jl", &code);

        let Self {
            autocon_dyn_data_control_order,
            current_autocon_dyn_data_item,
            staticon_dyn_data_control_order,
            staticon_dyn_data_types,
            feedback_data_len,
            ..
        } = self;
        let data_format = DataFormat {
            global_params: global_params.clone(),
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
            code,
            autocon_dyn_data_collector,
            staticon_dyn_data_collector,
            feedback_displayer,
            data_format,
        }
    }
}
