use funput_core::InputMethod;
use funput_engine::{Action, Engine};

#[test]
fn resolves_with_minimal_diff_and_preserves_raw_keys() {
    let (buffer, results) = crate::support::type_keys(InputMethod::Telex, "vwng");
    assert_eq!(buffer, "vưng");
    assert_eq!(results[1].action, Action::Send);
    assert_eq!(results[1].backspace, 0);
    assert_eq!(results[1].output, "ư");

    let mut engine = Engine::new();
    for key in "sawn".chars() {
        engine.process_char(key);
    }
    assert_eq!(engine.buffer(), "săn");
    assert_eq!(engine.keys(), "sawn");
}

#[test]
fn flip_is_the_sticky_latin_escape() {
    let mut engine = Engine::new();
    for key in "sawn".chars() {
        engine.process_char(key);
    }
    engine.flip_composing();
    assert_eq!(engine.buffer(), "sawn");
    let boundary = engine.process_char(' ');
    assert_eq!(boundary.action, Action::None);
    assert!(engine.buffer().is_empty());
}

#[test]
fn literal_and_boundary_cases_do_not_leak() {
    assert_eq!(
        crate::support::app_text(InputMethod::Telex, "wwas wwow wwin swwift "),
        "was wow win swift "
    );
    assert_eq!(
        crate::support::app_text(InputMethod::Telex, "lwwa "),
        "lwa "
    );
    assert_eq!(
        crate::support::app_text(InputMethod::Telex, "lw a "),
        "lư a "
    );
}

fn insert_w(base: &str, boundary: usize) -> String {
    let offset = base
        .char_indices()
        .nth(boundary)
        .map_or(base.len(), |pair| pair.0);
    format!("{}w{}", &base[..offset], &base[offset..])
}

fn first_vowel_boundary(base: &str) -> usize {
    let lower = base.to_ascii_lowercase();
    if lower.starts_with("qu") || lower.starts_with("gi") {
        return 3;
    }
    base.chars()
        .position(|ch| matches!(ch.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u' | 'y'))
        .unwrap_or(1)
}

#[test]
fn compound_permutations_converge_through_engine() {
    for (base, expected) in [
        ("truongf", "trường"),
        ("nguoif", "người"),
        ("mua", "mưa"),
        ("uu", "ưu"),
        ("thuor", "thuở"),
        ("quois", "quới"),
    ] {
        let start = if base == "thuor" {
            3
        } else {
            first_vowel_boundary(base)
        };
        for boundary in start..=base.chars().count() {
            let keys = insert_w(base, boundary);
            assert_eq!(
                crate::support::app_text(InputMethod::Telex, &keys),
                expected,
                "{keys}"
            );
        }
    }
}

#[test]
fn compound_resolution_preserves_raw_keys() {
    let mut engine = Engine::new();
    for key in "trwuongf".chars() {
        engine.process_char(key);
    }
    assert_eq!(engine.buffer(), "trường");
    assert_eq!(engine.keys(), "trwuongf");
}
