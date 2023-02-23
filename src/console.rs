use crate::utils::error::Error;

pub struct OutputLine {
    pub text: String,
    pub wrapped_error: Option<Error>,
}

impl OutputLine {
    pub fn new(text: String) -> OutputLine {
        OutputLine {
            text,
            wrapped_error: None,
        }
    }
}
