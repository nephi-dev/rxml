#[macro_export]
macro_rules! f_str {
    ($e:expr) => {
        String::from($e)
    };
}
#[macro_export]
macro_rules! f_utf {
    ($e:expr) => {
        String::from_utf8($e.to_vec()).unwrap()
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_f_str() {
        assert_eq!(f_str!("test"), String::from("test"));
    }
    #[test]
    fn test_f_utf() {
        assert_eq!(f_utf!(b"test"), String::from("test"));
    }
}
