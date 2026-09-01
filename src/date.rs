//! A minimal civil date, and the arithmetic a month grid needs.
//!
//! `src/elements/calendar.rs` needs leap years, month lengths, the weekday of a
//! date, and add-days / add-months. That is the whole of the requirement, and
//! it is one page of code — so this crate does not take `chrono`, `time` or
//! `jiff` into its **public API**, where the choice would become a consumer's
//! problem rather than this crate's. A consumer that already has one of those
//! converts three integers at the boundary.
//!
//! The arithmetic is Howard Hinnant's `days_from_civil` / `civil_from_days`
//! (the "chrono-Compatible Low-Level Date Algorithms" note), which is exact for
//! every year in `i32` and has one leap-year rule in it rather than one per
//! operation. `tests::round_trips_every_day_from_1800_to_2200` walks every day
//! in four centuries rather than sampling.
//!
//! There is deliberately no time, no zone and no clock: [`Date::today`] does
//! not exist. A calendar is given its `today` by the caller, because a UI
//! toolkit reading the system clock is a UI toolkit that cannot be tested and
//! cannot be told about the user's zone.

use std::fmt;

/// Day of the week.
///
/// `Sunday` is 0 because that is the phase `from_days` falls out with, not
/// because it is the week's first day anywhere in particular — which day starts
/// the week is [`Weekday::days_from`]'s argument, and the calendar's
/// `first_day_of_week`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Weekday {
    /// Sunday.
    Sunday,
    /// Monday.
    Monday,
    /// Tuesday.
    Tuesday,
    /// Wednesday.
    Wednesday,
    /// Thursday.
    Thursday,
    /// Friday.
    Friday,
    /// Saturday.
    Saturday,
}

impl Weekday {
    /// Every weekday, Sunday first.
    pub const ALL: [Weekday; 7] = [
        Weekday::Sunday,
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
    ];

    /// This weekday as an index from Sunday.
    pub fn index(self) -> usize {
        Weekday::ALL
            .iter()
            .position(|day| *day == self)
            .unwrap_or(0)
    }

    /// How many days this weekday is *after* `start`, wrapping.
    ///
    /// This is the whole of "which column does a date sit in": a grid whose
    /// first column is `start` puts a date at `date.weekday().days_from(start)`.
    /// So the calendar needs no first-day-of-week table.
    pub fn days_from(self, start: Weekday) -> usize {
        (7 + self.index() - start.index()) % 7
    }

    /// The seven weekdays in order, starting at `self`.
    pub fn week_from(self) -> [Weekday; 7] {
        let mut week = [Weekday::Sunday; 7];
        for (offset, slot) in week.iter_mut().enumerate() {
            *slot = Weekday::ALL[(self.index() + offset) % 7];
        }
        week
    }

    /// The English name, which is what a localisation parameter falls back to.
    pub fn name(self) -> &'static str {
        match self {
            Weekday::Sunday => "Sunday",
            Weekday::Monday => "Monday",
            Weekday::Tuesday => "Tuesday",
            Weekday::Wednesday => "Wednesday",
            Weekday::Thursday => "Thursday",
            Weekday::Friday => "Friday",
            Weekday::Saturday => "Saturday",
        }
    }

    /// The three-letter English abbreviation.
    pub fn short_name(self) -> &'static str {
        &self.name()[..3]
    }

    /// The two-letter English abbreviation used in a grid heading.
    ///
    /// Two letters and not one on purpose: one letter makes Tuesday and
    /// Thursday, and Saturday and Sunday, indistinguishable.
    pub fn min_name(self) -> &'static str {
        &self.name()[..2]
    }
}

/// The English month names, indexed from January.
const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// The English name of a month, `1..=12`. Out of range returns `""`.
pub fn month_name(month: u32) -> &'static str {
    MONTH_NAMES.get(month as usize - 1).copied().unwrap_or("")
}

