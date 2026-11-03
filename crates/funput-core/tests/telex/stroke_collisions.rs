use funput_core::InputMethod;

fn type_keys(keys: &str) -> String {
    crate::support::type_keys(InputMethod::Telex, keys)
}

#[test]
fn delayed_stroke_behavior_is_locked() {
    for (keys, output) in [
        ("daud", "đau"),
        ("dend", "đen"),
        ("doifd", "đòi"),
        ("dad", "đa"),
        ("did", "đi"),
    ] {
        assert_eq!(type_keys(keys), output, "collision changed: {keys}");
    }
}

#[test]
fn invalid_or_repeated_pending_intents_stay_literal() {
    assert_eq!(type_keys("dwwdead"), "dwdead");
    assert_eq!(type_keys("dwwswift"), "dwswift");
    assert_eq!(type_keys("dwwd"), "dwd");
    assert_eq!(type_keys("lwwa"), "lwa");
    assert_eq!(type_keys("dwdd"), "dưd");
}
