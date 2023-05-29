use std::collections::HashMap;

use crate::libs::markdown::to_html;
use tera::{to_value, try_get_value, Value};

pub fn to_html_text(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = try_get_value!("upper", "value", String, value);

    Ok(to_value(to_html(&s)).unwrap())
}
