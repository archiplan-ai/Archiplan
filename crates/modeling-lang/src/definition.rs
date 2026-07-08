//! Definition prose: the shared validator every door consults.
//!
//! A definition is one sentence of identity prose — what its element *is* —
//! at most [`MAX_CHARS`] characters
//! (`archi/requirements/element-definitions/`). Obligation vocabulary (must,
//! should, shall, ensures, handles) rejects wherever it stands: obligations
//! live in requirement docs, not in definitions. The source attach pass, the
//! statement schema and the engine's define path all call [`validate`] over
//! [`normalize`]d text, so no stored definition can exist that the parser
//! would refuse to read back.

/// Definition length ceiling, in characters.
pub const MAX_CHARS: usize = 240;

/// The obligation vocabulary, matched case-insensitively as whole words.
const MODAL_WORDS: [&str; 5] = ["must", "should", "shall", "ensures", "handles"];

/// Normalize definition text: whitespace runs (including line joins in a
/// block comment) collapse to single spaces, ends trimmed. Only normalized
/// text is ever stored, so the canonical render emits exactly what a
/// re-parse reproduces.
pub fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Validate normalized definition text; `Err` names the rule broken.
pub fn validate(text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Err("a definition cannot be empty".into());
    }
    let count = text.chars().count();
    if count > MAX_CHARS {
        return Err(format!(
            "a definition is at most {MAX_CHARS} characters (this one is {count})"
        ));
    }
    // One sentence: a terminator followed by whitespace is a boundary, so a
    // trailing period is fine and a dot inside a token (`mod.rs`) is not a
    // boundary. Normalized text has no trailing whitespace, so any boundary
    // means more text follows.
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if matches!(c, '.' | '!' | '?') && chars.peek().is_some_and(|n| n.is_whitespace()) {
            return Err(
                "a definition is a single sentence — text continues after a sentence end".into(),
            );
        }
    }
    // Identity, not obligation: the modal vocabulary rejects wherever it
    // stands, splice or no splice — whole words only, so `mustard` and
    // `handler` pass.
    for word in text.split(|c: char| !c.is_alphanumeric()) {
        let lower = word.to_lowercase();
        if MODAL_WORDS.contains(&lower.as_str()) {
            return Err(format!(
                "obligations live in requirement docs, not definitions — `{word}` is obligation vocabulary"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_collapses_whitespace() {
        assert_eq!(normalize("  a   b \t c\n d  "), "a b c d");
        assert_eq!(normalize(""), "");
    }

    #[test]
    fn identity_sentences_pass() {
        for ok in [
            "runs the verbs",
            "the operator — human or agent — outside the tool",
            "offside-rule tokenizer; rejects tabs in indentation",
            "save, anchor, remint, list, show, diff (archived or live), current",
            "source/mod.rs — coordinates the pipeline per module",
            "The interface in one place: the node and its declared ports.",
        ] {
            assert_eq!(validate(ok), Ok(()), "{ok}");
        }
    }

    #[test]
    fn length_gate_sits_at_240() {
        let at = "x".repeat(240);
        assert_eq!(validate(&at), Ok(()));
        let over = "x".repeat(241);
        let msg = validate(&over).unwrap_err();
        assert!(msg.contains("240") && msg.contains("241"), "{msg}");
    }

    #[test]
    fn second_sentences_reject_but_token_dots_pass() {
        assert!(validate("does one thing. does another").is_err());
        assert!(validate("one! two").is_err());
        assert!(validate("one? two").is_err());
        assert_eq!(validate("lives in mod.rs beside plan.json"), Ok(()));
        assert_eq!(validate("ends with a period."), Ok(()));
    }

    #[test]
    fn modal_words_reject_everywhere() {
        for bad in [
            "must reject tabs",
            "tokenizer, must reject tabs",
            "tokenizer; must reject tabs",
            "tokenizer — must reject tabs",
            "tokenizer: must reject tabs",
            "SHOULD hold",
            "it Shall pass",
            "ensures determinism",
            "handles the login flow",
        ] {
            let msg = validate(bad).unwrap_err();
            assert!(msg.contains("obligation"), "{bad}: {msg}");
        }
    }

    #[test]
    fn modal_lookalikes_pass_whole_word_matching() {
        for ok in ["a mustard-colored handler", "marshall the tokens", "the shoulder"] {
            assert_eq!(validate(ok), Ok(()), "{ok}");
        }
    }
}
