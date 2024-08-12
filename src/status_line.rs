#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum StatusLine {
    Absolute,
    Relative,
    None,
}

impl TryFrom<String> for StatusLine {
    type Error = (String, String);

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match value.as_ref() {
            "absolute" => Ok(StatusLine::Absolute),
            "relative" => Ok(StatusLine::Relative),
            "none" => Ok(StatusLine::None),
            _ => Err((
                "The possible value is one of \"absolute\", \"relative\", or \"none\"".to_string(),
                value,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from() {
        assert_eq!(
            StatusLine::try_from("absolute".to_string()),
            Ok(StatusLine::Absolute)
        );
        assert_eq!(
            StatusLine::try_from("relative".to_string()),
            Ok(StatusLine::Relative)
        );
        assert_eq!(
            StatusLine::try_from("none".to_string()),
            Ok(StatusLine::None)
        );
        assert_eq!(
            StatusLine::try_from("invalid".to_string()),
            Err((
                "The possible value is one of \"absolute\", \"relative\", or \"none\"".to_string(),
                "invalid".to_string(),
            ))
        );
    }
}
