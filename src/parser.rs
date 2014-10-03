// The parser processes a list of mustache tokens created in
// the compiler into a list of templater useable nodes.
// Nodes contain only the necessary information to be used
// to seek out appropriate data for injection.

use compiler::{Token, Text, Variable, OTag, CTag, Raw, Partial};

#[deriving(PartialEq, Eq, Clone, Show)]
pub enum Node<'a> {
    Static(&'a str),
    Value(&'a str, String),
    // (name, children, inverted)
    Section(&'a str, Vec<Node<'a>>, bool, String, String),
    Unescaped(&'a str, String),
    Part(&'a str, &'a str)
}
pub fn parse_nodes<'a>(list: &Vec<Token<'a>>) -> Vec<Node<'a>> {
    let mut nodes: Vec<Node> = vec![];
    let mut it = list.iter().enumerate();

    loop {
        match it.next() {
            Some((i, &token)) => {
                match token {
                    Text(text) => nodes.push(Static(text)),
                    Variable(name, raw) => {
                        let dot_notation = name.contains_char('.');
                        match dot_notation {
                            false => nodes.push(Value(name, raw.to_string())),
                            true => {
                                let parts: Vec<&str> = name.split_str(".").collect();
                                let (section, variable) = (parts[0], parts[parts.len() - 1]);
                                let mut var = "{{".to_string();
                                let mut otag = "{{#".to_string();
                                let mut ctag = "{{/".to_string();
                                var.push_str(variable);
                                var.push_str("}}");
                                otag.push_str(section);
                                otag.push_str("}}");
                                ctag.push_str(section);
                                ctag.push_str("}}");

                                nodes.push(Section(section, vec![Value(variable, var)], false, otag, ctag))
                            }
                        }
                    },
                    Raw(name, raw) => {
                        let dot_notation = name.contains_char('.');
                        match dot_notation {
                            false => nodes.push(Unescaped(name, raw.to_string())),
                            true => {
                                let parts: Vec<&str> = name.split_str(".").collect();
                                let (section, variable) = (parts[0], parts[parts.len() - 1]);
                                let mut var = String::new();
                                let ampersand = raw.contains_char('&');
                                match ampersand {
                                    true => {
                                        var.push_str("{{&");
                                        var.push_str(variable);
                                        var.push_str("}}");
                                    },
                                    false => {
                                        var.push_str("{{{");
                                        var.push_str(variable);
                                        var.push_str("}}}");
                                    }
                                }
                                let mut otag = "{{#".to_string();
                                let mut ctag = "{{/".to_string();
                                otag.push_str(section);
                                otag.push_str("}}");
                                ctag.push_str(section);
                                ctag.push_str("}}");

                                nodes.push(Section(section, vec![Unescaped(variable, var)], false, otag, ctag))
                            }
                        }
                    }
                    Partial(name, raw) => nodes.push(Part(name, raw)),
                    CTag(_, _) => {
                        // CTags that are processed outside of the context of a 
                        // corresponding OTag are ignored.
                        continue;
                    },
                    OTag(name, inverted, raw) => {
                        let mut children: Vec<Token> = vec![];
                        let mut count = 0u;
                        let mut otag_count = 1u;
                        for item in list.slice_from(i + 1).iter() {
                            count += 1;
                            match *item {
                                OTag(title, inverted, raw) => {
                                    if title == name {
                                        otag_count += 1;
                                    }
                                    children.push(*item);
                                },
                                CTag(title, temp) => {
                                    if title == name && otag_count == 1 {
                                        nodes.push(Section(name, parse_nodes(&children).clone(), inverted, raw.to_string(), temp.to_string()));
                                        break;
                                    } else if title == name && otag_count > 1 {
                                        otag_count -= 1;
                                        children.push(*item);
                                    } else {
                                        children.push(*item);
                                        continue;
                                    }
                                },
                                _ => {
                                    children.push(*item);
                                    continue;
                                }
                            }
                        }

                        // Advance the iterator to the position of the CTAG.  If the 
                        //OTag is never closed, these children will never be processed.
                        while count > 1 {
                            it.next();
                            count -= 1;
                        }
                    },
                }
            },
            None => break
        }
    }

    nodes
}


#[cfg(test)]
mod parser_tests {
    use compiler::{Token, Text, Variable, OTag, CTag, Raw, Partial};
    use parser;
    use parser::{Node, Static, Value, Section, Unescaped, Part};

