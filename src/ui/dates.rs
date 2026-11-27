//! Dates as the list and reader show them.

use chrono::{DateTime, Datelike, FixedOffset, Local};

/// How old a message is, as the list's date bands group it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    /// Sent today.
    Today,

    /// Sent yesterday.
    Yesterday,

    /// Sent in the last week.
    ThisWeek,

    /// Sent in the last month.
    ThisMonth,

    /// Anything older.
    Older,

    /// No usable date.
    Undated,
}

impl Section {
    /// The section a message sent at `date` falls in.
    pub fn of(date: Option<DateTime<FixedOffset>>) -> Self {
        let Some(date) = date else {
            return Self::Undated;
        };
        let days = (Local::now().date_naive() - date.with_timezone(&Local).date_naive()).num_days();
        match days {
            ..=0 => Self::Today,
            1 => Self::Yesterday,
            2..=6 => Self::ThisWeek,
            7..=30 => Self::ThisMonth,
            _ => Self::Older,
        }
    }

    /// The band's label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Today => "Today",
            Self::Yesterday => "Yesterday",
            Self::ThisWeek => "This week",
            Self::ThisMonth => "This month",
            Self::Older => "Older",
            Self::Undated => "Undated",
        }
    }
}

/// A list date: the time today, the weekday this week, the day this year, else the full date.
pub fn short(date: Option<DateTime<FixedOffset>>) -> String {
    let Some(date) = date.map(|date| date.with_timezone(&Local)) else {
        return String::new();
    };
    let now = Local::now();
    let format = if date.date_naive() == now.date_naive() {
        "%H:%M"
    } else if now.signed_duration_since(date).num_days() < 6 {
        "%a %H:%M"
    } else if date.year() == now.year() {
        "%b %e"
    } else {
        "%Y-%m-%d"
    };
    date.format(format).to_string()
}

/// A header date as written (RFC 3339 or 2822) in local time, or as written when unparsable.
pub fn long(raw: &str) -> String {
    let parsed = DateTime::parse_from_rfc3339(raw).or_else(|_| DateTime::parse_from_rfc2822(raw));
    let local = |date: DateTime<FixedOffset>| {
        date.with_timezone(&Local)
            .format("%a, %e %b %Y  %H:%M")
            .to_string()
    };
    parsed.map_or_else(|_| raw.to_owned(), local)
}
