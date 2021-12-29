use color_eyre::eyre::{eyre, Result};

/// A simple date structure that only contains the year and month.
/// The components can be returned as strings. In the case of the
/// months they are returned as `MM - Month Name`.
#[derive(Debug)]
pub struct Date {
    year: u16,
    month: u8,
}

impl Date {
    pub fn new(year: u16, month: u8) -> Result<Date> {
        if month < 1 || month > 12 {
            return Err(eyre!(
                "invalid month, should be between 1 and 12 got {}",
                month
            ));
        }

        if year < 1839 || year > 3000 {
            return Err(eyre!(
                "invalid year, should be between 1839 and 3000 got {}",
                year
            ));
        }
        Ok(Date { year, month })
    }

    pub fn get_month(&self) -> String {
        match self.month {
            1 => String::from("01 - January"),
            2 => String::from("02 - February"),
            3 => String::from("03 - March"),
            4 => String::from("04 - April"),
            5 => String::from("05 - May"),
            6 => String::from("06 - June"),
            7 => String::from("07 - July"),
            8 => String::from("08 - August"),
            9 => String::from("09 - September"),
            10 => String::from("10 - October"),
            11 => String::from("11 - November"),
            12 => String::from("12 - December"),
            _ => String::from(""),
        }
    }

    pub fn get_year(&self) -> String {
        self.year.to_string()
    }
}