    #[test]
    fn parse_dot_notation() {
        let tokens: Vec<Token> = vec![Variable("section.child_tag", "{{ section.child_tag }}")];
        let nodes = parser::parse_nodes(&tokens);
        let expected: Vec<Node> = vec![Section("section", vec![Value("child_tag", "{{child_tag}}".to_string())], false, "{{#section}}".to_string(), "{{/section}}".to_string())];
        assert_eq!(nodes, expected);

        let tokens: Vec<Token> = vec![Raw("section.child_tag", "{{& section.child_tag }}")];
        let nodes = parser::parse_nodes(&tokens);
        let expected: Vec<Node> = vec![Section("section", vec![Unescaped("child_tag", "{{&child_tag}}".to_string())], false, "{{#section}}".to_string(), "{{/section}}".to_string())];
        assert_eq!(nodes, expected);
        
        let tokens: Vec<Token> = vec![Raw("section.child_tag", "{{{ section.child_tag }}}")];
        let nodes = parser::parse_nodes(&tokens);
        let expected: Vec<Node> = vec![Section("section", vec![Unescaped("child_tag", "{{{child_tag}}}".to_string())], false, "{{#section}}".to_string(), "{{/section}}".to_string())];
        assert_eq!(nodes, expected);
    }

    #[test]
    fn parse_static() {
        let tokens: Vec<Token> = vec![Text("Static String ")];
        let nodes = parser::parse_nodes(&tokens);
        let expected: Vec<Node> = vec![Static("Static String ")];
        assert_eq!(nodes, expected);
    }

    #[test]
    fn parse_value() {
        let tokens: Vec<Token> = vec![Variable("token", "{{ token }}")];
        let nodes = parser::parse_nodes(&tokens);
        let expected: Vec<Node> = vec![Value("token", "{{ token }}".to_string())];
        assert_eq!(nodes, expected);
    }

    #[test]
    fn parse_section() {
        let tokens: Vec<Token> = vec![OTag("section", false, "{{# section }}"), Variable("child_tag", "{{ child_tag }}"), CTag("section", "{{/ section }}")];
        let nodes = parser::parse_nodes(&tokens);
        let expected: Vec<Node> = vec![Section("section", vec![Value("child_tag", "{{ child_tag }}".to_string())], false, "{{# section }}".to_string(), "{{/ section }}".to_string())];
        assert_eq!(nodes, expected);
    }

    #[test]
    fn parse_inverted() {
        let tokens: Vec<Token> = vec![OTag("inverted", true, "{{^ inverted }}"), Variable("child_tag", "{{ child_tag }}"), CTag("inverted", "{{/ inverted }}")];
        let nodes = parser::parse_nodes(&tokens);
        let expected: Vec<Node> = vec![Section("inverted", vec![Value("child_tag", "{{ child_tag }}".to_string())], true, "{{^ inverted }}".to_string(), "{{/ inverted }}".to_string())];
        assert_eq!(nodes, expected);
    }

    #[test]
    fn parse_unescaped() {
        let tokens: Vec<Token> = vec![Raw("unescaped", "{{& unescaped }}")];
        let nodes = parser::parse_nodes(&tokens);
        let expected: Vec<Node> = vec![Unescaped("unescaped", "{{& unescaped }}".to_string())];
        assert_eq!(nodes, expected);
    }

    #[test]
    fn parse_partial() {
        let tokens: Vec<Token> = vec![Partial("new","{{> new }}")];
        let nodes = parser::parse_nodes(&tokens);
        let expected: Vec<Node> = vec![Part("new", "{{> new }}")];
        assert_eq!(nodes, expected);
    }

    #[test]
    fn parse_all() {
        let tokens: Vec<Token> = vec![
            Text("Static String "), Variable("token", "{{ token }}"), OTag("section", false, "{{# section }}"),
            Variable("child_tag", "{{ child_tag }}"), CTag("section", "{{/ section }}"),
            Partial("new","{{> new }}"), Raw("unescaped", "{{& unescaped }}")
        ];
        let nodes = parser::parse_nodes(&tokens);
        let static_node = Static("Static String ");
        let value_node = Value("token", "{{ token }}".to_string());
        let section_node = Section("section", vec![Value("child_tag", "{{ child_tag }}".to_string())], false, "{{# section }}".to_string(), "{{/ section }}".to_string());
        let file_node = Part("new", "{{> new }}");
        let undescaped_node = Unescaped("unescaped", "{{& unescaped }}".to_string());
        let expected: Vec<Node> = vec![static_node, value_node, section_node, file_node, undescaped_node];
        assert_eq!(nodes, expected);
    }
}
