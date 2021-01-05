/// Represents the data type of a variable which is either an input or output in the generated
/// program. E.G. `IOType::FloatArray(20)` would be the type of `input [20]FLOAT some_data;`.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum IOType {
    Bool,
    Int,
    Float,
    BoolArray(usize),
    IntArray(usize),
    FloatArray(usize),
}

impl IOType {
    /// Returns an IOType which represents the type `[dimension]self`. For example, if dimension is
    /// 5 and `self` is `BoolArray(20)`, `wrap()` will return `BoolArray(100)`.
    pub(crate) fn wrap(self, dimension: usize) -> Self {
        match self {
            Self::Bool => Self::BoolArray(dimension),
            Self::Int => Self::IntArray(dimension),
            Self::Float => Self::FloatArray(dimension),
            Self::BoolArray(size) => Self::BoolArray(dimension * size),
            Self::IntArray(size) => Self::IntArray(dimension * size),
            Self::FloatArray(size) => Self::FloatArray(dimension * size),
        }
    }

    pub(crate) fn get_packed_size(&self) -> usize {
        match self {
            Self::Bool => 1,
            Self::Int => 4,
            Self::Float => 4,
            Self::BoolArray(size) => *size,
            Self::IntArray(size) => 4 * *size,
            Self::FloatArray(size) => 4 * *size,
        }
    }
}

/// Represents data that can be passed to the program or received from the program.
#[derive(PartialEq, Debug)]
pub enum IODataPtr<'a> {
    Bool(bool),
    Int(i32),
    Float(f32),
    BoolArray(&'a [bool]),
    IntArray(&'a [i32]),
    FloatArray(&'a [f32]),
}

impl From<bool> for IODataPtr<'static> {
    fn from(data: bool) -> Self {
        Self::Bool(data)
    }
}

impl From<i32> for IODataPtr<'static> {
    fn from(data: i32) -> Self {
        Self::Int(data)
    }
}

impl From<f32> for IODataPtr<'static> {
    fn from(data: f32) -> Self {
        Self::Float(data)
    }
}

impl<'a> From<&'a [bool]> for IODataPtr<'a> {
    fn from(data: &'a [bool]) -> Self {
        Self::BoolArray(data)
    }
}

impl<'a> From<&'a [i32]> for IODataPtr<'a> {
    fn from(data: &'a [i32]) -> Self {
        Self::IntArray(data)
    }
}

impl<'a> From<&'a [f32]> for IODataPtr<'a> {
    fn from(data: &'a [f32]) -> Self {
        Self::FloatArray(data)
    }
}

impl<'a> From<&'a OwnedIOData> for IODataPtr<'a> {
    fn from(data: &'a OwnedIOData) -> Self {
        data.borrow()
    }
}

impl<'a> IODataPtr<'a> {
    pub fn get_data_type(&self) -> IOType {
        match self {
            Self::Bool(..) => IOType::Bool,
            Self::Int(..) => IOType::Int,
            Self::Float(..) => IOType::Float,
            Self::BoolArray(arr) => IOType::BoolArray(arr.len()),
            Self::IntArray(arr) => IOType::IntArray(arr.len()),
            Self::FloatArray(arr) => IOType::FloatArray(arr.len()),
        }
    }

    pub fn to_owned(&self) -> OwnedIOData {
        match self {
            Self::Bool(value) => OwnedIOData::Bool(*value),
            Self::Int(value) => OwnedIOData::Int(*value),
            Self::Float(value) => OwnedIOData::Float(*value),
            Self::BoolArray(slice_ptr) => {
                OwnedIOData::BoolArray(Vec::from(*slice_ptr).into_boxed_slice())
            }
            Self::IntArray(slice_ptr) => {
                OwnedIOData::IntArray(Vec::from(*slice_ptr).into_boxed_slice())
            }
            Self::FloatArray(slice_ptr) => {
                OwnedIOData::FloatArray(Vec::from(*slice_ptr).into_boxed_slice())
            }
        }
    }

    pub fn unwrap_bool(self) -> bool {
        if let Self::Bool(value) = self {
            value
        } else {
            panic!("Tried to call unwrap_bool on {:?}", self)
        }
    }

    pub fn unwrap_int(self) -> i32 {
        if let Self::Int(value) = self {
            value
        } else {
            panic!("Tried to call unwrap_int on {:?}", self)
        }
    }

    pub fn unwrap_float(self) -> f32 {
        if let Self::Float(value) = self {
            value
        } else {
            panic!("Tried to call unwrap_float on {:?}", self)
        }
    }

