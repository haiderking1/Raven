use super::Policy;
use std::ffi::OsString;

#[test]
fn environment_values_default_to_adaptive_and_reject_misspellings() {
    // Test the exact environment resolver without mutating process-global env.
    assert_eq!(Policy::from_value(None), Ok(Policy::Adaptive));
    assert_eq!("adaptive".parse(), Ok(Policy::Adaptive));
    assert_eq!(
        Policy::from_value(Some("immediate".into())),
        Ok(Policy::Immediate)
    );
    assert_eq!("deadline".parse(), Ok(Policy::Deadline));
    for value in ["", "Immediate", " deadline", "deadline ", "fifo"] {
        let error = Policy::from_value(Some(OsString::from(value))).unwrap_err();
        assert!(error.to_string().contains("RAVEN_FRAME_PIPELINE"));
    }
}

#[cfg(unix)]
#[test]
fn non_unicode_environment_value_is_an_error_not_a_default() {
    use std::os::unix::ffi::OsStringExt;
    assert!(Policy::from_value(Some(OsString::from_vec(vec![0xff]))).is_err());
}
