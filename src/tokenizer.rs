use std::str::pattern::Pattern;

pub struct Tokenizer {
    pub tokens: Vec<Token>,
    pub lines: Box<[String]>,
}

impl Tokenizer {
    pub fn chunk(&mut self) -> anyhow::Result<()> {
        let lines_iter = self.lines.iter();
        for line in lines_iter {
            // potential new chunk
            if !Self::too_much_whitespace(line) {
                let trimmed_line = line.trim_start();
                if trimmed_line.is_empty() {
                    self.tokens.push(Token {
                        token_type: TokenType::BlankLine,
                        text: String::new(),
                        open: false,
                    });
                    continue;
                }
                let mut trimmed_chars = trimmed_line.chars();

                // we use the first character to determine what kind of line this is
                let mut token = trimmed_chars.next().map_or_else(
                    || Token {
                        token_type: TokenType::Paragraph,
                        text: String::from(trimmed_line),
                        open: false,
                    },
                    |ch| match ch {
                        '#' => Token {
                            token_type: TokenType::Header(1),
                            text: String::from(trimmed_line),
                            open: true,
                        },
                        '-' | '=' | '_' | '*' | '+' => Token {
                            token_type: TokenType::Undetermined(ch, 1),
                            text: String::from(trimmed_line),
                            open: true,
                        },
                        _ => Token {
                            token_type: TokenType::Paragraph,
                            text: String::from(trimmed_line),
                            open: false,
                        },
                    },
                );

                if token.open {
                    match token.token_type {
                        TokenType::Header(ref mut n) => {
                            for character in trimmed_chars {
                                if '#' == character {
                                    *n += 1;
                                }
                                // a header is only valid if it has 1-6 #
                                if *n > 6 {
                                    token.token_type = TokenType::Paragraph;
                                    break;
                                }
                            }
                            token.open = false;
                        }
                        TokenType::Undetermined(c, ref mut n) => match c {
                            '-' | '=' => {
                                for character in trimmed_chars {
                                    if '-' == character {
                                        *n += 1;
                                    }
                                }
                                match self.tokens[self.tokens.len() - 1].token_type {
                                    //TODO: this could actually also be a list if its following a paragraph
                                    // we can differentiate depending on if theres any non - or = characters
                                    TokenType::Paragraph => {
                                        token.token_type = TokenType::SetextHeading;
                                        token.open = false;
                                    }
                                    _ => {
                                        if *n >= 3 {
                                            token.token_type = TokenType::ThematicBreak;
                                            token.open = false;
                                        }
                                    }
                                }
                            }
                            _ => unreachable!(),
                        },
                        _ => {}
                    }
                }

                self.tokens.push(token);

                if "```".is_prefix_of(trimmed_line) || "~~~".is_prefix_of(trimmed_line) {}
                if "---".is_prefix_of(trimmed_line) || "===".is_prefix_of(trimmed_line) {}
                if "---".is_prefix_of(trimmed_line)
                    || "___".is_prefix_of(trimmed_line)
                    || "***".is_prefix_of(trimmed_line)
                {}
                if ">".is_prefix_of(trimmed_line) {}

                if "-".is_prefix_of(trimmed_line)
                    || "*".is_prefix_of(trimmed_line)
                    || "+".is_prefix_of(trimmed_line)
                {}

                // if trimmed_chars[0].is_digit(10)
                //     && (trimmed_chars[1] == ')' || trimmed_chars[1] == '.')
                // {}
            }
        }

        Ok(())
    }

    /// in Commonmark markdown, any line that starts with 4 spaces of whitespace
    /// escapes most punctuation like #, >, or ```.
    /// However, when it is not following a paragraph, the line is treated as
    /// a codeblock
    pub fn too_much_whitespace(line: &str) -> bool {
        let mut whitespace_len = 0;
        for ch in line.chars() {
            if ch == '\t' {
                return true;
            } else if ch.is_whitespace() {
                whitespace_len += 1;
            } else {
                return whitespace_len >= 4;
            }
            if whitespace_len >= 4 {
                return true;
            }
        }

        false
    }
}

#[derive(Debug)]
enum TokenType {
    // Since multiple tokens begin with the same character,
    // We leave open ambigious tokens in an undetermined state
    // where we keep track of the first character of the line
    // that started this token, and the amount of reoccurences
    Undetermined(char, u8),

    // Inline

    // amount of `
    CodeSpan(u8),

    // _ or *, amount
    Emphasis(char, u8),

    HardLineBreak,
    SoftLineBreak,

    Link,

    Autolink,

    // Leaf Blocks

    // amount of #
    Header(u32),
    // - or =
    SetextHeading,

    LinkReferenceDef,

    BlankLine,

    IndentCodeBlock,
    // ~ or `, amount
    FencedCodeBlock(char, u8),

    // * or _ or -
    ThematicBreak,

    Paragraph,

    // Container Blocks
    BlockQuote(Box<TokenType>),

    // - or + or *, type
    BulletList(char, Box<TokenType>),
    // ) or ., type
    OrderedList(char, Box<TokenType>),
}

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    text: String,
    open: bool,
}
