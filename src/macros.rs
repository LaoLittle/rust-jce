#[macro_export]
macro_rules! check_type {
    ($ty:ty, $buf:expr) => {
        $crate::de::check_buf($buf, std::mem::size_of::<$ty>())?
    };
}
