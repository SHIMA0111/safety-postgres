use itertools::Itertools;

const DATE_SEPARATORS: [&str; 2] = ["-", "/"];
const DATE_FORMATS: [[&str; 3]; 6] = [
    ["%Y", "%m", "%d"], ["%Y", "%b", "%d"], ["%Y", "%B", "%d"],
    ["%Y", "%m", "%e"], ["%Y", "%b", "%e"], ["%Y", "%B", "%e"],
];
const TIME_FORMATS: [&str; 18] = [
    "%H:%M", "%k:%M", "%I:%M", "%l:%M",
    "%I:%M %p", "%I:%M %P", "%l:%M %p", "%l:%M %P",
    "%H:%M:%S", "%k:%M:%S", "%I:%M:%S", "%i:%M:%S",
    "%H:%M:%S.%f",  "%k:%M:%S.%f",
    "%I:%M:%S %p", "%I:%M:%S %P", "%i:%M:%S %p", "%i:%M:%S %P",
];

const SPECIAL_DATE_FORMATS: [&str; 8] = [
    "%B %e, %Y", "%B %d, %Y", "%b %e, %Y", "%b %d, %Y",
    "%B %e, %y", "%B %d, %y", "%b %e, %y", "%b %d, %y",
];
const UNSUPPORTED_DATE_FORMATS: [[&str; 3]; 12] = [
    ["%y", "%m", "%d"], ["%y", "%b", "%d"], ["%y", "%B", "%d"],
    ["%y", "%m", "%e"], ["%y", "%b", "%e"], ["%y", "%B", "%e"],
    ["%C", "%m", "%d"], ["%C", "%b", "%d"], ["%C", "%B", "%d"],
    ["%C", "%m", "%e"], ["%C", "%b", "%e"], ["%C", "%B", "%e"],
];
const TIMEZONE_SPEC: [&str; 5] = [
    "%Z", "%z", "%:z", "%::z", "%:::z"
];

fn date_format_generator(formats: &[[&str; 3]], special_formats: &[&str], separators: &[&str]) -> Vec<String> {
    let mut generated_formats = Vec::new();
    for format_list in formats {
        for (format, sep) in format_list.iter().permutations(format_list.len()).cartesian_product(separators) {
            let format = format.iter().map(|word| word.to_string()).join(sep);
            generated_formats.push(format);
        }
    }
    for sp_format in special_formats {
        generated_formats.push(sp_format.to_string());
    }
    generated_formats
}

pub(super) fn support_date_formats() -> Vec<String> {
    date_format_generator(&DATE_FORMATS, &SPECIAL_DATE_FORMATS, &DATE_SEPARATORS)
}

pub(super) fn unsupported_date_formats() -> Vec<String> {
    date_format_generator(&UNSUPPORTED_DATE_FORMATS, &vec![], &DATE_SEPARATORS)
}

pub(super) fn support_time_formats() -> Vec<String> {
    TIME_FORMATS.iter().map(|format| format.to_string()).collect()
}

pub(super) fn unsupported_time_formats() -> Vec<String> {
    let mut unsupported_formats = Vec::new();

    for time_format in support_time_formats() {
        for timezone_format in TIMEZONE_SPEC {
            unsupported_formats.push(format!("{}{}", time_format, timezone_format));
            unsupported_formats.push(format!("{} {}", time_format, timezone_format));
        }
    }
    unsupported_formats
}

pub(super) fn support_datetime_formats() -> Vec<String> {

    let date_formats = support_date_formats();
    let time_formats = support_time_formats();

    let datetime_formats = date_formats
        .iter()
        .cartesian_product(time_formats.iter())
        .flat_map(|(date, time)| vec![format!("{} {}", date, time), format!("{}T{}", date, time)])
        .collect();
    datetime_formats
}

pub(super) fn timezone_datetime_formats() -> Vec<String> {
    support_date_formats()
        .iter()
        .cartesian_product(unsupported_time_formats())
        .flat_map(|(date, time_with_zone)| vec![format!("{} {}", date, time_with_zone), format!("{}T{}", date, time_with_zone)])
        .collect::<Vec<String>>()
}

pub(super) fn ambiguous_datetime_formats() -> Vec<String> {
    unsupported_date_formats()
        .iter()
        .cartesian_product(support_time_formats())
        .flat_map(|(ambiguous_date, time)| vec![format!("{} {}", ambiguous_date, time), format!("{}T{}", ambiguous_date, time)])
        .collect::<Vec<String>>()
}

pub(super) fn unsupported_datetime_formats() -> Vec<String> {
    unsupported_date_formats()
        .iter()
        .cartesian_product(unsupported_time_formats())
        .flat_map(|(ambiguous_date, time_with_timezone)| vec![format!("{} {}", ambiguous_date, time_with_timezone), format!("{}T{}", ambiguous_date, time_with_timezone)])
        .collect::<Vec<String>>()
}
