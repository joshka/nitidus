//! Writing mail: the editor file a draft round-trips through.
//!
//! The file is plain headers, a blank line, then the body, so any editor
//! works. `Attach:` lines name files to attach.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::mail::{Address, ComposeKind, Draft, Template};

impl Draft {
    pub fn from_template(kind: ComposeKind, t: Template) -> Self {
        let list = |a: &[Address]| a.iter().map(ToString::to_string).collect();
        Draft {
            kind,
            from: t.from.as_ref().map(ToString::to_string),
            to: list(&t.to),
            cc: list(&t.cc),
            bcc: list(&t.bcc),
            subject: t.subject.unwrap_or_default(),
            body: t.body,
            attach: vec![],
            carried: t
                .attachments
                .into_iter()
                .filter_map(|a| a.filename)
                .collect(),
        }
    }

    pub fn to_file(&self) -> String {
        let mut out = String::new();
        if let Some(from) = &self.from {
            out += &format!("From: {from}\n");
        }
        out += &format!("To: {}\n", self.to.join(", "));
        out += &format!("Cc: {}\n", self.cc.join(", "));
        out += &format!("Bcc: {}\n", self.bcc.join(", "));
        out += &format!("Subject: {}\n", self.subject);
        for path in &self.attach {
            out += &format!("Attach: {}\n", path.display());
        }
        out += "\n";
        out += &self.body;
        if !self.body.ends_with('\n') {
            out.push('\n');
        }
        out
    }

    pub fn update_from_file(&mut self, text: &str) -> Result<()> {
        let (head, body) = text.split_once("\n\n").unwrap_or((text, ""));
        let (mut to, mut cc, mut bcc, mut attach) = (vec![], vec![], vec![], vec![]);
        let mut subject = String::new();
        let mut from = None;
        for line in head.lines() {
            let Some((key, value)) = line.split_once(':') else {
                bail!("not a header line: {line:?}");
            };
            let value = value.trim();
            match key.trim().to_ascii_lowercase().as_str() {
                "from" => from = (!value.is_empty()).then(|| value.to_string()),
                "to" => to.extend(split_addresses(value)),
                "cc" => cc.extend(split_addresses(value)),
                "bcc" => bcc.extend(split_addresses(value)),
                "subject" => subject = value.to_string(),
                "attach" if !value.is_empty() => attach.push(expand(value)),
                "attach" => {}
                other => bail!("unknown header {other:?}"),
            }
        }
        for path in &attach {
            if !path.is_file() {
                bail!("attachment not found: {}", path.display());
            }
        }
        (self.from, self.to, self.cc, self.bcc) = (from, to, cc, bcc);
        self.subject = subject;
        self.attach = attach;
        self.body = body.to_string();
        Ok(())
    }

    /// What blocks sending, if anything.
    pub fn problem(&self) -> Option<&'static str> {
        if self.to.is_empty() && self.cc.is_empty() && self.bcc.is_empty() {
            Some("no recipients")
        } else {
            None
        }
    }
}

fn expand(path: &str) -> PathBuf {
    match (path.strip_prefix("~/"), std::env::var_os("HOME")) {
        (Some(rest), Some(home)) => Path::new(&home).join(rest),
        _ => PathBuf::from(path),
    }
}

/// Splits an address list on commas outside quotes and angle brackets.
pub fn split_addresses(list: &str) -> Vec<String> {
    let (mut out, mut cur, mut quoted, mut angle) = (vec![], String::new(), false, false);
    for c in list.chars() {
        match c {
            '"' => quoted = !quoted,
            '<' if !quoted => angle = true,
            '>' if !quoted => angle = false,
            ',' if !quoted && !angle => {
                out.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(c);
    }
    out.push(cur);
    out.into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// The user's editor: `$VISUAL`, then `$EDITOR`, then `vi`.
pub fn edit(path: &Path) -> Result<()> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".into());
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("{editor} \"$1\""))
        .arg("nitidus-editor")
        .arg(path)
        .status()
        .with_context(|| format!("run editor {editor:?}"))?;
    if !status.success() {
        bail!("editor exited with {status}");
    }
    Ok(())
}

/// Opens a file or URL with the system's opener.
pub fn open(target: &str) -> Result<()> {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    Command::new(opener)
        .arg(target)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .with_context(|| format!("run {opener}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses_split_outside_quotes() {
        assert_eq!(
            split_addresses(r#""Hopper, Grace" <g@x.test>, ada@x.test,"#),
            vec![r#""Hopper, Grace" <g@x.test>"#, "ada@x.test"]
        );
    }

    #[test]
    fn draft_round_trips_through_the_file() {
        let mut d = Draft::from_template(ComposeKind::New, Template::default());
        d.to = vec!["a@x.test".into()];
        d.subject = "Hi".into();
        d.body = "Body\n\nmore\n".into();
        let mut e = d.clone();
        e.update_from_file(&d.to_file()).unwrap();
        assert_eq!(e.to, d.to);
        assert_eq!(e.subject, "Hi");
        assert_eq!(e.body, d.body);
        assert!(e.update_from_file("X-Bogus: 1\n\nbody").is_err());
    }
}
