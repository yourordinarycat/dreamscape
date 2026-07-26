use std::ffi::OsStr;

pub fn name_to_str(str: Option<&OsStr>) -> Result<&str, std::io::Error> {
    let os_key = str.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "File does not have a name.")
    })?;
    os_key.to_str().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "File name is not valid UTF-8.")
    })
}
