pub trait CollectMapJoin<X> {
    /// Utility function to collect an iterator, map it with a function, and join it into a final string.
    fn collect_map_join(self, sep: &str, f: fn(X) -> String) -> String;

    /// Utility function to collect an iterator, map/filter it with a function, and join it into a final string.
    fn collect_filter_map_join(self, sep: &str, f: fn(X) -> Option<String>) -> String;
}

impl<T: Iterator<Item = X>, X> CollectMapJoin<X> for T {
    fn collect_map_join(self, sep: &str, f: fn(X) -> String) -> String {
        // When https://github.com/rust-lang/rust/issues/79524 is fixed, this can probably be simplified
        // self.map(f).intersperse(sep).collect()
        self.map(f).collect::<Vec<String>>().join(sep)
    }

    fn collect_filter_map_join(self, sep: &str, f: fn(X) -> Option<String>) -> String {
        self.filter_map(f).collect::<Vec<String>>().join(sep)
    }
}
