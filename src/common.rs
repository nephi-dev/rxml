#[macro_export]
macro_rules! f_str {
    ($e:expr) => {
        String::from($e)
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_f_str() {
        assert_eq!(f_str!("test"), String::from("test"));
    }
}
