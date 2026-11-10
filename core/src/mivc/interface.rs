use serde::{de::DeserializeOwned, Serialize};

/// The "format" of a piece of data. Anything saved to YAML implements this.
///
/// FORMAT is the name written at the top of the file.
/// VERSION goes up when the shape changes, so old files can still be recognized.
pub trait Interface: Serialize + DeserializeOwned {
    const FORMAT: &'static str;
    const VERSION: u32;
}
