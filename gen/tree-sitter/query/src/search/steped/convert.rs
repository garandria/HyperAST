struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            if let (Some(file), Some(line)) = (record.file(), record.line()) {
                eprintln!("{}:{} {} - {}", file, line, record.level(), record.args());
            } else {
                eprintln!("{} - {}", record.level(), record.args());
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

#[test]
fn convert() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    let language = tree_sitter_java::language();
    let source = r#"(class_declaration
        name: (identifier) @name
        body: (class_body) @class_body)"#;
    let source = r#"(class_declaration
        (identifier) @name
        (class_body) @class_body)"#;
    let source = "(_
        (expression_statement)
        .
        (statement)
    ) @a";
    let mut error_offset = 0u32;
    let mut error_type: tree_sitter::ffi::TSQueryError = 0;
    let bytes = source.as_bytes();
    // Compile the query.
    let ptr = unsafe {
        tree_sitter::ffi::ts_query_new(
            language.into_raw(),
            bytes.as_ptr().cast::<std::ffi::c_char>(),
            bytes.len() as u32,
            std::ptr::addr_of_mut!(error_offset),
            std::ptr::addr_of_mut!(error_type),
        )
    };
    let language = tree_sitter_java::language();

    // On failure, build an error based on the error code and offset.
    if ptr.is_null() {
        use tree_sitter::ffi;
        use tree_sitter::QueryError;
        use tree_sitter::QueryErrorKind;
        if error_type == ffi::TSQueryErrorLanguage {
            panic!();
        }

        let offset = error_offset as usize;
        let mut line_start = 0;
        let mut row = 0;
        let mut line_containing_error = None;
        for line in source.lines() {
            let line_end = line_start + line.len() + 1;
            if line_end > offset {
                line_containing_error = Some(line);
                break;
            }
            line_start = line_end;
            row += 1;
        }
        let column = offset - line_start;

        let kind;
        let message;
        match error_type {
            // Error types that report names
            ffi::TSQueryErrorNodeType | ffi::TSQueryErrorField | ffi::TSQueryErrorCapture => {
                let suffix = source.split_at(offset).1;
                let end_offset = suffix
                    .find(|c| !char::is_alphanumeric(c) && c != '_' && c != '-')
                    .unwrap_or(suffix.len());
                message = suffix.split_at(end_offset).0.to_string();
                kind = match error_type {
                    ffi::TSQueryErrorNodeType => QueryErrorKind::NodeType,
                    ffi::TSQueryErrorField => QueryErrorKind::Field,
                    ffi::TSQueryErrorCapture => QueryErrorKind::Capture,
                    _ => unreachable!(),
                };
            }

            // Error types that report positions
            _ => {
                message = line_containing_error.map_or_else(
                    || "Unexpected EOF".to_string(),
                    |line| line.to_string() + "\n" + &" ".repeat(offset - line_start) + "^",
                );
                kind = match error_type {
                    ffi::TSQueryErrorStructure => QueryErrorKind::Structure,
                    _ => QueryErrorKind::Syntax,
                };
            }
        };

        dbg!(QueryError {
            row,
            column,
            offset,
            message,
            kind,
        });
    };

    let query: *mut super::TSQuery = unsafe { std::mem::transmute(ptr) };
    eprintln!("{}", unsafe { query.as_ref().unwrap() });

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).unwrap();
    let text = "class A {}";
    let text = "class A {
        B f() {
            a++;
            return null;
        }
    }";
    let tree = parser.parse(text, None).unwrap();

    let mut qcursor = super::QueryCursor::<'_, tree_sitter::TreeCursor, tree_sitter::Node> {
        halted: false,
        ascending: false,
        states: vec![],
        capture_list_pool: super::CaptureListPool::default(),
        finished_states: Default::default(),
        max_start_depth: u32::MAX,
        did_exceed_match_limit: false,
        // .did_exceed_match_limit = false,
        // .ascending = false,
        // .halted = false,
        // .states = array_new(),
        // .finished_states = array_new(),
        // .capture_list_pool = capture_list_pool_new(),
        // .max_start_depth = UINT32_MAX,
        depth: 0,
        on_visible_node: true,
        query,
        cursor: tree.root_node().walk(),
        next_state_id: 0,
        //   array_clear(&self->states);
        //   array_clear(&self->finished_states);
        //   ts_tree_cursor_reset(&self->cursor, node);
        //   capture_list_pool_reset(&self->capture_list_pool);
        //   self->on_visible_node = true;
        //   self->next_state_id = 0;
        //   self->depth = 0;
        //   self->ascending = false;
        //   self->halted = false;
        //   self->query = query;
        //   self->did_exceed_match_limit = false;

        // .start_byte = 0,
        // .end_byte = UINT32_MAX,
        // .start_point = {0, 0},
        // .end_point = POINT_MAX,
        _phantom: std::marker::PhantomData,
    };
    let mut matched = false;
    while let Some(m) = qcursor.next_match() {
        dbg!(m.pattern_index);
        dbg!(m.captures.len());
        for c in &m.captures {
            let i = c.index;
            dbg!(i);
            unsafe {
                let mut length = 0u32;
                let name = tree_sitter::ffi::ts_query_capture_name_for_id(
                    ptr,
                    i,
                    std::ptr::addr_of_mut!(length),
                )
                .cast::<u8>();
                let name = std::slice::from_raw_parts(name, length as usize);
                let name = std::str::from_utf8_unchecked(name);
                dbg!(name)
            };
            dbg!(c.node.utf8_text(text.as_bytes()).unwrap());
        }
        matched = true;
    }
    assert!(matched)
}
