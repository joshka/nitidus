//! Envelopes and messages for unit tests, built the way Himalaya's JSON decodes them.
//!
//! Dates are fixed in 2020, so what a test renders doesn't change with the clock: list dates show
//! as `2020-…`, in the "Older" band.

use serde_json::{Value, json};

use super::{Envelope, Mailbox, Message};

/// An envelope from `sender` about `subject`, sent on `day` of January 2020, with IANA `flags`.
pub fn envelope(id: &str, sender: &str, subject: &str, day: u32, flags: &[&str]) -> Envelope {
    let flags: Vec<Value> = flags
        .iter()
        .map(|flag| json!({ "raw": flag, "iana": flag }))
        .collect();
    let json = json!({
        "id": id,
        "subject": subject,
        "from": [{ "name": sender, "email": format!("{}@x.test", sender.to_lowercase()) }],
        "date": format!("2020-01-{day:02}T09:30:00Z"),
        "flags": flags,
        "messageId": id,
    });
    serde_json::from_value(json).expect("a fixture envelope")
}

/// A message with `text` as its body, from Ada to Rae, undated so it renders the same anywhere.
pub fn message(id: &str, subject: &str, text: &str) -> Message {
    let json = json!({
        "id": id,
        "headers": {
            "from": [{ "name": "Ada Lovelace", "email": "ada@x.test" }],
            "to": [{ "name": "Rae Field", "email": "rae@x.test" }],
            "subject": subject,
        },
        "text": text,
        "parts": [{ "filename": "notes.pdf" }],
    });
    serde_json::from_value(json).expect("a fixture message")
}

/// A mailbox named `name` with Himalaya role `role`.
pub fn mailbox(name: &str, role: Option<&str>) -> Mailbox {
    serde_json::from_value(json!({ "id": name, "name": name, "role": role }))
        .expect("a fixture mailbox")
}
