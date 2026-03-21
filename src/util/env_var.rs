/// This parses a var (either env var or cmake var) and if it contains
/// a '=' char, it assumes it's key and value, otherwise it's just going to
/// be the key and the value is gonna be an empty string
pub fn parse_var(input: &str) -> (String, String) {
    if input.contains('=') {
        let mut sp = input.split('=');
        (
            sp.next().unwrap().to_string(),
            sp.next().unwrap().to_string(),
        )
    } else {
        (input.to_string(), "".to_string())
    }
}
