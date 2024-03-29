

pub enum Trace<T> {
    Ok(T),
    Err(String)
}

impl<T> Trace<T> {
    pub fn unwrap(self) -> T {
        match self {
            Trace::Ok(v) => v,
            Trace::Err(string) => {
                panic!("Panicked on a Trace unwrap value with error: {}", string)
            },
        }
    }
}

pub mod macros {
    #[macro_export]
    macro_rules! start_trace {
        ($message:expr) => {
            return Trace::Err(format!("Error in file {} in function {} at line {} with message: 
            {}", file!(), stdext::function_name!(), line!(), $message))
        };
    }

    #[macro_export]
    macro_rules! trace {
        ($expression: expr) => {
            match $expression {
                Trace::Ok(v) => v,
                Trace::Err(msg) => {
                    return Trace::Err(msg + &format!("\n  ...in file {} in function {} at line {},", 
                    file!(), stdext::function_name!(), line!()))
                },
            }
        };
    }
}