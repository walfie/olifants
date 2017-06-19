use api;
use error::*;
use futures::{Async, Stream};
use serde_json;

#[derive(Clone, Copy, Debug, PartialEq)]
enum EventType {
    Update,
    Delete,
    Notification,
}

#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Timeline<S> {
    lines: S,
    waiting_for: Option<EventType>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Event {
    Update(Box<api::Status>),
    Notification, // TODO
    Delete(i64),
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
                                    ErrorKind::JsonDecode(data.to_string())
                                }).map(|status| {
                                    Async::Ready(Some(Event::Update(Box::new(status))))
                                })
                            }
                            Delete => {
                                return data.parse::<i64>()
                                    .chain_err(|| ErrorKind::InvalidNumber(data.to_string()))
                                    .map(|id| Async::Ready(Some(Event::Delete(id))))
                            }
                            Notification => {
                                // Unimplemented
                            }
                        }
                    } else {
                        // We're in an unexpected state, reset to be safe
                        self.waiting_for = None;
                        let kind = ErrorKind::IllegalState("data", line);
                        return Err(Error::from_kind(kind));
                    }
                } else if line.starts_with("event: ") {
                    let event_type = match &line[7..] {
                        "update" => Update,
                        "delete" => Delete,
                        "notification" => Notification,
                        other => {
                            let kind = ErrorKind::UnknownEventType(other.to_string());
                            return Err(Error::from_kind(kind));
                        }
                    };

                    self.waiting_for = Some(event_type);
                } else if !line.is_empty() {
                    let kind = ErrorKind::IllegalState("event", line);
                    return Err(Error::from_kind(kind));
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
    buffer: String,
}

impl<S, B> Lines<S>
where
    B: AsRef<[u8]>,
    S: Stream<Item = B, Error = Error>,
{
    pub fn new(stream: S) -> Self {
        Lines {
            stream,
            buffer: String::new(),
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
            if let Some(index) = self.buffer.find('\n') {
                let mut split = self.buffer.split_off(index + 1);
                ::std::mem::swap(&mut self.buffer, &mut split);
                split.pop(); // Remove trailing newline
                return Ok(Async::Ready(Some(split)));
            } else {
                // Attempt to fill the buffer
                if let Some(chunk) = try_ready!(self.stream.poll()) {
                    let s = String::from_utf8_lossy(&chunk.as_ref());
                    self.buffer.push_str(&s);
                } else {
                    return Ok(Async::Ready(None));
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::{self, Future, Stream};
    use futures::unsync::mpsc;

    #[test]
    #[allow(unused_must_use)]
    fn lines() {
        // Run on a task context
        futures::lazy(|| {
            let (msg_tx, msg_rx) = mpsc::unbounded::<&str>();
            let mut lines = Lines::new(msg_rx.map_err(|_| Error::from_kind(ErrorKind::Http)));

            let send = move |msg| mpsc::UnboundedSender::send(&msg_tx, msg);

            send("First");
            assert_eq!(lines.poll().unwrap(), Async::NotReady);
            send(" line");
            assert_eq!(lines.poll().unwrap(), Async::NotReady);
            send("\nSecond line\nThird line");
            assert_eq!(
                lines.poll().unwrap(),
                Async::Ready(Some("First line".to_string()))
            );
            assert_eq!(
                lines.poll().unwrap(),
                Async::Ready(Some("Second line".to_string()))
            );
            assert_eq!(lines.poll().unwrap(), Async::NotReady);
            send("\n");
            assert_eq!(
                lines.poll().unwrap(),
                Async::Ready(Some("Third line".to_string()))
            );

            Ok::<(), ()>(())
        }).wait()
            .unwrap();
    }
}
