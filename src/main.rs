fn normalise_string(string: &String) -> String {
    let mut str = String::new();
    for c in &mut string.chars() {
        if c.is_whitespace() {
            str.push('-');
            continue;
        }
        if c.is_ascii_alphabetic() {
            str.push(c.to_ascii_lowercase());
            continue;
        }
        if c.is_ascii_graphic() {
            str.push(c.to_ascii_lowercase());
        }
    }
    str
}

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

fn is_table (lines: Vec<&str>) -> bool {
    if lines.len() <= 1 {
        return false;
    }
    let mut header_count = 0;
    for c in lines[0].chars().skip(1) {
        if c == '|' {
            header_count += 1;
        }
    }
    let mut hyphen_count = 0;

    for c in lines[1].chars().skip(1) {
        if c == '-' {
            hyphen_count += 1;
        }
        if c == '|' {
            if hyphen_count != 3 {
                return false;
            }
            header_count -= 1;
            hyphen_count = 0;
        }
    }

    return header_count == 0;
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


fn parse_link(line: &str) -> (String, String, bool) {
    let line: Vec<char> = line.chars().collect();
    let mut text = String::new();
    let mut link = String::new();
    let mut writing_text = false;
    let mut writing_link = false;
    let mut did_write_text = false;
    let mut is_link = false;
    let mut escaped = false;

    for i in 0..line.len() {
        if line[i] == '\\' && !escaped {
            escaped = true;
            continue;
        }
        if line[i] == '[' && !escaped && !writing_text && !did_write_text{
            writing_text = true;
            continue;
        }
        if line[i] == ']' && !escaped && writing_text {
            writing_text = false;
            did_write_text = true;
            if i == line.len() - 1 && line[i+1] != '(' {
                break;
            }
        }
        if line[i] == '(' && !escaped && did_write_text && !writing_link {
            writing_link = true;
            is_link = true;
            continue;
        }
        if line[i] == ')' && !escaped && writing_link {
            break;
        }
        if writing_text {
            text.push(line[i]);
        } else if writing_link {
            link.push(line[i]);
        }
        escaped = false;
    }

    (text, link, is_link)

}

fn parse_paragraph(lines: &Vec<&str>, start_line: usize) -> ( Vec<String>, usize ) {
    let to_be_parsed = Vec::from( &lines[start_line..]);
    let mut parsed = vec!(String::from("<p>"));
    let mut italic = false;
    let mut bold = false;
    let mut code = false;
    let mut escape = false;
    let mut image = false;
    let mut skip = 0;
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

        if to_be_parsed[ind].starts_with("- ") {
            let mut ul = parse_ul(&to_be_parsed, ind);
            ul.0 = ul.0.iter().map(
                |x| format!("    {}", x)
            ).collect::<Vec<_>>();
            parsed.extend(ul.0);
            ind += ul.1;
            continue;
        }

        if is_ol(to_be_parsed[ind]) {
            let mut ol = parse_ol(&to_be_parsed, ind);
            ol.0 = ol.0.iter().map(
                |x| format!("    {}", x)
            ).collect::<Vec<_>>();
            parsed.extend(ol.0);
            ind += ol.1;
            continue;
        }

        for c in to_be_parsed[ind].chars() {
            if skip > 0 {
                skip -= 1;
                continue;
            }

            if c == '\\' && !escape {
                escape = true;
                continue;
            }
            if c == '_' && !escape {
                amount_of_underscores += 1;
                continue;
            } else if amount_of_underscores != 0 && !escape {
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
                    3=> {
                        bold = !bold;
                        italic = !italic;

                        if bold {
                            last.push_str("<strong>");
                        } else {
                            last.push_str("</strong>");
                        }
                        if !italic {
                            last.push_str("</em>");
                        } else {
                            last.push_str("<em>");
                        }
                    }
                    _=> {}
                }
                amount_of_underscores = 0;
                last.push(c);
                continue;
            }
            if c == '*' && !escape {
                amount_of_asterisks += 1;
                continue;
            } else if amount_of_asterisks != 0 && !escape {
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
                    3=> {
                        italic = !italic;
                        if !italic {
                            last.push_str("</em>");
                        } else {
                            last.push_str("<em>");
                        }

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

            if c == '!' && !escape {
                image = true;
                continue;
            }

            if c == '[' && !escape {
                let link = parse_link(to_be_parsed[ind]);
                if link.2 {
                    if image {
                        last.push_str(format!("<img src=\"{}\">", {link.1.clone()}).as_str());
                    } else {
                        last.push_str(format!("<a href=\"{}\">{}</a>", link.1.clone(), link.0.clone()).as_str());
                    }
                    skip += 4+link.0.len()+link.1.len();
                    continue;
                } else {
                    last.push('!')
                }
            }
            image = false;
            if (c != '_' && c != '*') || escape {
                escape = false;
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
        if italic {
            last.push_str("</em>");
        }
        if bold {
            last.push_str("</strong>");
        }
        ind += 1;
    }
    parsed.push("</p>".to_string());
    ( parsed, ind )
}

fn parse_table(lines: &Vec<&str>, start_line: usize) -> (Vec<String>, usize) {
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
    let mut ind = 2;
    while ind < to_be_parsed.len() && !to_be_parsed[ind].is_empty() {
        if to_be_parsed[ind].contains("|---|") {
            ind += 1;
            continue
        }
        parsed.push(String::from("        <tr>"));
        for c in to_be_parsed[ind].chars().skip(1) {
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
    (parsed, ind)
}

fn get_header(line: &str) -> (u8, bool, String){
    let mut hashtags: u8 = 0;
    let mut is_header = false;

    for c in line.chars() {
        if c == '#' {
            hashtags += 1;
        } else {
            if c == ' ' {
                is_header = true;
            }
            break;
        }
    }
    (hashtags, is_header, line[hashtags as usize+1..].to_string())
}


fn parse (lines: Vec<&str>, title: String) -> Vec<String> {
    let to_be_parsed = lines.clone();
    let mut parsed = vec![
        "<!DOCTYPE html>".to_string(),
        "<html>".to_string(),
        "    <head>".to_string(),
        format!("        <title>{title}</title>"),
        "        <meta charset=\"utf-8\">".to_string(),
        "        <link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">".to_string(),
        "        <link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin>".to_string(),
        "        <link href=\"https://fonts.googleapis.com/css2?family=Geologica:wght,CRSV@100..900,0&display=swap\" rel=\"stylesheet\">".to_string(),
        "        <link rel=\"stylesheet\" href=:::\"style.css\">".to_string(),
        "    </head>".to_string(),
        "    <body>".to_string()
    ];
    let mut body: Vec<String> = Vec::new();
    let mut i = 0;

    while i < to_be_parsed.len() {
        if to_be_parsed[i].trim().is_empty() {
            i += 1;
            continue;
        }

        if to_be_parsed[i].starts_with("|") {
            let mut end = i + 1;

            while end < to_be_parsed.len() && to_be_parsed[end].starts_with("|") {
                end += 1;
            }

            let table_lines = &to_be_parsed[i..end];

            if is_table(table_lines.to_vec()) {
                let table = parse_table(&table_lines.to_vec(), 0);
                body.extend(table.0);
                i = end;
                body.push(String::new());
                i += 1;
                continue;
            }
        } else if to_be_parsed[i].chars().all(|c| matches!(c, '_' | '-' | '*'))
            && to_be_parsed[i].len() >= 3
        {
            let mut is_up_clear = false;
            let mut is_down_clear = false;

            if i == 0 || to_be_parsed[i - 1].trim().is_empty() {
                is_up_clear = true;
            }

            if i == to_be_parsed.len() - 1 || to_be_parsed[i + 1].trim().is_empty() {
                is_down_clear = true;
            }

            if is_down_clear && is_up_clear {
                body.push("<hr>".to_string());
                i += 1;
                continue;
            }
        }

        body.push(String::new());

        if to_be_parsed[i].starts_with("#") {
            let header = get_header(to_be_parsed[i]);
            if header.1 {
                body[i] = format!(
                    "<h{} id=\"{}\">{}</h{}>",
                    header.0,
                    normalise_string(&header.2),
                    header.2,
                    header.0
                );
                i += 1;
                continue;
            }
        }





        let paragraph = parse_paragraph(&to_be_parsed.to_vec(), i);
        body.extend(paragraph.0);
        i += paragraph.1;
    }
    body = body.iter().map(|str| format!("        {}", str)).collect::<Vec<_>>();
    parsed.extend(body);
    parsed.push("    </body>".to_string());
    parsed.push("</html>".to_string());
    parsed
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut args: Vec<String> = std::env::args().collect();
    let options = ["-o", "-t", "c"];
    if args.contains(&String::from("-h")) || args.is_empty() {
        print!("\
        md2web by Chepyrka2, 2026\n\
        OPTIONS:\n\
        -h - help\n\
        -i - input file\n\
        -o - output directory\n\
        -t - title\n\
        -c - accent color (css value, like #0000FF or red)\n\
        ");

        return Ok(());
    }

    let mut input_file = String::new();
    let mut output_directory= String::from("md2web");
    let mut title = String::from("md2web");

    if let Some(input_file_index) = args.iter().position(|arg| arg == "-i") {
        if input_file_index == args.len()-1 {
            return Err("input file cannot be empty".into());
        }
        if args[input_file_index+1].starts_with("\"") {
            for arg in &args[input_file_index+1..] {
                if arg.starts_with("\"") {
                    input_file.push_str(&arg[1..])
                } else if arg.ends_with("\"") {
                    input_file = String::from(&input_file[0..input_file.len() - 2]);
                    break;
                } else {
                    input_file.push_str(arg);
                }
                input_file.push(' ');
            }
            for i in 0..input_file.chars().filter(|c| *c == ' ').count() {
                args.remove(input_file_index);
            }
        }

        else {
            input_file = args[input_file_index+1].clone();
        }
        args.remove(input_file_index);
        args.remove(input_file_index);
    } else {
        if args[0].starts_with("\"") {
            for arg in &args {
                if arg.starts_with("\"") {
                    input_file.push_str(&arg[1..])
                } else if arg.ends_with("\"") {
                    input_file.push_str(arg);
                    input_file = String::from(&input_file[0..input_file.len() - 2]);
                    break;
                } else {
                    input_file.push_str(arg);
                }
                input_file.push(' ');
            }
            for i in 0..input_file.chars().filter(|c| *c == ' ').count() {
                args.remove(0);
            }
        }
        else {
            input_file = args[0].clone();
        }
        args.remove(0);
        args.remove(0);
    }

    if let Some(output_directory_index) = args.iter().position(|arg| arg == "-o") {
        if output_directory_index == args.len()-1 {
            return Err("output directory wasn't provided".into());
        }
        if args[output_directory_index+1].starts_with("\"") {
            for arg in &args[output_directory_index+1..] {
                if arg.starts_with("\"") {
                    output_directory.push_str(&arg[1..]);
                } else if arg.ends_with("\"") {
                    output_directory.push_str(arg);
                    output_directory = String::from(&output_directory[0..output_directory.len()-2]);
                }
                output_directory.push(' ');
            }
            for i in 0..output_directory.chars().filter(|c| *c == ' ').count() {
                args.remove(output_directory_index);
            }
        }

        else {
            output_directory = args[output_directory_index+1].clone();
        }
        args.remove(output_directory_index);
        args.remove(output_directory_index);
    }

    if let Some(title_index) = args.iter().position(|arg| arg == "-t") {
        if title_index == args.len()-1 {
            return Err("title wasn't provided".into());
        }
        if args[title_index+1].starts_with("\"") {
        }
    }

    Ok(())
}
