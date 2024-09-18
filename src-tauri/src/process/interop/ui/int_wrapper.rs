pub  enum IntWrapper {
    Int32(i32),
    Int64(i64),
}

impl IntWrapper {

    pub fn new_from_i32(value: i32) -> Self {
        IntWrapper::Int32(value)
    }


    pub fn new_from_i64(value: i64) -> Self {
        IntWrapper::Int64(value)
    }

 
    pub fn get_i64(&self) -> i64 {
        match *self {
            IntWrapper::Int32(value) => value as i64,
            IntWrapper::Int64(value) => value,
        }
    }

    
    pub fn get_i32(&self) -> Option<i32> {
        match *self {
            IntWrapper::Int32(value) => Some(value),
            IntWrapper::Int64(value) => {
                if value as i32 as i64 == value {
                    Some(value as i32)
                } else {
                    None
                }
            }
        }
    }
}