#[derive(Debug)]
pub struct GaiLog {
    pub prefix: Option<String>,
    pub scope: Option<String>,
    pub breaking: bool,
    pub header: Option<String>,
    pub body: Option<String>,

    // this only gets populated
    // if the prefix, scope, and breaking dont
    // exist
    //
    // we could just return the header
    // but me thinks it might get
    // confusing
    pub message: Option<String>,

    pub date: String,
    pub author: String,
    pub commit_hash: String,
}

impl GaiLog {
    fn parse(message: &str) -> Self {
        let mut lines = message.lines();

        let first = lines.next().unwrap_or("").trim();

        let body: String =
            lines.collect::<Vec<_>>().join("\n").trim().to_string();

        let body = if body.is_empty() { None } else { Some(body) };

        // delimit
        let Some(colon_pos) = first.find(':') else {
            return Self::message(message);
        };

        let before = &first[..colon_pos];
        let header = first[colon_pos + 1..].trim();

        // find breaking
        let (before_colon, breaking) = match before.strip_suffix('!')
        {
            Some(s) => (s, true),
            None => (before, false),
        };

        //find prefix + scope
        let (prefix, scope) =
            if let Some(paren_start) = before_colon.find('(') {
                let Some(paren_end) = before_colon.find(')') else {
                    return Self::message(message);
                };
                if paren_end <= paren_start {
                    return Self::message(message);
                }
                (
                    &before_colon[..paren_start],
                    Some(&before_colon[paren_start + 1..paren_end]),
                )
            } else {
                (before_colon, None)
            };

        let prefix = if prefix.is_empty() {
            None
        } else {
            Some(prefix.to_owned())
        };

        let header = if header.is_empty() {
            None
        } else {
            Some(header.to_owned())
        };

        Self {
            prefix,
            scope: scope.map(String::from),
            breaking,
            header,
            body,
            message: None,
            date: String::new(),
            author: String::new(),
            commit_hash: String::new(),
        }
    }

    fn message(message: &str) -> Self {
        Self {
            prefix: None,
            scope: None,
            breaking: false,
            header: None,
            body: None,
            message: Some(message.trim().to_string()),
            date: String::new(),
            author: String::new(),
            commit_hash: String::new(),
        }
    }
}
