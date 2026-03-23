fn strip_ordered_list_prefix(s: &str) -> Option<&str> {
    let mut chars = s.char_indices().peekable();

    let mut has_digit = false;
    let mut end = 0;

    while let Some((i, c)) = chars.peek() {
        if c.is_ascii_digit() {
            has_digit = true;
            end = i + c.len_utf8();
            chars.next();
        } else {
            break;
        }
    }

    if !has_digit {
        return None;
    }

    if let Some((i, '.')) = chars.peek() {
        end = i + 1;
        chars.next();
    } else {
        return None;
    }

    while let Some((i, c)) = chars.peek() {
        if c.is_whitespace() {
            end = i + c.len_utf8();
            chars.next();
        } else {
            break;
        }
    }

    Some(&s[end..])
}

fn is_ol(line: &str) -> bool {
    let mut chars = line.chars();
    let mut found_digit = false;

    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            found_digit = true;
        } else if c == '.' {
            return found_digit; // если перед точкой были цифры → true
        } else {
            return false; // встретили что-то не цифру и не точку
        }
    }

    false // дошли до конца строки без точки
}

fn parse_ol(lines: &Vec<&str>,  start_line: usize) -> (Vec<String>, usize) {
    let to_be_parsed = &lines[start_line..];
    let mut parsed = vec!(String::from("<ol>"));
    let mut ind = 0;

    while ind < to_be_parsed.len() && is_ol(to_be_parsed[ind]) {
        parsed.push(format!("    <li>{}</li>", strip_ordered_list_prefix(to_be_parsed[ind]).expect( format!("Cant strip the ol header! Line: {}", to_be_parsed[0]).as_str() )));
        ind += 1;
    }

    parsed.push(String::from("</ol>"));
    (parsed, ind)
}

fn parse_ul(lines: &Vec<&str>, start_line: usize) -> (Vec<String>, usize) {
    let to_be_parsed = &lines[start_line..];
    let mut parsed = vec!(String::from("<ul>"));
    let mut ind = 0;

    while ind < to_be_parsed.len() && to_be_parsed[ind].starts_with("- ") {
        let content = &to_be_parsed[ind][2..];
        parsed.push(format!("    <li>{content}</li>"));
        ind += 1;
    }

    parsed.push(String::from( "</ul>" ));
    (parsed, ind)
}