/// Whether `year` is a leap year in the proleptic Gregorian calendar.
///
/// The one place the rule is written. Everything else goes through
/// [`Date::to_days`] / [`Date::from_days`], which carry it implicitly.
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// How many days are in `month` of `year`. `month` is `1..=12`.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// A civil date: a year, a month and a day that exists.
///
/// The fields are private and [`Date::new`] validates, so a `Date` in hand is a
/// day that exists — the grid never has to ask. `Ord` is derived and is
/// chronological because the field order is year, month, day.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: i32,
    month: u32,
    day: u32,
}

impl Date {
    /// A date, or `None` when there is no such day.
    pub fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// The year.
    pub fn year(self) -> i32 {
        self.year
    }

    /// The month, `1..=12`.
    pub fn month(self) -> u32 {
        self.month
    }

    /// The day of the month, `1..=31`.
    pub fn day(self) -> u32 {
        self.day
    }

    /// Days since 1970-01-01, negative before it.
    ///
    /// Hinnant's `days_from_civil`. The shift to a March-based year is what
    /// puts the leap day at the end of the era, which is why there is no
    /// leap-year branch here at all.
    pub fn to_days(self) -> i64 {
        let year = self.year as i64 - i64::from(self.month <= 2);
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let month = self.month as i64;
        let day_of_year =
            (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + self.day as i64 - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    /// The date `days` days after 1970-01-01. Hinnant's `civil_from_days`.
    pub fn from_days(days: i64) -> Self {
        let days = days + 719_468;
        let era = days.div_euclid(146_097);
        let day_of_era = days - era * 146_097;
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let year = year_of_era + era * 400;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_prime = (5 * day_of_year + 2) / 153;
        let day = (day_of_year - (153 * month_prime + 2) / 5 + 1) as u32;
        let month = (month_prime + if month_prime < 10 { 3 } else { -9 }) as u32;
        Self {
            year: (year + i64::from(month <= 2)) as i32,
            month,
            day,
        }
    }

    /// Which day of the week this date falls on.
    pub fn weekday(self) -> Weekday {
        // 1970-01-01 was a Thursday, so the epoch's phase is 4 from Sunday.
        let index = (self.to_days() + 4).rem_euclid(7) as usize;
        Weekday::ALL[index]
    }

    /// This date, `days` days later. Negative goes back.
    pub fn add_days(self, days: i64) -> Self {
        Self::from_days(self.to_days() + days)
    }

    /// This date, `months` months later, **clamped** to the last day of the
    /// month it lands in.
    ///
    /// Jan 31 plus one month is Feb 28, and that does not round-trip back to
    /// Jan 31. That is the right answer for a month button and the wrong one
    /// for re-deriving a day, which is why `Calendar` keeps the focused date
    /// and re-derives rather than paging it back and forth.
    pub fn add_months(self, months: i32) -> Self {
        let total = self.year as i64 * 12 + (self.month as i64 - 1) + months as i64;
        // `div_euclid` / `rem_euclid`, not `/` and `%`: a date before year 0
        // makes `total` negative, and a signed remainder would put it in month
        // zero or below.
        let year = total.div_euclid(12) as i32;
        let month = total.rem_euclid(12) as u32 + 1;
        let day = self.day.min(days_in_month(year, month));
        Self { year, month, day }
    }

    /// Whether two dates are in the same month of the same year.
    pub fn is_same_month(self, other: Date) -> bool {
        self.year == other.year && self.month == other.month
    }

    /// The first day of this date's month.
    pub fn first_of_month(self) -> Self {
        Self {
            year: self.year,
            month: self.month,
            day: 1,
        }
    }

    /// The English name of this date's month.
    pub fn month_name(self) -> &'static str {
        month_name(self.month)
    }
}

impl fmt::Display for Date {
    /// ISO 8601, which is what a day cell's accessible name is built from.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> Date {
        Date::new(year, month, day).expect("a date the test wrote by hand exists")
    }

