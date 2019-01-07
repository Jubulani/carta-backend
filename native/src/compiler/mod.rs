mod tokeniser;

use std::fs;

use crate::compiler::tokeniser::{Token, TokenType, Tokeniser};

pub struct Schema {
    nuggets: Vec<ILNugget>,
}

impl Schema {
    fn push(&mut self, n: ILNugget) {
        self.nuggets.push(n);
    }
}

impl Schema {
    pub fn iter<'a>(&'a self) -> IterSchema<'a> {
        IterSchema {
            inner: self,
            pos: 0,
        }
    }

    fn get_type(&self, name: &str) -> NuggetType {
        match name {
            "int8" => NuggetType::SimpleType {
                size: 1,
                kind: "int8",
            },
            "int16_be" => NuggetType::SimpleType {
                size: 2,
                kind: "int16_be",
            },
            "int16_le" => NuggetType::SimpleType {
                size: 2,
                kind: "int16_le",
            },
            "int32_be" => NuggetType::SimpleType {
                size: 4,
                kind: "int32_be",
            },
            "int32_le" => NuggetType::SimpleType {
                size: 4,
                kind: "int32_le",
            },
            "int64_be" => NuggetType::SimpleType {
                size: 8,
                kind: "int64_be",
            },
            "int64_le" => NuggetType::SimpleType {
                size: 8,
                kind: "int64_le",
            },
            "uint8" => NuggetType::SimpleType {
                size: 1,
                kind: "uint8",
            },
            "uint16_be" => NuggetType::SimpleType {
                size: 2,
                kind: "uint16_be",
            },
            "uint16_le" => NuggetType::SimpleType {
                size: 2,
                kind: "uint16_le",
            },
            "uint32_be" => NuggetType::SimpleType {
                size: 4,
                kind: "uint32_be",
            },
            "uint32_le" => NuggetType::SimpleType {
                size: 4,
                kind: "uint32_le",
            },
            "uint64_be" => NuggetType::SimpleType {
                size: 8,
                kind: "uint64_be",
            },
            "uint64_le" => NuggetType::SimpleType {
                size: 8,
                kind: "uint64_le",
            },
            "f32_be" => NuggetType::SimpleType {
                size: 4,
                kind: "f32_be",
            },
            "f32_le" => NuggetType::SimpleType {
                size: 4,
                kind: "f32_le",
            },
            "f64_be" => NuggetType::SimpleType {
                size: 8,
                kind: "f64_be",
            },
            "f64_le" => NuggetType::SimpleType {
                size: 8,
                kind: "f64_le",
            },
            _ => panic!("Unrecognised type: {}", name),
        }
    }
}

pub struct IterSchema<'a> {
    inner: &'a Schema,
    pos: usize,
}

impl<'a> Iterator for IterSchema<'a> {
    type Item = &'a ILNugget;

