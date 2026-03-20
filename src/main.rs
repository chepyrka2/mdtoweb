use std::fmt::format;

fn parse_table(lines: &Vec<&str>, start_line: usize) -> Vec<String> {
    let mut parsed = vec!(String::from("<table>"));
    let to_be_parsed = &lines[start_line..];
    parsed.push(String::from("  <thead>"));
    parsed.push(String::from("      <tr>"));
    let mut head = String::new();
    for c in to_be_parsed[0].chars().skip(1) {
        if c == '|' {
            parsed.push( format!("           <th>{head}</th>"));
            head = String::new();
            continue;
        }
        head.push(c);
    }
    parsed.push(String::from("      </tr>"));
    parsed.push(String::from("  </thead>"));
    parsed.push(String::from("  <tbody>"));
    head = String::new();
    let mut ind = 1;
    while ind * 2 < to_be_parsed.len() && !to_be_parsed[ind*2].is_empty() {
        parsed.push(String::from("      <tr>"));
        for c in to_be_parsed[ind*2].chars().skip(1) {
            if c == '|' {
                parsed.push(format!("          <td>{head}</td>"));
                head = String::new();
                continue;
            }
            head.push(c);
        }
        parsed.push(String::from("      </tr>"));
        ind += 1;
    }
    parsed.push(String::from("  </tbody>"));
    parsed.push(String::from("</table>"));
    parsed
}

fn main() {
    // let v = vec!("|Roblox|67|", "|---|---|", "|Hello|World|", "|---|---|");
    // let v = parse_table(&v, 0);
    // for line in &v {
    //     println!("{line}");
    // }
    let mut italic = false;
    let mut bold = false;
    let mut paragraph = false;
    let mut bitalic = false;
    let mut ol = false;
    let mut ul = false;
}
