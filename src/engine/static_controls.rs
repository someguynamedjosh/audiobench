use crate::engine::codegen::{IOType, IOData};

pub trait StaticControl {
    /// Returns the variable name this static control has in the generated code.
    fn borrow_code_name(&self) -> &str;
    /// Returns the data type this control has in the generated code.
    fn get_data_type(&self) -> String;
    /// Returns code that provides the current value of this control without allowing it to change
    /// in real time.
    fn generate_static_code(&self) -> String;
    /// Returns true if the control's value must be available at compile time. This will cause the
    /// code to be recompiled every time the user changes the value, so it should be avoided if at
    /// all possible.
    fn is_static_only(&self) -> bool;
    /// Returns code that dynamically retrieves the value of this control. This will not be called
    /// if `is_static_only()` returns `true`. The second element of the tuple is the input that
    /// should be added to the program to allow dynamically passing the data into the program.
    fn generate_dynamic_code(&self) -> (String, (String, IOType));
}


