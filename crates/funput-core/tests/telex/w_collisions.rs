use funput_core::InputMethod;

fn type_keys(keys: &str) -> String {
    crate::support::type_keys(InputMethod::Telex, keys)
}

#[test]
fn snapshots_leading_w_and_latin_collisions() {
    // Leading `w` in Telex converts to `ư` wherever the syllable has no vowel yet;
    // doubling `w` reverts to literal `w`.
    assert_eq!(type_keys("w"), "ư");
    assert_eq!(type_keys("W"), "Ư");
    assert_eq!(type_keys("thw"), "thư");
    assert_eq!(type_keys("vwng"), "vưng");
    assert_eq!(type_keys("thwr"), "thử");
    assert_eq!(type_keys("nhwng"), "nhưng");
    assert_eq!(type_keys("chwx"), "chữ");
    assert_eq!(type_keys("vwaf"), "vừa");
    assert_eq!(type_keys("swwift"), "swift");
    assert_eq!(type_keys("sawn"), "săn");
}

#[test]
fn snapshots_repeated_pending_w_and_boundaries() {
    assert_eq!(type_keys("lwwa"), "lwa");
    assert_eq!(
        crate::support::type_words(InputMethod::Telex, "lw a"),
        "lư a"
    );
    assert_eq!(
        crate::support::type_words(InputMethod::Telex, "lww a"),
        "lw a"
    );
}
