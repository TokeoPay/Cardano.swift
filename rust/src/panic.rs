use super::error::CError;
use super::string::IntoCString;
use std::any::Any;
use std::panic;

pub type Result<T> = std::result::Result<T, CError>;

pub trait IntoResult<T> {
    fn into_result(self) -> Result<T>;
}

pub trait CResponse<T> {
    fn response(&self, val: &mut T, error: &mut CError) -> bool;
}

pub trait Zip<T1> {
    fn zip<T2>(self, other: Result<T2>) -> Result<(T1, T2)>;
}

impl<T1> Zip<T1> for Result<T1> {
    fn zip<T2>(self, other: Result<T2>) -> Result<(T1, T2)> {
        self.and_then(|val1| other.map(|val2| (val1, val2)))
    }
}

fn string_from_panic_err(err: Box<dyn Any>) -> String {
    if let Some(string) = err.downcast_ref::<String>() {
        string.clone()
    } else if let Some(string) = err.downcast_ref::<&'static str>() {
        String::from(*string)
    } else {
        format!("Reason: {:?}", err)
    }
}

impl<T> IntoResult<T> for std::result::Result<T, Box<dyn Any + Send + 'static>> {
    fn into_result(self) -> Result<T> {
        self.map_err(|err| CError::Panic(string_from_panic_err(err).into_cstr()))
    }
}

impl<T> IntoResult<T> for std::result::Result<T, Box<dyn Any + 'static>> {
    fn into_result(self) -> Result<T> {
        self.map_err(|err| CError::Panic(string_from_panic_err(err).into_cstr()))
    }
}

impl<T, E> IntoResult<T> for std::result::Result<T, E>
where
    E: Into<CError>,
{
    fn into_result(self) -> Result<T> {
        self.map_err(|err| err.into())
    }
}

impl<T: Copy> CResponse<T> for Result<T> {
    fn response(&self, val: &mut T, error: &mut CError) -> bool {
        match self {
            Err(err) => {
                *error = *err;
                false
            }
            Ok(value) => {
                *val = *value;
                true
            }
        }
    }
}

impl<T: Copy> CResponse<*mut T> for Result<Option<T>> {
    fn response(&self, val: &mut *mut T, error: &mut CError) -> bool {
        match self {
            Err(err) => {
                *error = *err;
                false
            }
            Ok(value) => {
                match value {
                    None => {
                        *val = std::ptr::null_mut();
                    }
                    Some(value) => unsafe {
                        **val = *value;
                    },
                }
                true
            }
        }
    }
}

#[allow(dead_code)]
pub fn handle_exception<F: FnOnce() -> R + panic::UnwindSafe, R>(func: F) -> Result<R> {
    handle_exception_result(|| Ok(func()))
}

pub fn handle_exception_result<F: FnOnce() -> Result<R> + panic::UnwindSafe, R>(
    func: F,
) -> Result<R> {
    panic::catch_unwind(func).into_result().and_then(|res| res)
}

#[allow(dead_code)]
pub fn hide_exceptions() {
    panic::set_hook(Box::new(|_| {}));
}
