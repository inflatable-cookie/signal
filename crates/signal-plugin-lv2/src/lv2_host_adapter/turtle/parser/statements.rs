impl<'a> Parser<'a> {
    fn parse_document(&mut self) -> Result<(), String> {
        loop {
            self.skip_trivia();
            let Some(byte) = self.peek() else {
                return Ok(());
            };
            if byte == b'@' {
                self.parse_directive()?;
            } else {
                self.parse_statement()?;
            }
        }
    }

    fn parse_directive(&mut self) -> Result<(), String> {
        // Consume '@' + keyword.
        self.bump();
        let keyword = self.take_name_chars();
        if keyword != "prefix" {
            return Err(self.error(format!(
                "unsupported directive @{keyword} (only @prefix is in the subset)"
            )));
        }
        self.skip_trivia();
        let prefix = self.take_name_chars();
        self.expect(b':', "':' after prefix name")?;
        self.skip_trivia();
        let iri = self.parse_iri_ref()?;
        self.expect(b'.', "'.' ending @prefix directive")?;
        self.prefixes.insert(prefix, iri);
        Ok(())
    }

    fn parse_statement(&mut self) -> Result<(), String> {
        let subject = self.parse_subject()?;
        self.parse_predicate_object_list(&subject)?;
        self.expect(b'.', "'.' ending statement")?;
        Ok(())
    }

    fn parse_subject(&mut self) -> Result<TurtleTerm, String> {
        self.skip_trivia();
        match self.peek() {
            Some(b'<') => Ok(TurtleTerm::Iri(self.parse_iri_ref()?)),
            Some(b'_') if self.peek_at(1) == Some(b':') => Ok(self.parse_labeled_blank()),
            Some(b'[') => Err(self
                .error("blank-node subjects are outside the supported Turtle subset".to_string())),
            Some(b'(') => {
                Err(self.error("collections are outside the supported Turtle subset".to_string()))
            }
            Some(_) => Ok(TurtleTerm::Iri(self.parse_prefixed_name()?)),
            None => Err(self.error("unexpected end of input reading subject")),
        }
    }

    fn parse_predicate_object_list(&mut self, subject: &TurtleTerm) -> Result<(), String> {
        loop {
            let predicate = self.parse_predicate()?;
            loop {
                let object = self.parse_object()?;
                self.triples.push(TurtleTriple {
                    subject: subject.clone(),
                    predicate: predicate.clone(),
                    object,
                });
                self.skip_trivia();
                if self.peek() == Some(b',') {
                    self.bump();
                } else {
                    break;
                }
            }
            self.skip_trivia();
            if self.peek() == Some(b';') {
                self.bump();
                self.skip_trivia();
                // Trailing ';' before '.' or ']' is legal Turtle.
                match self.peek() {
                    Some(b'.') | Some(b']') | None => return Ok(()),
                    Some(b';') => {
                        // Consecutive semicolons collapse.
                        continue;
                    }
                    _ => continue,
                }
            }
            return Ok(());
        }
    }

    fn parse_predicate(&mut self) -> Result<String, String> {
        self.skip_trivia();
        match self.peek() {
            Some(b'<') => self.parse_iri_ref(),
            Some(b'a')
                if !self
                    .peek_at(1)
                    .is_some_and(|byte| is_name_byte(byte) || byte == b':') =>
            {
                self.bump();
                Ok(RDF_TYPE.to_string())
            }
            Some(_) => self.parse_prefixed_name(),
            None => Err(self.error("unexpected end of input reading predicate")),
        }
    }

    fn parse_object(&mut self) -> Result<TurtleTerm, String> {
        self.skip_trivia();
        match self.peek() {
            Some(b'<') => Ok(TurtleTerm::Iri(self.parse_iri_ref()?)),
            Some(b'"') => self.parse_string_literal(),
            Some(b'[') => self.parse_blank_property_list(),
            Some(b'(') => {
                Err(self.error("collections are outside the supported Turtle subset".to_string()))
            }
            Some(b'_') if self.peek_at(1) == Some(b':') => Ok(self.parse_labeled_blank()),
            Some(byte) if byte == b'+' || byte == b'-' || byte.is_ascii_digit() => {
                self.parse_number()
            }
            Some(_) => {
                // Bareword: boolean or a prefixed name.
                let checkpoint = self.position;
                let word = self.take_name_chars();
                match word.as_str() {
                    "true" if self.peek() != Some(b':') => Ok(TurtleTerm::Bool(true)),
                    "false" if self.peek() != Some(b':') => Ok(TurtleTerm::Bool(false)),
                    _ => {
                        self.position = checkpoint;
                        Ok(TurtleTerm::Iri(self.parse_prefixed_name()?))
                    }
                }
            }
            None => Err(self.error("unexpected end of input reading object")),
        }
    }

    fn parse_blank_property_list(&mut self) -> Result<TurtleTerm, String> {
        self.expect(b'[', "'['")?;
        let blank = TurtleTerm::Blank(self.next_blank);
        self.next_blank += 1;
        self.skip_trivia();
        if self.peek() != Some(b']') {
            self.parse_predicate_object_list(&blank)?;
        }
        self.expect(b']', "']' closing blank node")?;
        Ok(blank)
    }

    fn parse_labeled_blank(&mut self) -> TurtleTerm {
        // Consume `_:`.
        self.bump();
        self.bump();
        let label = self.take_name_chars();
        let next = self.next_blank;
        let id = *self.labeled_blanks.entry(label).or_insert_with(|| next);
        if id == next {
            self.next_blank += 1;
        }
        TurtleTerm::Blank(id)
    }
}
