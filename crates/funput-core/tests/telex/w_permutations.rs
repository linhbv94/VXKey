use funput_core::{InputMethod, ToneStyle, apply};

struct Case {
    base: &'static str,
    target: &'static str,
}

fn insert_w(base: &str, boundary: usize) -> String {
    let offset = base
        .char_indices()
        .nth(boundary)
        .map_or(base.len(), |p| p.0);
    format!("{}w{}", &base[..offset], &base[offset..])
}

fn type_keys(keys: &str, style: ToneStyle) -> String {
    keys.chars().fold(String::new(), |buffer, key| {
        apply(&buffer, key, InputMethod::Telex, style).text
    })
}

fn first_vowel_boundary(base: &str) -> usize {
    let lower = base.to_ascii_lowercase();
    if lower.starts_with("qu") || lower.starts_with("gi") {
        return 3;
    }
    base.chars()
        .position(|ch| matches!(ch.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u' | 'y'))
        .map_or(1, |idx| idx + 1)
}

fn assert_converges(case: &Case, style: ToneStyle) {
    let start = first_vowel_boundary(case.base);
    for boundary in start..=case.base.chars().count() {
        let keys = insert_w(case.base, boundary);
        assert_eq!(
            type_keys(&keys, style),
            case.target,
            "did not converge: {keys}"
        );
    }
}

#[test]
fn snapshots_breve_and_single_horn_permutations() {
    for case in [
        Case {
            base: "lams",
            target: "lắm",
        },
        Case {
            base: "con",
            target: "cơn",
        },
        Case {
            base: "moif",
            target: "mời",
        },
    ] {
        assert_converges(&case, ToneStyle::Traditional);
    }
}

#[test]
fn snapshots_compound_permutations() {
    for case in [
        Case {
            base: "truongf",
            target: "trường",
        },
        Case {
            base: "nguoif",
            target: "người",
        },
        Case {
            base: "mua",
            target: "mưa",
        },
        Case {
            base: "uu",
            target: "ưu",
        },
        Case {
            base: "thuor",
            target: "thuở",
        },
        Case {
            base: "quois",
            target: "quới",
        },
        Case {
            base: "TRUONGF",
            target: "TRƯỜNG",
        },
    ] {
        for style in [ToneStyle::Traditional, ToneStyle::Modern] {
            assert_converges(&case, style);
        }
    }
}

#[test]
fn snapshots_all_tones_case_and_styles() {
    for (base, target) in [
        ("lans", "lắn"),
        ("lanf", "lằn"),
        ("lanr", "lẳn"),
        ("lanx", "lẵn"),
        ("lanj", "lặn"),
    ] {
        for style in [ToneStyle::Traditional, ToneStyle::Modern] {
            let start = first_vowel_boundary(base);
            for boundary in start..=base.chars().count() {
                assert_eq!(type_keys(&insert_w(base, boundary), style), target);
            }
        }
    }
    assert_eq!(type_keys("LAWMS", ToneStyle::Traditional), "LẮM");
    assert_eq!(type_keys("LAMWS", ToneStyle::Traditional), "LẮM");
}

#[test]
fn tone_between_shortcut_w_and_compound_w_keeps_the_horn() {
    for style in [ToneStyle::Traditional, ToneStyle::Modern] {
        for method in [InputMethod::Telex, InputMethod::TelexAdvanced] {
            for (keys, expected) in [
                ("ddwojwc", "được"),
                ("ddwowjc", "được"),
                ("ddwowcj", "được"),
                ("DDWOJWC", "ĐƯỢC"),
                ("mwowjt", "mượt"),
                ("mwojwt", "mượt"),
                ("thuowr", "thuở"),
                ("quowr", "quở"),
                ("uoww", "uow"),
                ("ww", "w"),
            ] {
                let actual = keys.chars().fold(String::new(), |buffer, key| {
                    apply(&buffer, key, method, style).text
                });
                assert_eq!(actual, expected, "{keys}, {method:?}, {style:?}");
            }
        }
    }
}
