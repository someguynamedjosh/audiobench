use crate::{
    engine::{
        codegen::AutomationCode,
        controls::{Control, UpdateRequest},
        data_transfer::{IOData, IOType},
    },
    registry::yaml::YamlNode,
};
use shared_util::mini_serde::{MiniDes, MiniSer};

#[derive(Clone, Debug)]
pub struct DurationControl {
    decimal_value: f32,
    fraction_mode: bool,
    fraction_numerator: u8,
    fraction_denominator: u8,
}

impl DurationControl {
    pub fn from_yaml(yaml: &YamlNode) -> Result<Self, String> {
        let fraction_mode = if let Ok(child) = yaml.unique_child("default_format") {
            child.parse_enumerated(&["decimal", "fractional"])? == 1
        } else {
            false
        };
        // Oof ouch owie my indentation
        let (decimal_value, fraction_numerator, fraction_denominator) =
            if let Ok(child) = yaml.unique_child("default") {
                if fraction_mode {
                    let (num, den) = child.parse_custom(|value| {
                        let slash_index = value.find("/").ok_or_else(|| {
                            format!("ERROR: Not a valid fraction, missing a slash.")
                        })?;
                        let tex_num = &value[..slash_index].trim();
                        let tex_den = &value[slash_index + 1..].trim();
                        let num = tex_num
                            .parse()
                            .map_err(|_| format!("ERROR: The numerator is not valid."))?;
                        let den = tex_den
                            .parse()
                            .map_err(|_| format!("ERROR: The numerator is not valid."))?;
                        Ok((num, den))
                    })?;
                    (1.0, num, den)
                } else {
                    (child.parse_ranged(Some(0.0), None)?, 1, 1)
                }
            } else {
                (1.0, 1, 1)
            };
        Ok(Self {
            decimal_value,
            fraction_mode,
            fraction_numerator,
            fraction_denominator,
        })
    }

    fn get_raw_value(&self) -> f32 {
        if self.fraction_mode {
            self.fraction_numerator as f32 / self.fraction_denominator as f32
        } else {
            self.decimal_value
        }
    }

    pub fn get_formatted_value(&self) -> String {
        if self.fraction_mode {
            format!("{}/{}", self.fraction_numerator, self.fraction_denominator)
        } else {
            let value = self.decimal_value;
            let decimals = if value < 0.999 {
                3
            } else if value < 9.99 {
                2
            } else if value < 99.9 {
                1
            } else {
                0
            };
            format!("{:.1$}", value, decimals)
        }
    }

    pub fn get_decimal_value(&self) -> f32 {
        self.decimal_value
    }

    pub fn set_decimal_value(&mut self, value: f32) -> UpdateRequest {
        // This assert does not protect anything but it *is* indicative of a bug.
        debug_assert!(!self.fraction_mode);
        if self.decimal_value == value {
            return UpdateRequest::Nothing;
        }
        self.decimal_value = value;
        UpdateRequest::UpdateDynData
    }

    pub fn get_fractional_value(&self) -> (u8, u8) {
        (self.fraction_numerator, self.fraction_denominator)
    }

    pub fn set_fractional_value(&mut self, (numerator, denominator): (u8, u8)) -> UpdateRequest {
        // This assert does not protect anything but it *is* indicative of a bug.
        debug_assert!(self.fraction_mode);
        if self.fraction_numerator == numerator && self.fraction_denominator == denominator {
            return UpdateRequest::Nothing;
        }
        self.fraction_numerator = numerator;
        self.fraction_denominator = denominator;
        UpdateRequest::UpdateDynData
    }

    pub fn is_using_fractional_mode(&self) -> bool {
        self.fraction_mode
    }

    pub fn use_decimal_mode(&mut self) -> UpdateRequest {
        // This assert does not protect anything but it *is* indicative of a bug.
        debug_assert!(self.fraction_mode);
        self.fraction_mode = false;
        self.decimal_value = self.fraction_numerator as f32 / self.fraction_denominator as f32;
        UpdateRequest::UpdateDynData
    }

    pub fn use_fractional_mode(&mut self) -> UpdateRequest {
        // This assert does not protect anything but it *is* indicative of a bug.
        debug_assert!(!self.fraction_mode);
        self.fraction_mode = true;
        // Numerator, denominator, distance.
        let mut closest_match = (1, 1, 1.0);
        for &den in &[2, 3, 4, 5, 6, 8, 10, 12, 15, 16, 20, 24, 32] {
            for num in 1..21 {
                let decimal_value = num as f32 / den as f32;
                let distance = (self.get_decimal_value() - decimal_value).abs();
                if distance < closest_match.2 {
                    closest_match = (num, den, distance);
                }
            }
        }
        self.fraction_numerator = closest_match.0;
        self.fraction_denominator = closest_match.1;
        UpdateRequest::UpdateDynData
    }

    pub fn toggle_mode(&mut self) -> UpdateRequest {
        if self.fraction_mode {
            self.use_decimal_mode()
        } else {
            self.use_fractional_mode()
        }
    }
}

#[rustfmt::skip] 
impl Control for DurationControl {
    fn get_parameter_types(&self) -> Vec<IOType> { vec![IOType::Float] }
    fn get_parameter_values(&self) -> Vec<IOData> { vec![IOData::Float(self.get_raw_value())] }
    fn generate_code(&self, params: &[&str], _automation_code: &AutomationCode) -> String { 
        assert!(params.len() == 1);
        format!("StaticControlSignal({})", params[0])
    }
    fn serialize(&self, ser: &mut MiniSer) { 
        ser.bool(self.fraction_mode);
        if self.fraction_mode {
            ser.u8(self.fraction_numerator);
            ser.u8(self.fraction_denominator);
        } else {
            ser.f32(self.decimal_value);
        }
    }
    fn deserialize(&mut self, des: &mut MiniDes) -> Result<(), ()> { 
        self.fraction_mode = des.bool()?;
        if self.fraction_mode {
            self.fraction_numerator = des.u8()?;
            self.fraction_denominator = des.u8()?;
        } else {
            self.decimal_value = des.f32()?;
        }
        Ok(())
    }
}
