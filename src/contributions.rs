use std::{
    fs,
    time::{Duration, SystemTime},
};

use regex::Regex;
use serde::{Deserialize, Serialize};

const CACHE_FILE: &str = "graph.json";
const CACHE_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const CONTRIBUTIONS_URL: &str = "https://github.com/users/marcospb19/contributions";

/// 7 rows (Monday=0 to Sunday=6), each row has up to 53 columns.
/// Each cell is Option<u8> where:
///   - None = day hasn't happened yet (future) or doesn't exist
///   - Some(0) = no contributions
///   - Some(1..=4) = contribution levels
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContributionGrid {
    /// rows[day_of_week][week_index] = Option<level>
    pub rows: Vec<Vec<Option<u8>>>,
}

pub fn load_contribution_grid() -> ContributionGrid {
    if let Ok(metadata) = fs::metadata(CACHE_FILE) {
        let modified = metadata
            .modified()
            .expect("Failed to read file modification time");
        let age = SystemTime::now()
            .duration_since(modified)
            .unwrap_or(Duration::MAX);

        if age < CACHE_MAX_AGE {
            let contents = fs::read_to_string(CACHE_FILE).expect("Failed to read cache file");
            let grid: ContributionGrid =
                serde_json::from_str(&contents).expect("Failed to deserialize cache file");
            return rotate_to_monday_start(grid);
        }
    }

    let grid = fetch_and_parse();

    let json = serde_json::to_string(&grid).expect("Failed to serialize contribution grid");
    fs::write(CACHE_FILE, json).expect("Failed to write cache file");

    rotate_to_monday_start(grid)
}

/// Rotate rows so the week starts on Monday instead of Sunday.
/// GitHub returns: [Sunday, Monday, Tuesday, Wednesday, Thursday, Friday, Saturday]
/// We want:       [Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday]
fn rotate_to_monday_start(mut grid: ContributionGrid) -> ContributionGrid {
    grid.rows.rotate_left(1);
    grid
}

fn fetch_and_parse() -> ContributionGrid {
    let html = reqwest::blocking::get(CONTRIBUTIONS_URL)
        .expect("Failed to fetch contributions page")
        .text()
        .expect("Failed to read response body");

    // Match each <td> tag that has the ContributionCalendar-day class.
    // The class attribute may appear anywhere within the tag, and other attributes may be in any
    // order.
    let td_re =
        Regex::new(r#"<td\b[^>]*\bclass="[^"]*ContributionCalendar-day[^"]*"[^>]*/?\s*>"#).unwrap();
    let id_re = Regex::new(r#"contribution-day-component-(\d+)-(\d+)"#).unwrap();
    let level_re = Regex::new(r#"data-level="(\d)""#).unwrap();

    let mut max_col: usize = 0;
    let mut cells: Vec<(usize, usize, u8)> = Vec::new();

    for td_match in td_re.find_iter(&html) {
        let tag = td_match.as_str();

        let Some(id_caps) = id_re.captures(tag) else {
            continue;
        };
        let Some(level_caps) = level_re.captures(tag) else {
            continue;
        };

        let row: usize = id_caps[1].parse().expect("Failed to parse row index");
        let col: usize = id_caps[2].parse().expect("Failed to parse column index");
        let level: u8 = level_caps[1]
            .parse()
            .expect("Failed to parse contribution level");

        if col > max_col {
            max_col = col;
        }

        cells.push((row, col, level));
    }

    let num_cols = max_col + 1;
    let mut rows: Vec<Vec<Option<u8>>> = vec![vec![None; num_cols]; 7];

    for (row, col, level) in cells {
        rows[row][col] = Some(level);
    }

    ContributionGrid { rows }
}
