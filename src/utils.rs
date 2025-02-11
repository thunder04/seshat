pub fn determine_possesive(s: &str) -> &'static str {
    if s.ends_with(['s', 'S']) { "'" } else { "'s" }
}
