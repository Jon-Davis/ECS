use std::any::Any;

/// The Component trait is need to be implemented by any
/// struct that wants to be stored in thre resources
pub trait Component: Any + Send + Sync + 'static {}
