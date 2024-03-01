use std::fmt::Display;

#[derive(Debug, Default)]
pub struct StdTable {
    headers: Vec<&'static str>,
    max_sizes: Vec<usize>,
    rows: Vec<Vec<String>>,
}

impl Display for StdTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, header) in self.headers.iter().enumerate() {
            let len = header.len();
            write!(
                f,
                "{header:>width$}{leader}",
                width = self.max_sizes.get(i).unwrap_or(&len),
                leader = if self.headers.len() - 1 != i {
                    " | "
                } else {
                    ""
                }
            )?;
        }
        writeln!(f)?;

        let max_row_len = self.max_sizes.iter().sum::<usize>() + (self.max_sizes.len() - 1) * 3;
        writeln!(f, "{}", "-".repeat(max_row_len))?;

        if self.rows.is_empty() {
            writeln!(f, "{:^width$}", "No content", width = max_row_len)?;
            return Ok(());
        }

        for row in self.rows.iter() {
            for (i, col) in row.iter().enumerate() {
                let len = col.len();
                write!(
                    f,
                    "{col:>width$}{leader}",
                    width = self.max_sizes.get(i).unwrap_or(&len),
                    leader = if row.len() - 1 != i { " | " } else { "" }
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl StdTable {
    pub fn new(headers: Vec<&'static str>) -> Self {
        Self {
            max_sizes: headers.iter().map(|h| h.len()).collect(),
            headers,
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
