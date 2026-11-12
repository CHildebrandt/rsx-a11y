//! Value validators shared by the lint rules.

/// Validate an autocomplete attribute value per the HTML spec.
pub(super) fn is_valid_autocomplete(value: &str) -> bool {
    const VALID_TOKENS: &[&str] = &[
        "on",
        "off",
        "name",
        "honorific-prefix",
        "given-name",
        "additional-name",
        "family-name",
        "honorific-suffix",
        "nickname",
        "email",
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
        "tel",
        "tel-country-code",
        "tel-national",
        "tel-area-code",
        "tel-local",
        "tel-extension",
        "impp",
        "url",
        "photo",
        "webauthn",
    ];
    const SECTION_PREFIXES: &[&str] = &["shipping", "billing"];

    let tokens: Vec<&str> = value.split_whitespace().collect();
    if tokens.is_empty() {
        return false;
    }

    let mut idx = 0;

    // Optional section-* prefix
    if tokens.get(idx).is_some_and(|t| t.starts_with("section-")) {
        idx += 1;
    }

    // Optional shipping/billing
    if tokens
        .get(idx)
        .is_some_and(|t| SECTION_PREFIXES.contains(t))
    {
        idx += 1;
    }

    // Must have at least the field token
    if idx >= tokens.len() {
        return false;
    }

    // Remaining token must be a valid field name
    let field = tokens[idx];
    idx += 1;

    // Must be the last token
    if idx != tokens.len() {
        return false;
    }

    VALID_TOKENS.contains(&field)
}

/// Validate a BCP 47 language tag (simplified check per jsx-a11y lang rule).
pub(super) fn is_valid_lang(lang: &str) -> bool {
    let lang = lang.trim();
    if lang.is_empty() {
        return false;
    }

    let parts: Vec<&str> = lang.split('-').collect();
    let primary = parts[0];

    // Primary language subtag must be 2-3 ASCII letters
    if primary.len() < 2 || primary.len() > 3 {
        return false;
    }
    if !primary.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }

    // All remaining subtags must be non-empty and alphanumeric
    for part in &parts[1..] {
        if part.is_empty() || part.len() > 8 {
            return false;
        }
        if !part.chars().all(|c| c.is_ascii_alphanumeric()) {
            return false;
        }
    }
    true
}
