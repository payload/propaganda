use std::str::FromStr;
use tide::http::Mime;

pub fn html() -> Mime {
    Mime::from_str("text/html; charset=utf-8").unwrap()
}

pub fn json() -> Mime {
    Mime::from_str("application/json; charset=utf-8").unwrap()
}
