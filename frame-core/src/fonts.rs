use font_kit::source::SystemSource;

#[must_use]
pub fn list_system_font_families() -> Vec<String> {
    let source = SystemSource::new();
    sorted_font_families(source.all_families().unwrap_or_default())
}

fn sorted_font_families(mut families: Vec<String>) -> Vec<String> {
    families.sort_unstable_by_key(|family| family.to_lowercase());
    families
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_font_families_orders_names_case_insensitively() {
        let actual = sorted_font_families(vec![
            "Zapfino".to_string(),
            "arial".to_string(),
            "Helvetica".to_string(),
        ]);

        assert_eq!(actual, ["arial", "Helvetica", "Zapfino"]);
    }
}
