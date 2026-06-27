use syntect::parsing::{SyntaxReference, SyntaxSet};

pub fn expires(size: u64) -> i64 {
    let min_age = 5.0;
    let max_age = 365.0;
    let max_size = 512.0;

    let size: f64 = ((size / 1024) / 1024) as f64;

    let mut expiry = min_age + (-max_age + min_age) * (size / max_size - 1.0).powf(3.0);

    if expiry < 5.0 {
        expiry = 5.0
    };

    (expiry * 86400.0) as i64
}

#[allow(unused)]
pub fn find_syntax_by_name<F>(ss: &SyntaxSet, predicate: F) -> Option<&SyntaxReference>
where
    F: Fn(&&SyntaxReference) -> bool,
{
    ss.syntaxes().iter().find(predicate)
}