    #[test]
    fn a_day_that_does_not_exist_is_not_a_date() {
        assert!(Date::new(2026, 2, 29).is_none());
        assert!(Date::new(2024, 2, 29).is_some());
        assert!(Date::new(2026, 13, 1).is_none());
        assert!(Date::new(2026, 0, 1).is_none());
        assert!(Date::new(2026, 4, 31).is_none());
        assert!(Date::new(2026, 1, 0).is_none());
    }

    #[test]
    fn leap_years_follow_the_gregorian_rule() {
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2026));
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(1900, 2), 28);
    }

    #[test]
    fn the_epoch_is_a_thursday() {
        assert_eq!(date(1970, 1, 1).to_days(), 0);
        assert_eq!(date(1970, 1, 1).weekday(), Weekday::Thursday);
    }

    #[test]
    fn known_weekdays() {
        assert_eq!(date(2026, 8, 20).weekday(), Weekday::Thursday);
        assert_eq!(date(2000, 1, 1).weekday(), Weekday::Saturday);
        assert_eq!(date(1900, 1, 1).weekday(), Weekday::Monday);
    }

    #[test]
    fn round_trips_every_day_from_1800_to_2200() {
        let start = date(1800, 1, 1).to_days();
        let end = date(2200, 12, 31).to_days();
        let mut expected = date(1800, 1, 1);

        for days in start..=end {
            let actual = Date::from_days(days);
            assert_eq!(actual, expected, "day {days} disagrees with the walk");
            assert_eq!(actual.to_days(), days);
            expected = expected.add_days(1);
        }
    }

    #[test]
    fn days_before_the_epoch_are_negative() {
        assert_eq!(date(1969, 12, 31).to_days(), -1);
        assert_eq!(Date::from_days(-1), date(1969, 12, 31));
    }

    #[test]
    fn add_months_clamps_and_does_not_round_trip() {
        assert_eq!(date(2026, 1, 31).add_months(1), date(2026, 2, 28));
        assert_eq!(
            date(2026, 1, 31).add_months(1).add_months(-1),
            date(2026, 1, 28)
        );
        assert_eq!(date(2024, 1, 31).add_months(1), date(2024, 2, 29));
        assert_eq!(date(2026, 12, 15).add_months(1), date(2027, 1, 15));
    }

    #[test]
    fn add_months_across_year_zero_uses_euclidean_division() {
        assert_eq!(date(1, 1, 15).add_months(-13), date(-1, 12, 15));
        assert_eq!(date(-1, 12, 15).add_months(13), date(1, 1, 15));
    }

    #[test]
    fn a_week_starts_where_it_is_told_to() {
        assert_eq!(Weekday::Sunday.days_from(Weekday::Sunday), 0);
        assert_eq!(Weekday::Sunday.days_from(Weekday::Monday), 6);
        assert_eq!(Weekday::Thursday.days_from(Weekday::Monday), 3);
        assert_eq!(
            Weekday::Monday.week_from(),
            [
                Weekday::Monday,
                Weekday::Tuesday,
                Weekday::Wednesday,
                Weekday::Thursday,
                Weekday::Friday,
                Weekday::Saturday,
                Weekday::Sunday,
            ]
        );
    }

    #[test]
    fn names_are_the_english_defaults() {
        assert_eq!(month_name(8), "August");
        assert_eq!(month_name(12), "December");
        assert_eq!(Weekday::Wednesday.short_name(), "Wed");
        assert_eq!(Weekday::Saturday.min_name(), "Sa");
        assert_eq!(Weekday::Sunday.min_name(), "Su");
        assert_eq!(date(2026, 8, 20).to_string(), "2026-08-20");
    }

    #[test]
    fn ordering_is_chronological() {
        assert!(date(2026, 1, 31) < date(2026, 2, 1));
        assert!(date(2025, 12, 31) < date(2026, 1, 1));
        assert_eq!(date(2026, 8, 1), date(2026, 8, 20).first_of_month());
        assert!(date(2026, 8, 1).is_same_month(date(2026, 8, 31)));
        assert!(!date(2026, 8, 1).is_same_month(date(2027, 8, 1)));
    }
}
