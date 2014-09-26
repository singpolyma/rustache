use parser::{Parser, Value, Static, Unescaped};
use build::{HashBuilder};
use super::{Data, Str, Bool, Vector, Hash};
pub struct Template<'a>;

impl<'a> Template<'a> {
    pub fn new() -> Template<'a> {
        Template
    }

    fn escape_html(input: &str) -> Box<String> {
        let mut rv = box String::new();
        for c in input.chars() {
            match c {
                '<'  => { rv.push_str("&lt;"); }
                '>'  => { rv.push_str("&gt;"); }
                '&'  => { rv.push_str("&amp;"); }
                '"'  => { rv.push_str("&quot;"); }
                _    => { rv.push_char(c); }
            }
        }
        rv
    }

    fn handle_unescaped_node<'a, W: Writer>(data: &Data, writer: &mut W) {
        let mut tmp: String = String::new();
        match *data {

            Str(ref val) => {
                tmp = tmp + *val;
            }
            Bool(val) => {
                if val {
                    tmp.push_str("true");
                } else {
                    tmp.push_str("false");
                }
            }
            Vector(_) => {
                fail!("expecting text, found vector data");
            }
            Hash(_) => {
                fail!("expecting text, found hash data");
            }
        }

        if tmp.len() != 0 {
            writer.write_str(tmp.as_slice()).ok().expect("write failed in render");
        }        
    }

    fn handle_value_node<'a, W: Writer>(data: &Data, writer: &mut W) {
        let mut tmp: String = String::new();
        match *data {

            Str(ref val) => {
                tmp = *Template::escape_html(&(*val.as_slice()));
            }
            Bool(val) => {
                if val {
                    tmp.push_str("true");
                } else {
                    tmp.push_str("false");
                }
            }
            Vector(_) => {
                fail!("expecting text, found vector data");
            }
            Hash(_) => {
                fail!("expecting text, found hash data");
            }
        }

        if tmp.len() != 0 {
            writer.write_str(tmp.as_slice()).ok().expect("write failed in render");
        }        
    }

    pub fn render_data<'a, W: Writer>(writer: &mut W,  
                                      datastore: &HashBuilder, 
                                      parser: &Parser) {
        let mut tmp: String = String::new();
        for node in parser.nodes.iter() {
            tmp.truncate(0);
            match *node {
                Unescaped(key)  => {
                    let tmp = key.to_string();
                    if datastore.data.contains_key(&tmp) {
                        let ref val = datastore.data[tmp];
                        Template::handle_unescaped_node(val, writer);
                    }
                }
                Value(key) => {
                    let tmp = key.to_string();
                    if datastore.data.contains_key(&tmp) {
                        let ref val = datastore.data[tmp];
                        Template::handle_value_node(val, writer);
                    }
                }

                Static(ref key) => {
                    tmp.push_str(key.as_slice());
                    writer.write_str(tmp.as_slice()).ok().expect("write failed in render");
                }
                _ => continue
            }
        }
    }
}



#[cfg(test)]
mod template_tests {
    use std::io::MemWriter;
    use std::str;

    use parser::Parser;
    use template::Template;
    use compiler::Compiler;
    use build::HashBuilder;

    #[test]
    fn test_escape_html() {
        let s1 = "a < b > c & d \"spam\"\'";
        let a1 = "a &lt; b &gt; c &amp; d &quot;spam&quot;'";
        let s2 = "1<2 <b>hello</b>";
        let a2 = "1&lt;2 &lt;b&gt;hello&lt;/b&gt;";

        let mut w = MemWriter::new();
        let compiler = Compiler::new("{{ value }}");
        let parser = Parser::new(&compiler.tokens);
        let mut data = HashBuilder::new().insert_string("value", s1);
        Template::render_data(&mut w, &data, &parser);
        assert_eq!(a1, str::from_utf8(w.get_ref()).unwrap());

        w = MemWriter::new();
        data = HashBuilder::new().insert_string("value", s2);
        Template::render_data(&mut w, &data, &parser);
        assert_eq!(a2, str::from_utf8(w.get_ref()).unwrap());
    }

    #[test]
    fn test_not_escape_html() {
        let s2 = "1<2 <b>hello</b>";
        let mut w = MemWriter::new();
        let compiler2 = Compiler::new("{{& value }}");

        let parser = Parser::new(&compiler2.tokens);
        let data = HashBuilder::new().insert_string("value", s2);

        Template::render_data(&mut w, &data, &parser);
        assert_eq!(s2, str::from_utf8(w.get_ref()).unwrap());        
    }

    #[test]
    fn test_render_to_io_stream() {
        let mut w = MemWriter::new();

        let data = HashBuilder::new().insert_string("value1", "The heading");

        let compiler = Compiler::new("<h1>{{ value1 }}</h1>");
        let parser = Parser::new(&compiler.tokens);

        Template::render_data(&mut w, &data, &parser);
        assert_eq!("<h1>The heading</h1>".to_string(), str::from_utf8_owned(w.unwrap()).unwrap());
    }

    #[test]
    fn test_unescaped_node_correct_string_data() {
        let mut w = MemWriter::new();
        let compiler = Compiler::new("<h1>{{ value1 }}</h1>");
        let parser = Parser::new(&compiler.tokens);
        let data = HashBuilder::new().insert_string("value1", "The heading");

        Template::render_data(&mut w, &data, &parser);

        assert_eq!("<h1>The heading</h1>".to_string(), str::from_utf8_owned(w.unwrap()).unwrap());
    }

    #[test]
    fn test_unescaped_node_correct_bool_data() {
        let mut w = MemWriter::new();
        let compiler = Compiler::new("{{ value1 }}");
        let parser = Parser::new(&compiler.tokens);
        let data = HashBuilder::new().insert_bool("value1", true);

        Template::render_data(&mut w, &data, &parser);

        assert_eq!("true".to_string(), str::from_utf8_owned(w.unwrap()).unwrap());
    }

    #[test]
    #[should_fail]
    fn test_unescaped_node_incorrect_vector_data() {
        let mut w = MemWriter::new();
        let compiler = Compiler::new("<h1>{{ value1 }}</h1>");
        let parser = Parser::new(&compiler.tokens);
        let mut data = HashBuilder::new();

        data = data.insert_vector("value1", |builder| {
            builder.push_string("Prophet Velen")
        });

        Template::render_data(&mut w, &data, &parser);
    }

    #[test]
    #[should_fail]
    fn test_unescaped_node_incorrect_hash_data() {
        let mut w = MemWriter::new();
        let compiler = Compiler::new("<h1>{{ value1 }}</h1>");
        let parser = Parser::new(&compiler.tokens);
        let mut data = HashBuilder::new();

        data = data.insert_hash("value1", |builder| {
            builder.insert_string("name", "Hearthstone: Heroes of Warcraft")
        });

        Template::render_data(&mut w, &data, &parser);
    }
}
