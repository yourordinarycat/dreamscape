macro_rules! CONNECTION_ID_ATTR_NAME {
    () => {
        "data-blg-connection-id"
    };
}

pub const CONNECTION_ID_ATTR: &str = CONNECTION_ID_ATTR_NAME!();
pub const CONNECTION_ID_ATTR_SELECTOR: &str = concat!("[", CONNECTION_ID_ATTR_NAME!(), "]");
