use api;
use error::*;
use futures::{Async, Stream};
use serde_json;
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
pub enum Endpoint {
    User,
    Notification,
    Federated,
    Local,
    Hashtag(String),
    LocalHashtag(String),
    Other(String),
}

impl Endpoint {
    pub fn as_path<'a>(&self) -> Cow<'a, str> {
        use url::percent_encoding::{QUERY_ENCODE_SET, utf8_percent_encode};
        use self::Endpoint::*;

        match *self {
            User => "/api/v1/streaming/user".into(),
            Notification => "/api/v1/streaming/user/notification".into(),
            Federated => "/api/v1/streaming/public".into(),
            Local => "/api/v1/streaming/public/local".into(),

            Hashtag(ref tag) => {
                let encoded_tag = utf8_percent_encode(tag, QUERY_ENCODE_SET);
                format!("/api/v1/streaming/hashtag?tag={}", encoded_tag).into()
            }
            LocalHashtag(ref tag) => {
                let encoded_tag = utf8_percent_encode(tag, QUERY_ENCODE_SET);
                format!("/api/v1/streaming/hashtag/local?tag={}", encoded_tag).into()
            }

            Other(ref path) => path.clone().into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum EventType {
    Update,
    Notification,
    Delete,
}

#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Timeline<S> {
    lines: S,
    waiting_for: Option<EventType>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Event {
    Update(Box<api::v1::Status>),
    Notification(Box<api::v1::Notification>),
    Delete(api::v1::StatusId),
    Heartbeat,
}

impl<S> Timeline<S>
where
    S: Stream<Item = String, Error = Error>,
{
    pub fn from_lines(lines: S) -> Timeline<S> {
        Timeline {
            lines,
            waiting_for: None,
        }
    }
}

impl<S> Stream for Timeline<S>
where
    S: Stream<Item = String, Error = Error>,
{
    type Item = Event;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>> {
        use self::EventType::*;

        loop {
            if let Some(line) = try_ready!(self.lines.poll()) {
                if line.starts_with(':') {
                    return Ok(Async::Ready(Some(Event::Heartbeat)));
                }

                if let Some(event_type) = self.waiting_for {
                    if line.starts_with("data: ") {
                        let data = &line[6..];

                        self.waiting_for = None;

                        match event_type {
                            Update => {
                                return serde_json::from_str(data).chain_err(|| {
                                    ErrorKind::Deserialize(data.to_string())
                                }).map(|status| {
                                    Async::Ready(Some(Event::Update(Box::new(status))))
                                })
                            }
                            Notification => {
                                return serde_json::from_str(data)
                                    .chain_err(|| ErrorKind::Deserialize(data.to_string()))
                                    .map(|notification| {
                                        Async::Ready(
                                            Some(Event::Notification(Box::new(notification))),
                                        )
                                    })
                            }
                            Delete => {
                                return data.parse::<api::v1::StatusId>()
                                    .chain_err(|| ErrorKind::StatusId(data.to_string()))
                                    .map(|id| Async::Ready(Some(Event::Delete(id))))
                            }
                        }
                    } else {
                        // We're in an unexpected state, reset to be safe
                        self.waiting_for = None;
                        bail!(ErrorKind::StreamingState("data", line));
                    }
                } else if line.starts_with("event: ") {
                    let event_type = match &line[7..] {
                        "update" => Update,
                        "delete" => Delete,
                        "notification" => Notification,
                        other => {
                            bail!(ErrorKind::EventType(other.to_string()));
                        }
                    };

                    self.waiting_for = Some(event_type);
                } else if !line.is_empty() {
                    bail!(ErrorKind::StreamingState("event", line));
                }
            } else {
                return Ok(Async::Ready(None));
            }
        }
    }
}

#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Lines<S> {
    stream: S,
    buffer: Vec<u8>,
}

impl<S, B> Lines<S>
where
    B: AsRef<[u8]>,
    S: Stream<Item = B, Error = Error>,
{
    pub fn new(stream: S) -> Self {
        Lines {
            stream,
            buffer: Vec::new(),
        }
    }
}

impl<S, B> Stream for Lines<S>
where
    B: AsRef<[u8]>,
    S: Stream<Item = B, Error = Error>,
{
    type Item = String;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>> {
        loop {
            if let Some(index) = self.buffer.iter().position(|c| *c == b'\n') {
                // Buffer contains a newline, split the buffer and return
                let mut split = self.buffer.split_off(index + 1);
                ::std::mem::swap(&mut self.buffer, &mut split);
                split.pop(); // Remove trailing newline

                return String::from_utf8(split)
                    .map(|line| Async::Ready(Some(line)))
                    .chain_err(|| ErrorKind::Utf8);
            } else if let Some(chunk) = try_ready!(self.stream.poll()) {
                // No newline in current chunk, attempt to fill the buffer
                self.buffer.extend_from_slice(&chunk.as_ref());
            } else {
                // Underlying stream is finished
                return Ok(Async::Ready(None));
            }
        }
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod test {
    use super::*;
    use futures::{self, Future, Stream};
    use futures::unsync::mpsc;

    #[test]
    fn endpoint_encoding() {
        let path = Endpoint::Hashtag("こんにちは".into()).as_path();
        assert_eq!(
            path,
            "/api/v1/streaming/hashtag?\
            tag=%E3%81%93%E3%82%93%E3%81%AB%E3%81%A1%E3%81%AF"
        );
        assert!(path.parse::<::hyper::Uri>().is_ok());
    }

    #[test]
    fn lines() {
        let (msg_tx, msg_rx) = mpsc::unbounded::<&[u8]>();
        let mut lines = Lines::new(msg_rx.map_err(|_| Error::from_kind(ErrorKind::Http)));

        let send = move |msg| mpsc::UnboundedSender::send(&msg_tx, msg);
        let mut expect = |value| assert_eq!(lines.poll().unwrap(), value);

        // Run on a task context
        futures::lazy(|| {
            send("First".as_bytes());
            expect(Async::NotReady);
            send(" line".as_bytes());
            expect(Async::NotReady);

            send("\nSecond line\nThird line".as_bytes());
            expect(Async::Ready(Some("First line".to_string())));
            expect(Async::Ready(Some("Second line".to_string())));

            send("\n".as_bytes());
            expect(Async::Ready(Some("Third line".to_string())));

            // Send two chunks that, individually, are invalid UTF-8, but
            // combine to form a valid UTF-8 character.
            let cool = "🆒\n";
            assert!(!cool.is_char_boundary(1));
            let (cool1, cool2) = cool.as_bytes().split_at(1);

            send(cool1);
            expect(Async::NotReady);
            send(cool2);
            expect(Async::Ready(Some("🆒".to_string())));

            Ok::<(), ()>(())
        }).wait()
            .unwrap();
    }
}
