//! Reading beyond the text: attachments, links, the HTML in a browser, and the raw source.

use std::path::Path;

use super::App;
use super::action::ReadAction;
use super::overlay::{Overlay, PickFor};
use crate::system;
use crate::ui::overlay::Picker;
use crate::worker::{AfterDownload, Download, Request};

impl App {
    /// Performs a reading action on the selected message.
    pub(super) fn read(&mut self, action: ReadAction) {
        match action {
            ReadAction::OpenAttachments => self.download(AfterDownload::Open),
            ReadAction::SaveAttachments => self.download(AfterDownload::Report),
            ReadAction::Links => self.pick_link(),
            ReadAction::Browser => self.open_html(),
            ReadAction::Source => self.toggle_source(),
        }
    }

    /// Writes the attachments to a scratch directory to open them, or to downloads to keep them.
    fn download(&mut self, then: AfterDownload) {
        let Some(message) = self.current_message() else {
            return;
        };
        let dir = match then {
            AfterDownload::Open => system::scratch_dir(&message.id),
            AfterDownload::Report => Ok(system::downloads_dir()),
        };
        match dir {
            Ok(dir) => self.request(
                "Fetching attachments",
                Request::Download(Download { message, dir, then }),
            ),
            Err(error) => self.error(format!("{error:#}")),
        }
    }

    /// Attachments were written: open one (picking when there are several), or say where.
    pub(super) fn downloaded(&mut self, paths: Vec<String>, then: AfterDownload) {
        match (then, paths.as_slice()) {
            (_, []) => self.error("No attachments"),
            (AfterDownload::Report, [first, ..]) => self.report_saved(paths.len(), first),
            (AfterDownload::Open, [only]) => self.open_target(only),
            (AfterDownload::Open, _) => self.pick_file(paths),
        }
    }

    /// Says how many attachments were saved, and where.
    fn report_saved(&mut self, count: usize, first: &str) {
        let dir = Path::new(first)
            .parent()
            .map_or_default(|dir| dir.display().to_string());
        self.info(format!("Saved {count} attachment(s) to {dir}"));
    }

    /// Asks which downloaded file to open.
    fn pick_file(&mut self, paths: Vec<String>) {
        let name = |path: &String| {
            Path::new(path)
                .file_name()
                .map_or_else(|| path.clone(), |name| name.to_string_lossy().into_owned())
        };
        let items = paths.into_iter().map(|path| (name(&path), path)).collect();
        self.overlay = Some(Overlay::Picker(
            Picker::new("Open attachment", items),
            PickFor::File,
        ));
    }

    /// Asks which of the message's links to open.
    fn pick_link(&mut self) {
        let links = self
            .reader
            .message()
            .map_or_default(crate::mail::Message::links);
        if links.is_empty() {
            self.error("No links in this message");
            return;
        }
        let items = links.into_iter().map(|link| (link.clone(), link)).collect();
        self.overlay = Some(Overlay::Picker(
            Picker::new("Open link", items),
            PickFor::Link,
        ));
    }

    /// Writes the HTML part to a file and opens it in the browser.
    fn open_html(&mut self) {
        let html = self
            .reader
            .message()
            .and_then(|message| Some((message.id.clone(), message.html.clone()?)));
        let Some((id, html)) = html else {
            self.error("No HTML part");
            return;
        };
        let path = std::env::temp_dir().join(format!("nitidus-{id}.html"));
        match std::fs::write(&path, html) {
            Ok(()) => self.open_target(&path.display().to_string()),
            Err(error) => self.error(format!("write {}: {error}", path.display())),
        }
    }

    /// Shows or hides the raw source, reading it the first time.
    fn toggle_source(&mut self) {
        let needs_reading = self.reader.toggle_source();
        if let Some(message) = self.current_message().filter(|_| needs_reading) {
            self.request("Reading source", Request::Source(message));
        }
    }

    /// Opens a file or URL with the system's opener.
    pub(super) fn open_target(&mut self, target: &str) {
        match system::open(target) {
            Ok(()) => self.info(format!("Opened {target}")),
            Err(error) => self.error(format!("{error:#}")),
        }
    }
}
