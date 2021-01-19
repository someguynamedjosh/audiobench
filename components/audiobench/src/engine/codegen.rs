use super::controls::{AnyControl, AutomationSource, Control};
use super::data_transfer::{DataFormat, GlobalParameters};
use super::data_transfer::{DynDataCollector, FeedbackDisplayer};
use crate::engine::parts::*;
use crate::gui::module_widgets::FeedbackMode;
use julia_helper::GeneratedCode;
use shared_util::prelude::*;

pub(super) struct CodeGenResult {
    pub code: GeneratedCode,
    pub dyn_data_collector: DynDataCollector,
    pub feedback_displayer: FeedbackDisplayer,
    pub data_format: DataFormat,
}

pub struct AutomationCode {
    ordered_modules: Vec<Rcrc<Module>>,
}

impl AutomationCode {
    pub fn value_of(&self, source: &AutomationSource) -> String {
        let module_index = self
            .ordered_modules
            .iter()
            .position(|mod_ptr| Rc::ptr_eq(mod_ptr, &source.module))
            .unwrap(); // Our list should contain all the modules that exist.
        format!("m{}o{}", module_index, source.output_index)
    }
}

struct CodeGenerator<'a> {
    graph: &'a ModuleGraph,
    execution_order: Vec<usize>,
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
    fn generate_code(mut self, global_params: &GlobalParameters) -> CodeGenResult {
        let buffer_length = global_params.buffer_length;
        let sample_rate = global_params.sample_rate;

        let mut code = "".to_owned();
        let mut ordered_modules = Vec::new();
        let mut ordered_controls = Vec::new();
        let mut feedback_widget_selectors = Vec::new();
        for module_ptr in self.graph.borrow_modules() {
            ordered_modules.push(Rc::clone(module_ptr));
        }

        code.push_str("module Generated\n\n  using Main.Registry.Factory.Lib\n\n");
        code.push_str("  mutable struct StaticData");
        for (index, module) in self.graph.borrow_modules().iter().enumerate() {
            let module_ref = module.borrow();
            let template_ref = module_ref.template.borrow();
            code.push_str(&format!(
                "\n    m{}::Main.Registry.{}.{}Module.StaticData",
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
            "      if index > length(static_container)\n",
            "        static_init(index)\n",
            "      end\n",
            "    else\n",
            "      static_container[index + 1] = data\n",
            "    end\n",
        ));
        code.push_str("  end # function static_init\n\n");

        code.push_str("  mutable struct FeedbackData\n");
        // code.push_str("    ");
        for (module_index, module_ptr) in self.graph.borrow_modules().iter().enumerate() {
            let module = module_ptr.borrow();
            let template = module.template.borrow();
            for (widget_index, outline) in template.widget_outlines.iter().enumerate() {
                if outline.get_feedback_mode() != FeedbackMode::None {
                    feedback_widget_selectors.push((Rc::clone(module_ptr), widget_index));
                    code.push_str(&format!(
                        "    m{}w{}::Vector{{Float32}}\n",
                        module_index, widget_index
                    ));
                }
            }
        }
        code.push_str("  end # struct FeedbackData\n\n");

        let mut exec_body = String::new();
        code.push_str(concat!(
            "  function exec(midi_controls::Vector{Float32}, pitch_wheel::Float32,\n",
            "    bpm::Float32, elapsed_time::Float32, elapsed_beats::Float32,\n",
            "    do_feedback::Bool, note_input::NoteInput, static_index::Integer,",
        ));
        exec_body.push_str(concat!(
            "    set_zero_subnormals(true)\n",
            "    static_index += 1\n", // grumble grumble
            "    global_input = GlobalInput(midi_controls, pitch_wheel, bpm, elapsed_time, ",
            "elapsed_beats)\n",
            "    start_trigger = Trigger(note_input.start_trigger, repeat([false], buffer_length - 1)...)\n",
            "    release_trigger = Trigger(note_input.release_trigger, repeat([false], buffer_length - 1)...)\n",
            "    note_output = NoteOutput()\n",
            "    context = NoteContext(global_input, note_input, note_output)\n",
            "    feedback = FeedbackData(",
        ));
        for _ in 0..feedback_widget_selectors.len() {
            exec_body.push_str("Vector{Float32}(), ");
        }
        exec_body.push_str(")\n\n    @. context.note_out.audio = 0.0\n");
        let automation_code = AutomationCode {
            ordered_modules: ordered_modules.clone(),
        };
        for index in std::mem::replace(&mut self.execution_order, Vec::new()) {
            let module_ref = self.graph.borrow_modules()[index].borrow();
            let template_ref = module_ref.template.borrow();
            exec_body.push_str("    \n");

            for (control_index, control) in module_ref.controls.iter().enumerate() {
                let control_ptr = control.as_dyn_ptr();
                let control = control_ptr.borrow();
                let mut idents = Vec::new();
                if control.get_parameter_types().len() > 0 {
                    code.push_str("\n    ");
                }
                for (parameter_index, ptype) in
                    control.get_parameter_types().into_iter().enumerate()
                {
                    let ident = format!("m{}c{}p{}", index, control_index, parameter_index);
                    code.push_str(&format!(" {}::{},", ident, ptype));
                    idents.push(ident);
                }
                let ident_refs: Vec<_> = idents.iter().map(|i| &i[..]).collect();
                let code = control.generate_code(&ident_refs[..], &automation_code);
                drop(control);
                exec_body.push_str(&format!("    m{}c{} = {}\n", index, control_index, code));
                ordered_controls.push(control_ptr);
            }
            let template = module_ref.template.borrow();
            let mut first = true;
            for (widget_index, widget) in template.widget_outlines.iter().enumerate() {
                if let FeedbackMode::ControlSignal { control_index } = widget.get_feedback_mode() {
                    if first {
                        first = false;
                        exec_body.push_str("    if do_feedback\n");
                    }
                    exec_body.push_str(&format!(
                        "      push!(feedback.m{}w{}, m{}c{}[%, 1])\n",
                        index, widget_index, index, control_index
                    ));
                }
            }
            if !first {
                exec_body.push_str("    end\n");
            }

            exec_body.push_str("    ");
            for output_index in 0..template_ref.outputs.len() {
                exec_body.push_str(&format!("m{}o{}, ", index, output_index,));
            }
            exec_body.push_str(&format!("static_container[static_index].m{}, = \n", index));
            exec_body.push_str(&format!(
                "    Main.Registry.{}.{}Module.exec(\n      context, do_feedback,\n",
                template_ref.lib_name, template_ref.module_name
            ));

            exec_body.push_str("      ");
            for (control_index, _) in module_ref.controls.iter().enumerate() {
                exec_body.push_str(&format!("m{}c{}, ", index, control_index));
            }
            let mut first = true;
            for (widget_index, widget) in template.widget_outlines.iter().enumerate() {
                if let FeedbackMode::ManualValue { name } = widget.get_feedback_mode() {
                    if first {
                        first = false;
                        exec_body.push_str("\n      ");
                    }
                    exec_body.push_str(&format!("feedback.m{}w{}, ", index, widget_index));
                }
            }
            exec_body.push_str(&format!(
                "\n      static_container[static_index].m{},\n    )\n",
                index
            ));
        }
        code.push_str("\n  )\n");
        code.push_str(&exec_body);
        code.push_str("\n\n    (Array(context.note_out.audio), feedback)\n");
        code.push_str("  end # function exec\n\n");
        code.push_str("end # module Generated\n");
        let code = GeneratedCode::from_unique_source("Generated/note_graph.jl", &code);

        let Self {
            dyn_data_types,
            feedback_data_len,
            ..
        } = self;
        let data_format = DataFormat {
            global_params: global_params.clone(),
            dyn_data_types,
            feedback_data_len,
        };
        let dyn_data_collector = DynDataCollector::new(ordered_controls);
        let feedback_displayer = FeedbackDisplayer::new(feedback_widget_selectors);

        println!("{}", code.as_str());

        CodeGenResult {
            code,
            dyn_data_collector,
            feedback_displayer,
            data_format,
        }
    }
}
