use chrono::NaiveDateTime;
use std::borrow::Cow;

#[derive(Debug)]
struct Post<'a> {
    id: u64,
    image: Cow<'a, str>,
    text: Cow<'a, str>,
    date: NaiveDateTime,
    user: u32,
    title: Cow<'a, str>,
}