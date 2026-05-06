use regex::Regex;

fn main() {
    let keyword_regex = Regex::new(r"(?i)@?(nomi|nom\s*nom|nomnom|nomiii|nom)\b").unwrap();
    let text = "@Nomi what is the time?";
    println!("{}", keyword_regex.replace_all(text, "").trim());
}
