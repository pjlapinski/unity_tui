use std::io::{Error, ErrorKind};

macro_rules! hash_map {
    () => {
        std::collections::HashMap::new()
    };
    ($(($key:expr, $value:expr)),*$(,)?) => {{
            let mut map = std::collections::HashMap::new();
            $(
            map.insert($key, $value);
            )*
            map
        }};
}

macro_rules! hash_set {
    () => {
        std::collections::HashSet::new()
    };
    ($($value:expr),*$(,)?) => {{
            let mut map = std::collections::HashSet::new();
            $(
            map.insert($value);
            )*
            map
        }};
}

pub(crate) use hash_map;
pub(crate) use hash_set;

pub trait ErrTo<T, U> {
    /// Maps a Result<T, U> to Result<T, String> by calling .to_string on a contained error.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Result<i32, &str> = Ok(15);
    /// assert_eq!(Ok(15), x.err_to_str());
    ///
    /// let x: Result<i32, &str> = Err("hey");
    /// assert_eq!(Err("hey".to_owned()), x.err_to_str());
    /// ```
    fn err_to_str(self) -> Result<T, String>;
    fn err_to_io_err(self) -> Result<T, Error>;
}

impl<T, U> ErrTo<T, U> for Result<T, U>
where
    U: ToString,
{
    fn err_to_str(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }

    fn err_to_io_err(self) -> Result<T, Error> {
        self.map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
    }
}

pub trait PairWith<T, U> {
    /// Creates a tuple (T, U) by moving the given arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// let (a, b) = 1.pair_with(2);
    /// assert_eq!(a, 1);
    /// assert_eq!(b, 2);
    /// ```
    fn pair_with(self, other: U) -> (T, U);
}

impl<T, U> PairWith<T, U> for T {
    fn pair_with(self, other: U) -> (T, U) {
        (self, other)
    }
}
