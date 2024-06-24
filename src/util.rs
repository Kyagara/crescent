use crossterm::style::Stylize;

/// Use crossterm's styling to print a string in bold and cyan.
pub fn println_bold_cyan(title: &str) {
    println!("{}", title.bold().cyan());
}

/// Use crossterm's styling to print a field and its value, value will be in white.
pub fn println_field_value<T: std::fmt::Display>(name: &str, value: T) {
    println!("{}: {value}", name.white());
}

/// Format uptime in a human-readable format.
pub fn get_uptime_from_seconds(secs: u64) -> String {
    // Laughing while writing this
    // It works at least
    let check_plural = |number: u64, str: &str| {
        if number > 1 {
            return str.to_string() + "s";
        }
        str.to_string()
    };

    if secs <= 59 {
        return format!("{secs} {}", check_plural(secs, "second"));
    }

    let mins = secs / 60;

    if mins <= 59 {
        return format!("{mins} {}", check_plural(mins, "minute"));
    }

    let hours = mins / 60;

    if hours <= 23 {
        return format!("{hours} {}", check_plural(hours, "hour"));
    }

    let days = hours / 24;

    format!("{days} {}", check_plural(days, "day"))
}

#[cfg(test)]
mod tests {
    use super::get_uptime_from_seconds;

    #[test]
    fn unit_get_uptime_from_seconds() {
        let mut secs: u64 = 1;

        assert_eq!(get_uptime_from_seconds(secs), "1 second");
        secs = 30;
        assert_eq!(get_uptime_from_seconds(secs), "30 seconds");
        secs = 60;
        assert_eq!(get_uptime_from_seconds(secs), "1 minute");
        secs = 1800;
        assert_eq!(get_uptime_from_seconds(secs), "30 minutes");
        secs = 3600;
        assert_eq!(get_uptime_from_seconds(secs), "1 hour");
        secs = 43200;
        assert_eq!(get_uptime_from_seconds(secs), "12 hours");
        secs = 86400;
        assert_eq!(get_uptime_from_seconds(secs), "1 day");
        secs = 604_800;
        assert_eq!(get_uptime_from_seconds(secs), "7 days");
    }
}
