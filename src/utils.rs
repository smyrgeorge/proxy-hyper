/// Removes whitespaces and new line characters.
/// For more information check is_whitespace().
pub fn strip_whitespaces(s: String) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
