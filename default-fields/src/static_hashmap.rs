#![allow(non_upper_case_globals)]

use std::collections::HashMap;

use lazy_static::lazy_static;

// Just a demo. Should use <https://github.com/sile/amf>.
#[derive(Debug, PartialEq)]
pub enum AmfValue {
    String(String),
    Boolean(bool),
    Number(f64),
}

lazy_static! {
    static ref DEFAULT_flashVer: AmfValue = AmfValue::String("LNX 11,1,102,55".to_owned());
    static ref DEFAULT_fpad: AmfValue = AmfValue::Boolean(false);
    static ref DEFAULT_audioCodecs: AmfValue = AmfValue::Number(3575.0_f64);
    static ref DEFAULT_videoCodecs: AmfValue = AmfValue::Number(252.0_f64);
    static ref DEFAULT_videoFunction: AmfValue = AmfValue::Number(1.0_f64);
    static ref DEFAULT_objectEncoding: AmfValue = AmfValue::Number(0.0_f64);

    pub static ref CONNECTION_INFO: HashMap<&'static str, Option<&'static AmfValue>> = {
        let mut info = HashMap::new();

        info.insert("app", None);
        info.insert("flashVer", Some(&*DEFAULT_flashVer));
        info.insert("swfUrl", None);
        info.insert("tcUrl", None);
        info.insert("fpad", Some(&*DEFAULT_fpad));
        info.insert("audioCodecs", Some(&*DEFAULT_audioCodecs));
        info.insert("videoCodecs", Some(&*DEFAULT_videoCodecs));
        info.insert("videoFunction", Some(&*DEFAULT_videoFunction));
        info.insert("pageUrl", None);
        info.insert("objectEncoding", Some(&*DEFAULT_objectEncoding));

        info
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_hashmap() {
        let mut conn_info = CONNECTION_INFO.clone();
        let url = AmfValue::String("live.example.com".into());
        conn_info.insert("pageUrl", Some(&url));
        println!("===== static_hashmap =====");
        println!("{:#?}", conn_info);
        println!();
        assert_eq!(conn_info["pageUrl"], Some(&url));
    }
}
