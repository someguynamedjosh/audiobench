use jlrs::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut julia = unsafe { Julia::init(16).unwrap() };
        julia
            .dynamic_frame(|global, frame| {
                // Create the two arguments, each value requires one slot
                let i = Value::new(frame, 2u64)?;
                let j = Value::new(frame, 1u32)?;

                // We can find the addition-function in the base module
                let func = Module::base(global).function("+")?;

                // Call the function and unbox the result.
                let output = func.call2(frame, i, j)?.unwrap();
                assert_eq!(output.cast::<u64>()?, 3);
                Ok(())
            })
            .unwrap();
    }
}
