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
        Utf8 {
            description("invalid UTF-8 string")
        }
        Encode {
            description("could not encode value")
        }
        JsonDecode(value: String) {
            description("invalid JSON")
            display("could not parse JSON:\n{}", value)
        }
    }
}
