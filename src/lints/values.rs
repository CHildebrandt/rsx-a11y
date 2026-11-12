//! Value validators shared by the lint rules.

/// Autofill field names that may be preceded by a contact token
/// (`home`, `work`, `mobile`, `fax`, `pager`).
const CONTACT_FIELDS: &[&str] = &[
    "tel",
    "tel-country-code",
    "tel-national",
    "tel-area-code",
    "tel-local",
    "tel-local-prefix",
    "tel-local-suffix",
    "tel-extension",
    "email",
    "impp",
];

/// Autofill field names that cannot take a contact token.
const FIELDS: &[&str] = &[
    "name",
    "honorific-prefix",
    "given-name",
    "additional-name",
    "family-name",
    "honorific-suffix",
    "nickname",
    "username",
    "new-password",
    "current-password",
    "one-time-code",
    "organization-title",
    "organization",
    "street-address",
    "address-line1",
    "address-line2",
    "address-line3",
    "address-level4",
    "address-level3",
    "address-level2",
    "address-level1",
    "country",
    "country-name",
    "postal-code",
    "cc-name",
    "cc-given-name",
    "cc-additional-name",
    "cc-family-name",
    "cc-number",
    "cc-exp",
    "cc-exp-month",
    "cc-exp-year",
    "cc-csc",
    "cc-type",
    "transaction-currency",
    "transaction-amount",
    "language",
    "bday",
    "bday-day",
    "bday-month",
    "bday-year",
    "sex",
    "url",
    "photo",
];

/// Validate an `autocomplete` value against the HTML autofill grammar:
///
/// ```text
/// "on" | "off"
/// | [section-*] [shipping|billing] [home|work|mobile|fax|pager] field [webauthn]
/// ```
///
/// where a contact token (`home`, `work`, …) is only allowed before
/// telephone, email and impp fields. Tokens are ASCII case-insensitive.
pub(super) fn is_valid_autocomplete(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let mut tokens: Vec<&str> = lower.split_whitespace().collect();

    match tokens.as_slice() {
        [] => return false,
        ["on" | "off"] => return true,
        _ => {}
    }

    if tokens.last() == Some(&"webauthn") {
        tokens.pop();
    }

    let mut rest = tokens.as_slice();
    if let [first, tail @ ..] = rest
        && first
            .strip_prefix("section-")
            .is_some_and(|s| !s.is_empty())
    {
        rest = tail;
    }
    if let [first, tail @ ..] = rest
        && matches!(*first, "shipping" | "billing")
    {
        rest = tail;
    }
    match rest {
        [field] => FIELDS.contains(field) || CONTACT_FIELDS.contains(field),
        [contact, field] => {
            matches!(*contact, "home" | "work" | "mobile" | "fax" | "pager")
                && CONTACT_FIELDS.contains(field)
        }
        _ => false,
    }
}

/// Validate a BCP 47 language tag (a structural check, not a registry lookup).
///
/// Accepts a 2–3 letter (or 5–8 letter) primary language subtag followed by
/// alphanumeric subtags of 1–8 characters, plus private-use (`x-…`) and
/// grandfathered `i-…` tags.
pub(super) fn is_valid_lang(lang: &str) -> bool {
    let lang = lang.trim();
    let mut parts = lang.split('-');
    let Some(primary) = parts.next() else {
        return false;
    };
    let primary_ok = match primary.len() {
        1 => primary.eq_ignore_ascii_case("x") || primary.eq_ignore_ascii_case("i"),
        2 | 3 | 5..=8 => true,
        _ => false,
    } && primary.chars().all(|c| c.is_ascii_alphabetic());
    if !primary_ok {
        return false;
    }
    let rest: Vec<&str> = parts.collect();
    if primary.len() == 1 && rest.is_empty() {
        return false;
    }
    rest.iter().all(|part| {
        (1..=8).contains(&part.len()) && part.chars().all(|c| c.is_ascii_alphanumeric())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autocomplete_grammar() {
        for valid in [
            "on",
            "off",
            "email",
            "work email",
            "home tel",
            "section-blue shipping postal-code",
            "billing cc-number",
            "shipping mobile tel",
            "username webauthn",
            "Current-Password",
        ] {
            assert!(is_valid_autocomplete(valid), "{valid:?} should be valid");
        }
        for invalid in [
            "",
            "banana",
            "on off",
            "work name",
            "email work",
            "section- email",
            "shipping billing email",
            "webauthn",
        ] {
            assert!(
                !is_valid_autocomplete(invalid),
                "{invalid:?} should be invalid"
            );
        }
    }

    #[test]
    fn lang_tags() {
        for valid in [
            "en",
            "en-US",
            "zh-Hans-CN",
            "sgn-BE-FR",
            "x-klingon",
            "es-419",
        ] {
            assert!(is_valid_lang(valid), "{valid:?} should be valid");
        }
        for invalid in ["", "e", "123", "en-", "en_US", "toolongprimary", "x"] {
            assert!(!is_valid_lang(invalid), "{invalid:?} should be invalid");
        }
    }
}
