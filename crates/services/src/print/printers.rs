use std::process::Command;

use super::output::hidden;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Printers {
    pub names: Vec<String>,
    pub default: String,
}

pub fn list_printers() -> Printers {
    let mut printers = if cfg!(windows) {
        parse_windows(&output(
            "powershell.exe",
            &[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Get-CimInstance -ClassName Win32_Printer | \
                 ForEach-Object { \"$($_.Default)|$($_.Name)\" }",
            ],
        ))
    } else {
        parse_cups(&output("lpstat", &["-a"]), &output("lpstat", &["-d"]))
    };
    if printers.default.is_empty() {
        printers.default = printers.names.first().cloned().unwrap_or_default();
    }
    printers
}

fn output(program: &str, arguments: &[&str]) -> String {
    hidden(Command::new(program).args(arguments))
        .output()
        .map(|result| String::from_utf8_lossy(&result.stdout).into_owned())
        .unwrap_or_default()
}

fn parse_windows(text: &str) -> Printers {
    let mut printers = Printers::default();
    for line in text.lines() {
        let Some((flag, name)) = line.split_once('|') else {
            continue;
        };
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        printers.names.push(name.to_owned());
        if flag.trim().eq_ignore_ascii_case("true") {
            printers.default = name.to_owned();
        }
    }
    printers
}

fn parse_cups(listing: &str, current: &str) -> Printers {
    Printers {
        names: listing
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .map(str::to_owned)
            .collect(),
        default: current
            .split_once(':')
            .map(|(_, name)| name.trim().to_owned())
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_windows_printers() {
        let printers = parse_windows("False|PDF\nTrue|Caja 1\n\n|\n");
        assert_eq!(printers.names, vec!["PDF", "Caja 1"]);
        assert_eq!(printers.default, "Caja 1");
    }

    #[test]
    fn parses_cups_printers() {
        let printers = parse_cups("caja accepting\npdf accepting", "system default: pdf\n");
        assert_eq!(printers.names, vec!["caja", "pdf"]);
        assert_eq!(printers.default, "pdf");
    }
}
