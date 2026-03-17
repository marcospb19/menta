use std::{
    env,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use fs_err as fs;
use regex::Regex;
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, Weekday};

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

impl ContributionGrid {
    pub fn column_count(&self) -> usize {
        self.rows.iter().map(Vec::len).max().unwrap_or(0)
    }
}

fn contributions_json_cache_path() -> PathBuf {
    Path::new(&env::var("HOME").expect("HOME environment variable not set"))
        .join(".contributions.json")
}

pub fn load_contribution_grid(force_update: bool) -> ContributionGrid {
    let should_read_from_cache = 'should_read_from_cache: {
        if let Ok(metadata) = fs::metadata(contributions_json_cache_path())
            && !force_update
        {
            let modified = metadata
                .modified()
                .expect("Failed to read file modification time");
            let age = SystemTime::now()
                .duration_since(modified)
                .unwrap_or(Duration::MAX);

            if age < Duration::from_hours(2) {
                break 'should_read_from_cache true;
            }
        }

        break 'should_read_from_cache false;
    };

    let grid: ContributionGrid = if should_read_from_cache {
        let contents =
            fs::read_to_string(contributions_json_cache_path()).expect("Failed to read cache file");
        serde_json::from_str(&contents).expect("Failed to deserialize cache file")
    } else {
        fetch_grid_and_parse()
    };

    // also update the cache
    if !should_read_from_cache {
        let json = serde_json::to_string(&grid).expect("Failed to serialize contribution grid");
        fs::write(contributions_json_cache_path(), json).expect("Failed to write cache file");
    }

    trim_to_streak(remove_last_day_if_future(rotate_to_monday_start(grid)))
}

/// Remap the grid so weeks start on Monday instead of Sunday.
///
/// GitHub's grid has weeks as Sun..Sat columns. We want Mon..Sun columns.
/// - Sunday (github row 0, col C) moves to: new row 6, col C-1
///   (it becomes the last day of the *previous* Mon-start week)
/// - Mon..Sat (github rows 1..6, col C) move to: new row (github_row - 1), same col C
///
/// The very first Sunday (col 0) maps to col -1, so it's dropped.
fn rotate_to_monday_start(grid: ContributionGrid) -> ContributionGrid {
    let old_rows = &grid.rows;
    if old_rows.len() != 7 {
        return grid;
    }

    let old_num_cols = old_rows.iter().map(Vec::len).max().unwrap_or(0);
    if old_num_cols == 0 {
        return grid;
    }

    // Sunday row may have one more column than Mon-Sat rows (if today is Sunday).
    // After remapping, the new number of columns stays the same as old_num_cols,
    // because Sunday shifts left by 1 (losing first, but old_num_cols - 1 is the max
    // it reaches), and Mon-Sat keep their columns.
    let new_num_cols = old_num_cols;

    let mut new_rows: Vec<Vec<Option<u8>>> = vec![vec![None; new_num_cols]; 7];

    // Mon..Sat (github rows 1..6) -> new rows 0..5, same column
    for github_row in 1..7 {
        let new_row = github_row - 1;
        for col in 0..old_rows[github_row].len() {
            new_rows[new_row][col] = old_rows[github_row][col];
        }
    }

    // Sunday (github row 0) -> new row 6, column shifted left by 1
    for col in 1..old_rows[0].len() {
        new_rows[6][col - 1] = old_rows[0][col];
    }

    // After rotation this can happen
    if new_rows
        .iter()
        .all(|row| row.last().is_some_and(Option::is_none))
    {
        for row in &mut new_rows {
            row.pop();
        }
    }

    ContributionGrid { rows: new_rows }
}

