use crate::Module;

// ui模块， todo：包含的功能还未想好
trait UiModule<T: Module + Sized>: Module {}
