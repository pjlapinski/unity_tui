pub trait ErrToStr<T, U> {
    fn err_to_str(self) -> Result<T, String>;
}
impl<T, U> ErrToStr<T, U> for Result<T, U> where U: ToString {
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
    fn err_to_str(self) -> Result<T, String> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.to_string()),
        }
    }
}
