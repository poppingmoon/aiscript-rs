// https://tc39.es/ecma262/multipage/global-object.html#sec-uri-handling-functions

use std::str::Utf8Error;

use percent_encoding::{
    AsciiSet, NON_ALPHANUMERIC, percent_decode_str, percent_encode_byte, utf8_percent_encode,
};

const URI_COMPONENT_ESCAPE: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'_')
    .remove(b'-')
    .remove(b'.')
    .remove(b'!')
    .remove(b'~')
    .remove(b'*')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')');

const URI_ESCAPE: &AsciiSet = &URI_COMPONENT_ESCAPE
    .remove(b';')
    .remove(b'/')
    .remove(b'?')
    .remove(b':')
    .remove(b'@')
    .remove(b'&')
    .remove(b'=')
    .remove(b'+')
    .remove(b'$')
    .remove(b',')
    .remove(b'#');

const PRESERVE_ESCAPE_SET: [u8; 11] = [
    b';', b'/', b'?', b':', b'@', b'&', b'=', b'+', b'$', b',', b'#',
];

pub fn encode_uri(input: &str) -> String {
    utf8_percent_encode(input, URI_ESCAPE).to_string()
}

pub fn encode_uri_component(input: &str) -> String {
    utf8_percent_encode(input, URI_COMPONENT_ESCAPE).to_string()
}

pub fn decode_uri(input: &str) -> Result<String, Utf8Error> {
    let mut bytes = Vec::new();
    for b in percent_decode_str(input) {
        if PRESERVE_ESCAPE_SET.contains(&b) {
            bytes.extend(percent_encode_byte(b).as_bytes());
        } else {
            bytes.push(b);
        }
    }
    std::str::from_utf8(&bytes).map(Into::into)
}

pub fn decode_uri_component(input: &str) -> Result<String, Utf8Error> {
    Ok(percent_decode_str(input).decode_utf8()?.to_string())
}
