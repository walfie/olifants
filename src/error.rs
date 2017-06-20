error_chain!{
    errors {
        ClientInitialization {
            description("failed to initialize client")
        }
        InvalidUrl {
            description("invalid URL")
        }
        Api {
            description("error returned from API")
        }
        Http {
            description("HTTP error")
        }
        Encode {
            description("could not encode value")
        }
        InvalidUtf8 {
            description("bytes contained invalid UTF-8")
        }
        JsonDecode(value: String) {
            description("invalid JSON")
            display("could not parse JSON:\n{}", value)
        }
        UnknownEventType(value: String) {
            description("unknown event type")
            display("unknown event type returned from API: {}", value)
        }
        IllegalState(expected: &'static str, actual: String) {
            description("streaming API is in an unexpected state")
            display("expected `{}` from API, received `{}`", expected, actual)
        }
        InvalidNumber(value: String) {
            description("received invalid number from API")
            display("could not parse {} as an integer", value)
        }
    }
}
