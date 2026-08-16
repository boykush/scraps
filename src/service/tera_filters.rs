use chrono::format::StrftimeItems;
use chrono::DateTime;
use chrono_tz::Tz;
use tera::{Error, Kwargs, State, Tera, TeraResult, Value};

/// tera v1 shipped these filters through its default `builtins` feature, which
/// v2 dropped along with its chrono/serde_json dependencies. Both the built-in
/// templates and user-authored ones rely on them, so they are reimplemented
/// here to keep template behaviour unchanged across the upgrade.
pub fn register(tera: &mut Tera) {
    tera.register_filter("date", date);
    tera.register_filter("json_encode", json_encode);
    tera.register_filter("slice", slice);
    tera.register_filter("striptags", striptags);
}

fn date(value: Value, kwargs: Kwargs, _: &State) -> TeraResult<String> {
    let timestamp = value
        .as_i64()
        .ok_or_else(|| Error::message("Filter `date` expects a unix timestamp"))?;
    let datetime = DateTime::from_timestamp(timestamp, 0).ok_or_else(|| {
        Error::message(format!(
            "Filter `date` received an out of range timestamp: {timestamp}"
        ))
    })?;

    let format = kwargs.get::<&str>("format")?.unwrap_or("%Y-%m-%d");
    // Parsing up front keeps an invalid format an error rather than a panic
    // inside chrono's Display impl.
    let items = StrftimeItems::new(format).parse().map_err(|_| {
        Error::message(format!(
            "Filter `date` received an invalid format: {format}"
        ))
    })?;

    match kwargs.get::<&str>("timezone")? {
        Some(name) => {
            let timezone = name.parse::<Tz>().map_err(|_| {
                Error::message(format!(
                    "Filter `date` received an unknown timezone: {name}"
                ))
            })?;
            Ok(datetime
                .with_timezone(&timezone)
                .format_with_items(items.iter())
                .to_string())
        }
        None => Ok(datetime.format_with_items(items.iter()).to_string()),
    }
}

fn json_encode(value: Value, _: Kwargs, _: &State) -> TeraResult<String> {
    serde_json::to_string(&value)
        .map_err(|e| Error::message(format!("Filter `json_encode` failed: {e}")))
}

fn slice(value: &[Value], kwargs: Kwargs, _: &State) -> TeraResult<Vec<Value>> {
    let len = value.len() as i64;
    // Negative bounds count back from the end, as they did in tera v1.
    let resolve = |bound: i64| -> usize {
        if bound < 0 {
            (len + bound).clamp(0, len) as usize
        } else {
            bound.min(len) as usize
        }
    };

    let start = resolve(kwargs.get::<i64>("start")?.unwrap_or(0));
    let end = resolve(kwargs.get::<i64>("end")?.unwrap_or(len));
    if start >= end {
        return Ok(Vec::new());
    }
    let step = kwargs.get::<usize>("step")?.unwrap_or(1).max(1);

    Ok(value[start..end].iter().step_by(step).cloned().collect())
}

fn striptags(value: &str, _: Kwargs, _: &State) -> String {
    let mut stripped = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(open) = rest.find('<') {
        // An unclosed `<` is literal text rather than the start of a tag.
        let Some(close) = rest[open..].find('>') else {
            break;
        };
        stripped.push_str(&rest[..open]);
        rest = &rest[open + close + 1..];
    }
    stripped.push_str(rest);

    stripped
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn render(template: &str, context: &tera::Context) -> String {
        let mut tera = Tera::new();
        register(&mut tera);
        tera.add_raw_template("test.txt", template).unwrap();
        tera.render("test.txt", context).unwrap()
    }

    #[rstest]
    #[case("{{ ts | date }}", "2024-05-01")]
    #[case("{{ ts | date(format=\"%Y/%m/%d\") }}", "2024/05/01")]
    #[case("{{ ts | date(format=\"%Y-%m-%d %H:%M\") }}", "2024-05-01 12:30")]
    fn date_formats_a_unix_timestamp(#[case] template: &str, #[case] expected: &str) {
        let mut context = tera::Context::new();
        context.insert("ts", &1714566600i64); // 2024-05-01T12:30:00Z

        assert_eq!(render(template, &context), expected);
    }

    #[test]
    fn date_applies_the_timezone() {
        let mut context = tera::Context::new();
        context.insert("ts", &1714566600i64); // 2024-05-01T12:30:00Z
        context.insert("timezone", "Asia/Tokyo");

        // 12:30 UTC is 21:30 the same day in Tokyo.
        let template = "{{ ts | date(format=\"%Y-%m-%d %H:%M\", timezone=timezone) }}";
        assert_eq!(render(template, &context), "2024-05-01 21:30");
    }

    #[test]
    fn date_errors_on_an_unknown_timezone() {
        let mut tera = Tera::new();
        register(&mut tera);
        tera.add_raw_template("test.txt", "{{ ts | date(timezone=\"Mars/Olympus\") }}")
            .unwrap();

        let mut context = tera::Context::new();
        context.insert("ts", &1714566600i64);

        assert!(tera.render("test.txt", &context).is_err());
    }

    #[test]
    fn json_encode_quotes_and_escapes() {
        let mut context = tera::Context::new();
        context.insert("title", "a \"quoted\" title");

        assert_eq!(
            render("{{ title | json_encode() }}", &context),
            "\"a \\\"quoted\\\" title\""
        );
    }

    #[rstest]
    #[case("{{ items | slice(end=2) | join(sep=\",\") }}", "a,b")]
    #[case("{{ items | slice(start=1) | join(sep=\",\") }}", "b,c,d")]
    #[case("{{ items | slice(start=1, end=3) | join(sep=\",\") }}", "b,c")]
    #[case("{{ items | slice(end=-1) | join(sep=\",\") }}", "a,b,c")]
    #[case("{{ items | slice(step=2) | join(sep=\",\") }}", "a,c")]
    #[case("{{ items | slice(end=99) | join(sep=\",\") }}", "a,b,c,d")]
    #[case("{{ items | slice(start=3, end=1) | join(sep=\",\") }}", "")]
    fn slice_selects_a_sub_range(#[case] template: &str, #[case] expected: &str) {
        let mut context = tera::Context::new();
        context.insert("items", &["a", "b", "c", "d"]);

        assert_eq!(render(template, &context), expected);
    }

    #[rstest]
    #[case("<p>hello <b>world</b></p>", "hello world")]
    #[case("no tags here", "no tags here")]
    #[case("2 < 3, unclosed", "2 < 3, unclosed")]
    // Anything between a `<` and the next `>` goes, matching tera v1's `<[^>]*>`.
    #[case("a < b and c > d", "a  d")]
    #[case("<a href=\"https://example.com\">link</a>", "link")]
    fn striptags_removes_markup(#[case] input: &str, #[case] expected: &str) {
        let mut context = tera::Context::new();
        context.insert("html", input);

        assert_eq!(render("{{ html | striptags }}", &context), expected);
    }
}