    pub fn unwrap_bool_array(self) -> &'a [bool] {
        if let Self::BoolArray(value) = self {
            value
        } else {
            panic!("Tried to call unwrap_bool_array on {:?}", self)
        }
    }

    pub fn unwrap_int_array(self) -> &'a [i32] {
        if let Self::IntArray(value) = self {
            value
        } else {
            panic!("Tried to call unwrap_int_array on {:?}", self)
        }
    }

    pub fn unwrap_float_array(self) -> &'a [f32] {
        if let Self::FloatArray(value) = self {
            value
        } else {
            panic!("Tried to call unwrap_float_array on {:?}", self)
        }
    }

    fn write_packed_data(&self, target: &mut [u8]) {
        assert!(self.get_data_type().get_packed_size() == target.len());
        match self {
            Self::Bool(value) => target[0] = if *value { 1 } else { 0 },
            Self::Int(value) => {
                let bytes = value.to_ne_bytes();
                for i in 0..4 {
                    target[i] = bytes[i];
                }
            }
            Self::Float(value) => {
                let bytes = value.to_ne_bytes();
                for i in 0..4 {
                    target[i] = bytes[i];
                }
            }
            Self::BoolArray(arr) => {
                for (index, value) in arr.iter().enumerate() {
                    target[index] = if *value { 1 } else { 0 };
                }
            }
            Self::IntArray(arr) => {
                for (index, value) in arr.iter().enumerate() {
                    let bytes = value.to_ne_bytes();
                    for i in 0..4 {
                        target[index * 4 + i] = bytes[i];
                    }
                }
            }
            Self::FloatArray(arr) => {
                for (index, value) in arr.iter().enumerate() {
                    let bytes = value.to_ne_bytes();
                    for i in 0..4 {
                        target[index * 4 + i] = bytes[i];
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum OwnedIOData {
    Bool(bool),
    Int(i32),
    Float(f32),
    BoolArray(Box<[bool]>),
    IntArray(Box<[i32]>),
    FloatArray(Box<[f32]>),
}

impl OwnedIOData {
    pub fn borrow(&self) -> IODataPtr {
        match self {
            Self::Bool(value) => IODataPtr::Bool(*value),
            Self::Int(value) => IODataPtr::Int(*value),
            Self::Float(value) => IODataPtr::Float(*value),
            Self::BoolArray(value) => IODataPtr::BoolArray(&value[..]),
            Self::IntArray(value) => IODataPtr::IntArray(&value[..]),
            Self::FloatArray(value) => IODataPtr::FloatArray(&value[..]),
        }
    }
}

pub struct DataPacker {
    data: Vec<u8>,
    format: Vec<IOType>,
    offsets: Vec<usize>,
}

impl DataPacker {
    pub fn new(argument_types: Vec<IOType>) -> Self {
        let mut data_size = 0;
        let mut offsets = Vec::with_capacity(argument_types.len());
        for ptype in &argument_types {
            offsets.push(data_size);
            data_size += ptype.get_packed_size();
        }
        Self {
            data: vec![0; data_size],
            format: argument_types,
            offsets,
        }
    }

    pub fn set_argument(&mut self, index: usize, data: IODataPtr) {
        assert!(index < self.format.len());
        let data_type = data.get_data_type();
        assert!(data_type == self.format[index]);
        let data_len = data_type.get_packed_size();
        let offset = self.offsets[index];
        data.write_packed_data(&mut self.data[offset..offset + data_len]);
    }

    pub(crate) fn borrow_packed_data(&mut self) -> &mut [u8] {
        &mut self.data[..]
    }

    pub fn borrow_format(&self) -> &[IOType] {
        &self.format[..]
    }
}

pub struct DataUnpacker {
    data: Vec<u8>,
    format: Vec<IOType>,
    offsets: Vec<usize>,
}

impl DataUnpacker {
    pub fn new(argument_types: Vec<IOType>) -> Self {
        let mut data_size = 0;
        let mut offsets = Vec::with_capacity(argument_types.len());
        for ptype in &argument_types {
            offsets.push(data_size);
            data_size += ptype.get_packed_size();
        }
        Self {
            data: vec![0; data_size],
            format: argument_types,
            offsets,
        }
    }

    pub unsafe fn get_argument(&self, index: usize) -> IODataPtr {
        assert!(index < self.format.len());
        let data_type = &self.format[index];
        let offset = self.offsets[index];
        match data_type {
            IOType::Bool => IODataPtr::Bool(self.data[offset] > 0),
            IOType::Int => IODataPtr::Int(i32::from_ne_bytes([
                self.data[offset + 0],
                self.data[offset + 1],
                self.data[offset + 2],
                self.data[offset + 3],
            ])),
            IOType::Float => IODataPtr::Float(f32::from_ne_bytes([
                self.data[offset + 0],
                self.data[offset + 1],
                self.data[offset + 2],
                self.data[offset + 3],
            ])),
            IOType::BoolArray(len) => IODataPtr::BoolArray(std::slice::from_raw_parts(
                &self.data[offset] as *const u8 as _,
                *len,
            )),
            IOType::IntArray(len) => IODataPtr::IntArray(std::slice::from_raw_parts(
                &self.data[offset] as *const u8 as _,
                *len,
            )),
            IOType::FloatArray(len) => IODataPtr::FloatArray(std::slice::from_raw_parts(
                &self.data[offset] as *const u8 as _,
                *len,
            )),
        }
    }

    pub(crate) fn borrow_packed_data(&mut self) -> &mut [u8] {
        &mut self.data[..]
    }

    pub fn borrow_format(&self) -> &[IOType] {
        &self.format[..]
    }
}