/// Removes the last day from the grid if it represents "tomorrow" in local time.
///
/// GitHub may be ahead of the user's local timezone, showing a future day
/// with 0 contributions. This function checks the last day with data in the
/// grid and removes it if it's ahead of the current local day.
fn remove_last_day_if_future(grid: ContributionGrid) -> ContributionGrid {
    let num_cols = grid.rows.iter().map(Vec::len).max().unwrap_or(0);
    if num_cols == 0 || grid.rows.len() != 7 {
        return grid;
    }

    // Find the most recent day with data by scanning backwards
    let mut last_row = 0;
    let mut last_col = 0;
    let mut found = false;

    for col in (0..num_cols).rev() {
        for row in (0..7).rev() {
            if let Some(Some(_)) = grid.rows[row].get(col) {
                last_row = row;
                last_col = col;
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }

    if !found {
        return grid;
    }

    // Get local date and current day of week (0=Monday, 6=Sunday)
    let local_now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let today_row = match local_now.weekday() {
        Weekday::Monday => 0,
        Weekday::Tuesday => 1,
        Weekday::Wednesday => 2,
        Weekday::Thursday => 3,
        Weekday::Friday => 4,
        Weekday::Saturday => 5,
        Weekday::Sunday => 6,
    };

    // If the grid's last day is ahead of today's day (future), remove it
    if last_row > today_row {
        let last_value = grid.rows[last_row][last_col];
        assert_eq!(last_value, Some(0), "last day should be 0 when removing");
        let mut rows = grid.rows;
        rows[last_row][last_col] = None;
        return ContributionGrid { rows };
    }

    grid
}

/// Trim the grid to only show the current contribution streak plus 2 extra
/// weeks of padding before it.
///
/// Walks backwards from the most recent day with data, counting consecutive
/// days with contributions (level > 0). `None` cells (future/missing days) are
/// skipped. The streak breaks on the first `Some(0)`.
///
/// The grid is then trimmed so that the first visible column is
/// `max(0, streak_start_col - 2)`.
fn trim_to_streak(grid: ContributionGrid) -> ContributionGrid {
    let num_cols = grid.rows.iter().map(Vec::len).max().unwrap_or(0);
    if num_cols == 0 || grid.rows.len() != 7 {
        return grid;
    }

    // We iterate backwards through every (col, row) pair.
    // Within a week the day order is Mon(0)..Sun(6), so the latest day in a
    // column is row 6 and the earliest is row 0.  Walking backwards means we
    // go row 6, 5, 4, … 0 then move to the previous column at row 6.
    let mut col = num_cols - 2;
    let mut row: usize = 6;

    // Phase 1: skip None cells to find the most recent existing day.
    loop {
        let cell = grid.rows[row].get(col).copied().flatten();
        match cell {
            Some(_) => break, // found the most recent existing day
            None => {
                // move backwards
                if row == 0 {
                    if col == 0 {
                        // entire grid is None — nothing to trim
                        return grid;
                    }
                    col -= 1;
                    row = 6;
                } else {
                    row -= 1;
                }
            }
        }
    }

    // Initialize streak_start_col to the column of the most recent day.
    // If the streak is 0 (today is Some(0)), this is the correct fallback.
    let mut streak_start_col: usize = col;

    // Phase 2: walk backwards through existing days, counting the streak.
    // `col`/`row` currently point at the most recent existing day.
    loop {
        let cell = grid.rows[row].get(col).copied().flatten();
        match cell {
            Some(level) if level > 0 => {
                // streak continues
                streak_start_col = col;
            }
            Some(0) => {
                // streak broken — don't update streak_start_col,
                // it already points at the earliest streak day
                break;
            }
            None => {
                // skip non-existent days — they don't break the streak
            }
            _ => unreachable!(),
        }

        // move backwards
        if row == 0 {
            if col == 0 {
                // reached the very beginning of the grid
                streak_start_col = 0;
                break;
            }
            col -= 1;
            row = 6;
        } else {
            row -= 1;
        }
    }

    let start_col = streak_start_col.saturating_sub(7);

    let rows = grid
        .rows
        .into_iter()
        .map(|row| {
            if start_col < row.len() {
                row[start_col..].to_vec()
            } else {
                vec![]
            }
        })
        .collect();

    ContributionGrid { rows }
}

fn fetch_grid_and_parse() -> ContributionGrid {
    let html = reqwest::blocking::get(CONTRIBUTIONS_URL)
        .expect("Failed to fetch contributions page")
        .text()
        .expect("Failed to read response body");

    // Match each <td> tag that has the ContributionCalendar-day class.
    // The class attribute may appear anywhere within the tag, and other attributes may be in any
    // order.
    let td_re =
        Regex::new(r#"<td\b[^>]*\bclass="[^"]*ContributionCalendar-day[^"]*"[^>]*/?\s*>"#).unwrap();
    let id_re = Regex::new(r"contribution-day-component-(\d+)-(\d+)").unwrap();
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