fn parse_paragraph(lines: &Vec<&str>, start_line: usize) -> Vec<String> {
    let to_be_parsed = Vec::from( &lines[start_line..]);
    let mut parsed = vec!(String::from("<p>"));
    let mut italic = false;
    let mut bold = false;
    let mut code = false;
    let mut ind = 0;
    let mut amount_of_underscores = 0;
    let mut amount_of_asterisks = 0;
    while ind < to_be_parsed.len() && !to_be_parsed[ind].is_empty() {
        let mut indent = if code {"".to_string()} else {"    ".to_string()};
        parsed.push(indent.clone());
        let last = parsed.last_mut().unwrap();

        if to_be_parsed[ind] == "```" {
            code = !code;
            indent = "    ".to_string();
            last.truncate(0);
            last.push_str(format!("{indent}").as_str());
            let what_to_push = if code { "<pre><code>" } else { "</code></pre>" };
            last.push_str(what_to_push);
            ind += 1;
            continue;
        }

        if to_be_parsed[ind].chars().next().expect( format!("Cant get first char! Problematic line: {}", to_be_parsed[ind]).as_str() )== '-' {
            let ul = parse_ul(&to_be_parsed, ind);
            parsed.extend(ul.0);
            ind += ul.1;
            continue;
        }

        if is_ol(to_be_parsed[ind]) {
            let mut ol = parse_ol(&to_be_parsed, ind);
            parsed.extend(ol.0);
            ind += ol.1;
            continue;
        }

        for c in to_be_parsed[ind].chars() {
            if c == '\\' && (amount_of_asterisks != 0 || amount_of_underscores != 0) {
                for _ in 0..amount_of_underscores {
                    last.push('_');
                    amount_of_underscores -= 1;
                }
                for _ in 0..amount_of_asterisks {
                    last.push('*');
                    amount_of_asterisks -= 1;
                }
                continue;
            }
            if c == '_' && amount_of_underscores < 2 {
                amount_of_underscores += 1;
                continue;
            } else if amount_of_underscores != 0 {
                match amount_of_underscores {
                    1=> {
                        italic = !italic;
                        if !italic {
                            last.push_str("</em>");
                        } else {
                            last.push_str("<em>");
                        }
                    }
                    2=> {
                        bold = !bold;
                        if bold {
                            last.push_str("<strong>");
                        } else {
                            last.push_str("</strong>");
                        }
                    }
                    _=> {}
                }
                amount_of_underscores = 0;
                last.push(c);
                continue;
            }
            if c == '*' && amount_of_asterisks < 2 {
                amount_of_asterisks += 1;
                continue;
            } else if amount_of_asterisks != 0 {
                match amount_of_asterisks {
                    1=> {
                        italic = !italic;
                        if !italic {
                            last.push_str("</em>");
                        } else {
                            last.push_str("<em>");
                        }
                    }
                    2=> {
                        bold = !bold;
                        if bold {
                            last.push_str("<strong>");
                        } else {
                            last.push_str("</strong>");
                        }
                    }
                    _=> {}
                }
                amount_of_asterisks = 0;
                last.push(c);
                continue;
            }

            if c != '_' && c != '*' {
                last.push(c);
                continue;
            }
        }
        if to_be_parsed[ind].ends_with("  ") {
            last.truncate(last.trim_end().len());
            last.push_str("<br>");
        }
        if to_be_parsed[ind].trim_start().starts_with("> ") {
            *last = format!("{}<blockquote>{}</blockquote>", indent, &to_be_parsed[ind].trim_start()[2..] );
        }
        ind += 1;
    }
    parsed.push("</p>".to_string());
    parsed
}

fn parse_table(lines: &Vec<&str>, start_line: usize) -> Vec<String> {
    let mut parsed = vec!(String::from("<table>"));
    let to_be_parsed = &lines[start_line..];
    parsed.push(String::from("    <thead>"));
    parsed.push(String::from("        <tr>"));
    let mut head = String::new();
    for c in to_be_parsed[0].chars().skip(1) {
        if c == '|' {
            parsed.push( format!("            <th>{head}</th>"));
            head = String::new();
            continue;
        }
        head.push(c);
    }
    parsed.push(String::from("        </tr>"));
    parsed.push(String::from("    </thead>"));
    parsed.push(String::from("    <tbody>"));
    head = String::new();
    let mut ind = 1;
    while ind * 2 < to_be_parsed.len() && !to_be_parsed[ind*2].is_empty() {
        parsed.push(String::from("        <tr>"));
        for c in to_be_parsed[ind*2].chars().skip(1) {
            if c == '|' {
                parsed.push(format!("            <td>{head}</td>"));
                head = String::new();
                continue;
            }
            head.push(c);
        }
        parsed.push(String::from("        </tr>"));

        ind += 1;
    }
    parsed.push(String::from("    </tbody>"));
    parsed.push(String::from("</table>"));
    parsed
}

fn main() {
    // let v = vec!("|Roblox|67|", "|---|---|", "|Hello|World|", "|---|---|");
    // let v = parse_table(&v, 0);
    // for line in &v {
    //     println!("{line}");
    // }
    let v = vec!("__sigma rizz__ is so *tuff* like", "```", "ohio на боге", "```", "and what alex 67 said", "> gooon", "67  ", "- goon", "- goon", "1. 67", "4. 41");
    let out = parse_paragraph(&v, 0);
    for line in out {
        println!("{}", line.as_str());
    }
}
