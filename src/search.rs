use regex::Regex;

pub struct SplitWindows {
    expr: Regex,
}

impl SplitWindows {
    pub fn new() -> Self {
        Self {
            expr: Regex::new(r#"(^\w|\b\w)"#).unwrap(),
        }
    }

    pub fn windows<'a, 'b: 'a>(
        &'a self,
        text: &'b str,
        len: usize,
    ) -> impl Iterator<Item = &'b str> + 'a {
        self.expr
            .find_iter(text)
            .filter_map(move |cx| text.get(cx.start()..cx.start() + len))
    }
}

#[cfg(test)]
mod tests {
    use super::SplitWindows;

    static TEXT: &str = "HOW NOW BROWN COW";

    #[test]
    fn windows() {
        let splitter = SplitWindows::new();
        let windows: Vec<_> = splitter.windows(TEXT, 3).collect();
        assert_eq!(&["HOW", "NOW", "BRO", "COW",], &*windows)
    }
}
