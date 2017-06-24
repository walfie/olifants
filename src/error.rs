use hyper;

error_chain!{
    errors {
        Initialization {
            description("failed to initialize client")
        }
        Uri(uri: String) {
            description("invalid URI")
            display("could not parse URI: `{}`", uri)
        }
        Http {
            description("HTTP error")
        }
        StatusCode(
            status: hyper::StatusCode,
            version: hyper::HttpVersion,
            headers: hyper::Headers,
            body: String
        ) {
            description("Received non-2XX status code from server")
            display(
                "HTTP error\n{} {}\n{}\n{}",
                version,
                status,
                headers,
                body
            )
        }
        Deserialize(value: String) {
            description("could not deserialize value")
            display("could not deserialize value: `{}`", value)
        }
        Utf8 {
            description("bytes contained invalid UTF-8")
        }
        EventType(value: String) {
            description("unknown event type")
            display("unknown event type returned from API: `{}`", value)
        }
        StreamingState(expected: &'static str, actual: String) {
            description("streaming API is in an unexpected state")
            display("expected `{}` from streaming API, received `{}`", expected, actual)
        }
        StatusId(value: String) {
            description("received invalid status ID from API")
            display("could not parse status ID `{}` as an integer", value)
        }
    }
}
