error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }
    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
        Network(::reqwest::Error);
        Parse(::serde_json::error::Error);
    }
    errors {
        ExpiredToken {
            description("expired token")
            display("token has expired")
        }
        MissingField(field: String){
            description("missing field")
            display("missing field: {}", field)
        }
        NotAvailable {
            description("not available")
            display("the resource requested is not available")
        }
    }
}
