use chrono::NaiveDateTime;
use std::borrow::Cow;

#[derive(Debug)]
struct Comment<'a> {
    id: u64,
    image: Cow<'a, str>,
    text: Cow<'a, str>,
    date: NaiveDateTime,
    user: u32,
    pointat: u64
}