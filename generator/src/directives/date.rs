use icu::{
    calendar::{Date, Iso},
    datetime::{DateTimeFormatter, fieldsets::YMD},
    locale::Locale,
};

pub fn format_iso(date: &Date<Iso>) -> String {
    let year = date.era_year().year;
    let month = date.month().ordinal;
    let day = date.day_of_month().0;

    format!("{:04}-{:02}-{:02}", year, month, day)
}

pub fn format_display(date: &Date<Iso>, locale_str: &str) -> String {
    let locale = Locale::try_from_str(locale_str).expect("Invalid locale string.");

    let dtf = DateTimeFormatter::try_new(locale.into(), YMD::long())
        .expect("Failed to initialize formatter");

    dtf.format(date).to_string()
}
