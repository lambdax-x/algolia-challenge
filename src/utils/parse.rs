use std::str::FromStr;

use chrono::prelude::*;
use chrono::Duration;

// A parser for Things
// is a function from Strings
// to Lists of pairs
// of Things and Strings
// - Fritz Ruehr
//
// We just return a parsed T and the consumed string for convenience
// or the string in case of a failure / non consumption
type ParseResult<'a, T> = Result<(T, &'a str), &'a str>;

fn parse_char(c: char, input: &str) -> ParseResult<()> {
    match input.chars().next() {
        Some(e) if c == e =>  Ok(((), &input[1..])),
        _ => Err(input)
    }
}

fn parse_number<T: FromStr>(input: &str) -> ParseResult<T> {
    const RADIX: u32 = 10;
    let digits: String = input.chars().take_while(|c| c.is_digit(RADIX)).collect();
    let maybe_n = digits.parse::<T>();
    maybe_n.or(Err(input)).map(|n| (n, &input[digits.len()..]))
}

pub fn parse_string<'a>(expected: &str, input: &'a str) -> ParseResult<'a, ()> {
    let mut expected_iter = expected.chars();
    let mut input_iter = input.chars();

    loop {
        let (maybe_c1, maybe_c2) = (expected_iter.next(), input_iter.next());
        match (maybe_c1, maybe_c2) {
            (Some(c1), Some(c2)) if c1 == c2 => continue,
            (None, Some(_)) => return Ok(((), &input[expected.len()..])),
            _ => return Err(input)
        }
    }
}

/// Parse a time range [from ; to] given a string `year[-month[-day[ hour[:minutes]]]]`.
pub fn parse_time_range(input: &str) -> Option<(NaiveDateTime, NaiveDateTime)> {
    let mut maybe_year = None;
    let mut maybe_month = None;
    let mut maybe_day = None;
    let mut maybe_hour = None;
    let mut maybe_mins = None;

    // Parse the input in a monadic way
    // The result of this parsing *will* be Err
    // and should be Err("")
    let result: ParseResult<()> = parse_number(input).and_then(|(y, other)| { maybe_year = Some(y); parse_char('-', other) })
                                                     .and_then(|(_, other)| { parse_number::<u32>(other) })
                                                     .and_then(|(m, other)| { maybe_month = Some(m); parse_char('-', other) })
                                                     .and_then(|(_, other)| { parse_number::<u32>(other) })
                                                     .and_then(|(d, other)| { maybe_day = Some(d); parse_char(' ', other) })
                                                     .and_then(|(_, other)| { parse_number::<u32>(other) })
                                                     .and_then(|(h, other)| { maybe_hour = Some(h); parse_char(':', other) })
                                                     .and_then(|(_, other)| { parse_number::<u32>(other) })
                                                     .and_then(|(m, other)| { maybe_mins = Some(m); Err(other) });

    if result.is_ok() || !result.unwrap_err().is_empty() {
        return None;
    }

    // Build a tuple of Option<NaiveDateTime> which represent the bounds
    let (maybe_from, mut maybe_to) = match (maybe_year, maybe_month, maybe_day, maybe_hour, maybe_mins) {
        (Some(year), Some(month), Some(day), Some(hour), Some(mins)) => {
            let maybe_from = NaiveDate::from_ymd_opt(year, month, day).and_then(|naive_date| naive_date.and_hms_opt(hour, mins, 0));
            let maybe_to = maybe_from.and_then(|naive_date_time| naive_date_time.checked_add_signed(Duration::minutes(1)));
            (maybe_from, maybe_to)
        },
        (Some(year), Some(month), Some(day), Some(hour), _) => {
            let maybe_from = NaiveDate::from_ymd_opt(year, month, day).and_then(|naive_date| naive_date.and_hms_opt(hour, 0, 0));
            let maybe_to = maybe_from.and_then(|naive_date_time| naive_date_time.checked_add_signed(Duration::hours(1)));
            (maybe_from, maybe_to)
        },
        (Some(year), Some(month), Some(day), _, _) => {
            let maybe_from = NaiveDate::from_ymd_opt(year, month, day).and_then(|naive_date| naive_date.and_hms_opt(0, 0, 0));
            let maybe_to = maybe_from.and_then(|naive_date_time| naive_date_time.checked_add_signed(Duration::days(1)));
            (maybe_from, maybe_to)
        }
        (Some(year), Some(month), _, _, _) => {
            let maybe_from = NaiveDate::from_ymd_opt(year, month, 1).and_then(|naive_date| naive_date.and_hms_opt(0, 0, 0));
            let maybe_to = maybe_from.and_then(|naive_date_time| {
                match naive_date_time.month() {
                    12 => naive_date_time.with_year(naive_date_time.year() + 1).and_then(|date| date.with_month(1)),
                    month => naive_date_time.with_month(month + 1)
                }
            });
            (maybe_from, maybe_to)
        }
        (Some(year), _, _, _, _) => {
            let maybe_from = NaiveDate::from_ymd_opt(year, 1, 1).and_then(|naive_date| naive_date.and_hms_opt(0, 0, 0));
            let maybe_to = maybe_from.and_then(|naive_date_time| naive_date_time.with_year(naive_date_time.year() + 1));
            (maybe_from, maybe_to)
        }
        _ => (None, None)
    };

    maybe_to = maybe_to.and_then(|naive_date_time| naive_date_time.checked_sub_signed(Duration::seconds(1)));

    // Return the range iff both bounds are valid dates
    maybe_from.and_then(|from| {
        maybe_to.and_then(|to| {
            Some((from, to))
        })
    })
}

pub fn parse_count_param(input: &str) -> Option<usize> {
    let maybe_count = parse_string("size=", input).and_then(|(_, other)| parse_number::<usize>(other))
                                                   .ok();
    if maybe_count.is_some() && !maybe_count.unwrap().1.is_empty() {
        return None;
    }
    maybe_count.map(|(count, _)| count)
}