    fn next(&mut self) -> Option<&'a ILNugget> {
        if self.pos < self.inner.nuggets.len() {
            let n = &self.inner.nuggets[self.pos];
            self.pos += 1;
            Some(n)
        } else {
            None
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct ILNugget {
    name: String,
    kind: NuggetType,
}

struct ArrayType {}

#[derive(PartialEq, Debug)]
enum NuggetType {
    CompoundType { children: Vec<ILNugget> },
    SimpleType { size: usize, kind: &'static str },
}

/*struct NuggetStructDefn {
    name: String,
    members: Vec<NuggetType::SimpleType>,
}*/

trait CompilerState {
    fn new_token(self: Box<Self>, t: &Token, schema: &mut Schema) -> Box<dyn CompilerState>;
}

struct EmptyState;

impl CompilerState for EmptyState {
    fn new_token(self: Box<Self>, t: &Token, _: &mut Schema) -> Box<dyn CompilerState> {
        if let Some(s) = new_state(t) {
            return s;
        }
        self
    }
}

struct NewNuggetState {
    state: NewNuggetSubState,
    name: String,
    kind: Option<NuggetType>,
}

#[derive(PartialEq)]
enum NewNuggetSubState {
    Name,
    TypeOf,
    Kind,
}

impl NewNuggetState {
    fn new(t: &Token) -> NewNuggetState {
        NewNuggetState {
            state: NewNuggetSubState::Name,
            name: t.get_value().to_string(),
            kind: None,
        }
    }
}

impl CompilerState for NewNuggetState {
    fn new_token(mut self: Box<Self>, t: &Token, schema: &mut Schema) -> Box<dyn CompilerState> {
        match self.state {
            NewNuggetSubState::Name => {
                // Next state must be TypeOf
                if t.kind != TokenType::TypeOf {
                    panic!("Expected ':', got: {:?}", t);
                }
                self.state = NewNuggetSubState::TypeOf;
            }
            NewNuggetSubState::TypeOf => {
                // Next state must be the type name
                self.kind = Some(schema.get_type(t.get_value()));
                self.state = NewNuggetSubState::Kind;
            }
            NewNuggetSubState::Kind => {
                // Next state must be a newline
                if t.kind != TokenType::NewLine {
                    panic!("Expected '\n', got: {:?}", t);
                }

                if let Some(kind) = self.kind {
                    let nugget = ILNugget {
                        name: self.name,
                        kind,
                    };
                    schema.push(nugget);
                    return Box::new(EmptyState);
                } else {
                    panic!("Kind not available");
                }
            }
        }

        self
    }
}

struct StructState {
    state: StructSubState,
    name: String,
    complete_children: Vec<ILNugget>,
    building_child: Option<ILNugget>,
}

#[derive(PartialEq)]
enum StructSubState {
    Name,
    OpenBrace,
    ChildName,
    ChildTypeOf,
    ChildKind,
}

impl StructState {
    fn new(t: &Token) -> StructState {
        StructState {
            state: StructSubState::Name,
            name: t.get_value().to_string(),
            complete_children: Vec::new(),
            building_child: None,
        }
    }
}

impl CompilerState for StructState {
    fn new_token(mut self: Box<Self>, t: &Token, schema: &mut Schema) -> Box<dyn CompilerState> {
        // New lines are ignored in struct definitions
        if t.kind == TokenType::NewLine {
            return self;
        }

        /*match self.state {
        StructSubState::Name => {
            // Next state must be OpenBrace
            if t.kind != TokenType::OpenBrace {
                panic!("Expected '{{', got: {:?}", t);
            }
            self.state = StructSubState::OpenBrace;
        }
        StructSubState::OpenBrace => {
            // Next state must be the type name, or a close brace for an empty struct
            match t.kind {
                TokenType::CloseBrace => "??",
                TokenType::Word => {}
            }
            self.kind = Some(schema.get_type(t.get_value()));
            self.state = NewNuggetSubState::Kind;
        } /*StructSubState::Kind => {
              // Next state must be a newline
              if t.kind != TokenType::NewLine {
                  panic!("Expected '\n', got: {:?}", t);
              }

              if let Some(kind) = self.kind {
                  let nugget = ILNugget {
                      name: self.name,
                      kind,
                  };
                  schema.push(nugget);
                  return Box::new(EmptyState);
              } else {
                  panic!("Kind not available");
              }
          }*/
        }*/

        self
    }
}

fn new_state(t: &Token) -> Option<Box<dyn CompilerState>> {
    if t.kind == TokenType::Word {
        // Match against language keywords
        return match t.get_value() {
            "struct" => Some(Box::new(StructState::new(t))),

            // If not a keyword, must be a new nugget name
            _ => Some(Box::new(NewNuggetState::new(t))),
        };
    } else if t.kind == TokenType::NewLine {
        // Empty newline - nothing to parse
        return None;
    }

    panic!("Unknown token: {:?}", t);
}

pub fn compile_schema_file(filename: &str) -> Schema {
    let s = fs::read_to_string(filename).unwrap();
    compile_schema(&s)
}

fn compile_schema(s: &str) -> Schema {
    let tokeniser = Tokeniser::new(s);

    let mut schema = Schema {
        nuggets: Vec::new(),
    };
    let mut state: Box<dyn CompilerState> = Box::new(EmptyState {});
    for token in tokeniser.iter() {
        state = state.new_token(token, &mut schema);
    }

    // Add a final newline, in case one doesn't exist in the input
    // This will flush any remaining (valid) tokens
    state.new_token(&Token::new(TokenType::NewLine, None), &mut schema);

    schema
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_builtin() {
        let schema = compile_schema("new_name: uint64_le");
        let mut iter = schema.iter();
        assert_eq!(
            iter.next(),
            Some(&ILNugget {
                name: "new_name".to_string(),
                kind: NuggetType::SimpleType {
                    size: 8,
                    kind: "uint64_le"
                },
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_empty_input() {
        let schema = compile_schema("\n  \t\n\n");
        let mut iter = schema.iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_multiple_builtins() {
        let schema = compile_schema(
            "name1: int8
name2: uint64_be

name3: f64_le
",
        );
        let mut iter = schema.iter();
        assert_eq!(
            iter.next(),
            Some(&ILNugget {
                name: "name1".to_string(),
                kind: NuggetType::SimpleType {
                    size: 1,
                    kind: "int8"
                },
            })
        );
        assert_eq!(
            iter.next(),
            Some(&ILNugget {
                name: "name2".to_string(),
                kind: NuggetType::SimpleType {
                    size: 8,
                    kind: "uint64_be"
                },
            })
        );
        assert_eq!(
            iter.next(),
            Some(&ILNugget {
                name: "name3".to_string(),
                kind: NuggetType::SimpleType {
                    size: 8,
                    kind: "f64_le"
                },
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_new_type() {
        let schema = compile_schema(
            "struct new_type {
    inner_val1: int8,
    inner_val2: int8,
}
val: new_type",
        );
        let mut iter = schema.iter();
        /*assert_eq!(
            iter.next(),
            Some(&ILNugget {
                name: "new_type".to_string(),
                size: 1,
                kind: NuggetType::CompoundType {
                    children: vec![ILNugget {
                        name: "inner_val".to_string(),
                        size: 1,
                        kind: NuggetType::SimpleType {
                            size: 1,
                            kind: "int8"
                        }
                    }]
                },
            })
        );*/
        assert_eq!(
            iter.next(),
            Some(&ILNugget {
                name: "val".to_string(),
                kind: NuggetType::CompoundType {
                    children: vec![
                        ILNugget {
                            name: "inner_val1".to_string(),
                            kind: NuggetType::SimpleType {
                                size: 1,
                                kind: "int8"
                            }
                        },
                        ILNugget {
                            name: "inner_val2".to_string(),
                            kind: NuggetType::SimpleType {
                                size: 1,
                                kind: "int8"
                            }
                        }
                    ]
                },
            })
        );
        assert_eq!(iter.next(), None);
    }
}
