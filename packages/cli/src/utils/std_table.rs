use std::fmt::Display;

#[derive(Debug, Default, Clone, Copy, clap::ValueEnum)]
pub enum Format {
    #[default]
    Table,
    Simple,
    Csv,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format = match self {
            Format::Table => "table",
            Format::Simple => "simple",
            Format::Csv => "csv",
        };
        write!(f, "{}", format)
    }
}

impl Format {
    pub fn divider(&self) -> &str {
        match self {
            Format::Table => " | ",
            Format::Simple => " ",
            Format::Csv => ",",
        }
    }

    pub fn header_separator(&self) -> Option<&str> {
        match self {
            Format::Table => Some("-"),
            Format::Simple | Format::Csv => None,
        }
    }

    pub fn pad_enabled(&self) -> bool {
        match self {
            Format::Table | Format::Simple => true,
            Format::Csv => false,
        }
    }

    pub fn with_header(&self) -> bool {
        match self {
            Format::Table | Format::Csv => true,
            Format::Simple => false,
        }
    }
}

#[derive(Debug, Default)]
pub struct StdTable {
    headers: Vec<&'static str>,
    max_sizes: Vec<usize>,
    rows: Vec<Vec<String>>,
    format: Format,
}

impl Display for StdTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let divider = self.format.divider();
        let pad_enabled = self.format.pad_enabled();
        if self.format.with_header() {
            for (i, header) in self.headers.iter().enumerate() {
                let len = header.len();
                write!(
                    f,
                    "{header:>width$}{divider}",
                    width = if pad_enabled {
                        self.max_sizes.get(i).unwrap_or(&len)
                    } else {
                        &0
                    },
                    divider = if self.headers.len() - 1 != i {
                        divider
                    } else {
                        ""
                    }
                )?;
            }
            writeln!(f)?;
        }

        let max_row_len =
            self.max_sizes.iter().sum::<usize>() + (self.max_sizes.len() - 1) * divider.len();

        if let Some(sep) = self.format.header_separator() {
            writeln!(f, "{}", sep.repeat(max_row_len))?;
        }

        if self.rows.is_empty() {
            writeln!(f, "{:^width$}", "No content", width = max_row_len)?;
            return Ok(());
        }

        for row in self.rows.iter() {
            for (i, col) in row.iter().enumerate() {
                let len = col.len();
                write!(
                    f,
                    "{col:>width$}{divider}",
                    width = if pad_enabled {
                        self.max_sizes.get(i).unwrap_or(&len)
                    } else {
                        &0
                    },
                    divider = if row.len() - 1 != i { divider } else { "" }
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl StdTable {
    pub fn new(headers: Vec<&'static str>, format: Format) -> Self {
        Self {
            max_sizes: headers.iter().map(|h| h.len()).collect(),
            headers,
            format,
            ..Default::default()
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        for (i, col) in row.iter().enumerate() {
            match self.max_sizes.get_mut(i) {
                Some(entry) => *entry = *entry.max(&mut col.len()),
                None => self.max_sizes.push(col.len()),
            };
        }
        self.rows.push(row);
    }
}
