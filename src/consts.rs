use crate::Column;

// The name of the app
pub const APP_NAME: &str = "fx";
// The number of lines excluding the file list
pub const MARGIN: usize = 8;
// The offset for navigation up/down
pub const PADDING: usize = 2;
// The default visible columns
pub const COLUMNS: [Column; 4] = [Column::Name, Column::Type, Column::Size, Column::Created];
