
#[macro_use]
extern crate bitflags;

use std::f32;
use std::cmp::max;
use std::collections::BTreeMap;

/// Rectangle for padding and spacing constraints.
#[derive(Clone)]
pub struct Rectangle {
    pub top:    f32,
    pub left:   f32,
    pub bottom: f32,
    pub right:  f32,
}

/// Individual size constraint for a cell.
#[derive(Clone)]
pub struct Size {
    pub width:  f32,
    pub height: f32,
}

impl Size {
    pub fn join_max(a: &Size, b: &Size) -> Self {
        Size{
            width: f32::max(a.width, b.width),
            height: f32::max(a.height, b.height),
        }
    }

    pub fn join_min(a: &Size, b: &Size) -> Self {
        Size{
            width: f32::min(a.width, b.width),
            height: f32::min(a.height, b.height),
        }
    }

    /// Divides the width and height by a givin division level. Used when
    /// a size must be spread across multiple table cells.
    pub fn spread(&self, divisions: f32) -> Self {
        Size{
            width: self.width / divisions,
            height: self.height / divisions,
        }
    }

    /// Returns whether this size should fit within another size.
    pub fn within(&self, other: &Size) -> bool {
        if other.width > self.width && other.height > self.height {
            true
        } else {
            false
        }
    }
}

/// Combines the maximum, minimum and preferred sizes for a cell.
#[derive(Clone)]
pub struct SizeGrouping {
    pub minimum:   Size,
    pub maximum:   Size,
    pub preferred: Size,
}

impl Default for SizeGrouping {
    fn default() -> Self {
        SizeGrouping{
            minimum:   Size{width: 0.0, height: 0.0},
            preferred: Size{width: 0.0, height: 0.0},
            maximum:   Size{width: f32::MAX, height: f32::MAX},
        }
    }
}

impl SizeGrouping {
    pub fn join(a: &SizeGrouping, b: &SizeGrouping) -> SizeGrouping {
        SizeGrouping{
            minimum:   Size::join_max(&a.minimum,   &b.minimum),
            preferred: Size::join_max(&a.preferred, &b.preferred),
            maximum:   Size::join_min(&a.maximum,   &b.maximum),
        }
    }

    pub fn spread(&self, divisions: f32) -> SizeGrouping {
        SizeGrouping{
            minimum:   self.minimum.spread(divisions),
            preferred: self.preferred.spread(divisions),
            maximum:   self.maximum.spread(divisions),
        }
    }

    /// Attempts to fit an `item` of a given size within an `area`, subject
    /// to layout rules specified by `flags`. Returns the X, Y coordinates
    /// as well as width and height of the box fitted to the area.
    pub fn box_fit(&self, area: &Size, flags: &CellFlags) -> (f32, f32, f32, f32) {
        // combine maximum width and area width, depending on if fill has been actiated
        let w = if flags.contains(CellFlags::FillHorizontal) {
            f32::min(self.maximum.width, area.width)
        } else {
            self.preferred.width
        };

        // combine maximum height and area height, depending on if fill has been actiated
        let h = if flags.contains(CellFlags::FillVertical) {
            f32::min(self.maximum.height, area.height)
        } else {
            self.preferred.height
        };

        // find horizontal location of output box
        let x = if flags.contains(CellFlags::AnchorRight) {
            // take size of the area and remove width, will anchor us to the right side
            area.width - w
        } else if flags.contains(CellFlags::AnchorHorizontalCenter) {
            // tricky because we have to find the midpoint, then adjust by half of width
            (area.width / 2.0) - (w / 2.0)
        } else {
            // AnchorLeft is the same as doing nothing, so we just put this on the left side.
            0.0
        };

        // find vertical location of output box
        let y = if flags.contains(CellFlags::AnchorBottom) {
            // take size of the area and remove height, will anchor us to the top side
            area.height - h
        } else if flags.contains(CellFlags::AnchorHorizontalCenter) {
            // tricky because we have to find the midpoint, then adjust by half of height
            (area.height / 2.0) - (h / 2.0)
        } else {
            // AnchorTop is the same as doing nothing, so we just put this on the top side.
            0.0
        };

        (x, y, w, h)
    }
}

bitflags! {
    pub struct CellFlags: u16 {
        const None                   = 0b0000_0000_0000_0001;
        /// Expands cell to fill all remaining horizontal space.
        const ExpandHorizontal       = 0b0000_0000_0000_0001;
        /// Expands cell to fill all remaining vertical space.
        const ExpandVertical         = 0b0000_0000_0000_0010;
        /// Expands cell's contents to fill all remaining horizontal space.
        const FillHorizontal         = 0b0000_0000_0000_0100;
        /// Expands cell's contents to fill all remaining vertical space.
        const FillVertical           = 0b0000_0000_0000_1000;
        /// Anchors the cell to the top of its available space.
        const AnchorTop              = 0b0000_0000_0001_0000;
        /// Anchors the cell to the bottom of its available space.
        const AnchorBottom           = 0b0000_0000_0010_0000;
        /// Anchors the cell to the left of its available space.
        const AnchorLeft             = 0b0000_0000_0100_0000;
        /// Anchors the cell to the right of its available space.
        const AnchorRight            = 0b0000_0000_1000_0000;
        /// Anchors the cell to the center of its available space, horizontally.
        const AnchorHorizontalCenter = 0b0000_0001_0000_0000;
        /// Anchors the cell to the center of its available space, vertically.
        const AnchorVerticalCenter   = 0b0000_0010_0000_0000;
        /// Cell will be the same size as all cells which are uniform.
        const Uniform                = 0b0000_0100_0000_0000;
    }
}

pub type PositioningFn = FnMut(&(f32, f32, f32, f32));

/// Encapsulates all properties for a cell; contributes to eventual layout decisions.
pub struct CellProperties {
    /// Controls the desired sizes for this cell.
    pub size: SizeGrouping,
    /// Controls various binary flags for the cell.
    pub flags: CellFlags,
    /// Controls how many columns this cell will occupy.
    pub colspan: u8,
    /// Applies positioning updates for this cell. Note that this
    /// value always becomes `None` when cloned, so you cannot set
    /// default callbacks for cell policies.
    pub callback: Option<Box<PositioningFn>>
}

impl Default for CellProperties {
    fn default() -> Self {
        CellProperties{
            size: Default::default(),
            flags: CellFlags::None,
            colspan: 1,
            callback: None,
        }
    }
}

impl Clone for CellProperties {
    fn clone(&self) -> Self {
        CellProperties{
            size: self.size.clone(),
            flags: self.flags.clone(),
            colspan: self.colspan.clone(),
            callback: None,
        }
    }
}

pub enum LayoutOp {
    /// Inserts a cell in the resulting layout.
    Cell(CellProperties),
    /// Inserts a row break in the resulting layout.
    Row,
}

pub struct TableLayout {
    pub cell_defaults:   CellProperties,
    pub row_defaults:    BTreeMap<u8, CellProperties>,
    pub column_defaults: BTreeMap<u8, CellProperties>,
    pub opcodes:         Vec<LayoutOp>,

    pub row: u8,
    pub column: u8,
}

impl CellProperties {
    /// Inherits the default settings as determined by a
    /// `TableLayout`. Will first try to match the defaults for the
    /// column this would be added to, then the row, then the fallback
    /// defaults. Note that these defaults apply only if the cell
    /// was added next and if the defaults have not been changed
    /// since. The correct use of `with_defaults` is to initialize
    /// `CellProperties` for immediate insertion to a layout.
    pub fn with_defaults(layout: &TableLayout) -> Self {
        // try to get the column default
        let column_value = layout.column_defaults.get(&layout.column);
        if column_value.is_some() {
            return (*column_value.unwrap()).clone();
        }

        // try to get the row default
        let row_value = layout.row_defaults.get(&layout.row);
        if row_value.is_some() {
            return (*row_value.unwrap()).clone();
        }

        // just get the default i guess
        CellProperties{..layout.cell_defaults.clone()}
    }

    pub fn minimum_size(&mut self, minimum: Size) -> &mut Self {
        self.size.minimum = minimum;
        self
    }

    pub fn maximum_size(&mut self, maximum: Size) -> &mut Self {
        self.size.maximum = maximum;
        self
    }

    pub fn preferred_size(&mut self, preferred: Size) -> &mut Self {
        self.size.preferred = preferred;
        self
    }

    pub fn expand(&mut self) -> &mut Self {
        self.flags |= CellFlags::ExpandHorizontal | CellFlags::ExpandVertical;
        self
    }

    pub fn expand_horizontal(&mut self) -> &mut Self {
        self.flags |= CellFlags::ExpandHorizontal;
        self
    }

    pub fn expand_vertical(&mut self) -> &mut Self {
        self.flags |= CellFlags::ExpandVertical;
        self
    }

    pub fn fill(&mut self) -> &mut Self {
        self.flags |= CellFlags::FillHorizontal | CellFlags::FillVertical;
        self
    }

    pub fn fill_horizontal(&mut self) -> &mut Self {
        self.flags |= CellFlags::FillHorizontal;
        self
    }

    pub fn fill_vertical(&mut self) -> &mut Self {
        self.flags |= CellFlags::FillVertical;
        self
    }

    pub fn anchor_top(&mut self) -> &mut Self {
        self.flags |= CellFlags::AnchorTop;
        self
    }

    pub fn anchor_bottom(&mut self) -> &mut Self {
        self.flags |= CellFlags::AnchorBottom;
        self
    }

    pub fn anchor_left(&mut self) -> &mut Self {
        self.flags |= CellFlags::AnchorLeft;
        self
    }

    pub fn anchor_right(&mut self) -> &mut Self {
        self.flags |= CellFlags::AnchorRight;
        self
    }

    pub fn anchor_center(&mut self) -> &mut Self {
        self.flags |= CellFlags::AnchorHorizontalCenter | CellFlags::AnchorVerticalCenter;
        self
    }

    pub fn anchor_horizontal_center(&mut self) -> &mut Self {
        self.flags |= CellFlags::AnchorHorizontalCenter;
        self
    }

    pub fn anchor_vertical_center(&mut self) -> &mut Self {
        self.flags |= CellFlags::AnchorVerticalCenter;
        self
    }

    pub fn uniform(&mut self) -> &mut Self {
        self.flags |= CellFlags::Uniform;
        self
    }

    pub fn colspan(&mut self, span: u8) -> &mut Self {
        self.colspan = span;
        self
    }
}

impl TableLayout {
    /// Calculates the number of rows and columns which exist in this table layout.
    pub fn get_rows_cols(&self) -> (u8, u8) {
        let mut cols   = 0;
        let mut colcur = 0;
        let mut rows   = 0;

        for op in &self.opcodes {
            match op {
                LayoutOp::Cell(cp) => { cols += cp.colspan },
                LayoutOp::Row => { cols = max(cols, colcur); colcur = 0; rows += 1 },
            }
        }

        if colcur > 0 {
            cols = max(cols, colcur);
            rows += 1;
        }

        (rows, cols)
    }

    /// Removes all layout declarations from the table. Does not remove row or column defaults.
    pub fn clear(&mut self) {
        self.row = 0;
        self.column = 0;
        self.opcodes.clear()
    }

    /// Removes all layout declarations and resets ALL settings to factory default.
    pub fn full_clear(&mut self) {
        self.clear();
        self.row_defaults.clear();
        self.column_defaults.clear();
        self.cell_defaults = Default::default()
    }

    /// Adds a new row to the layout.
    pub fn add_row(&mut self) {
        self.opcodes.push(LayoutOp::Row);
        self.row += 1;
        self.column = 0
    }

    /// Hands the cell off to the layout.
    pub fn add_cell(&mut self, properties: CellProperties) {
        self.column += properties.colspan;
        self.opcodes.push(LayoutOp::Cell(properties))
    }

    pub fn impose(&mut self, width: f32, height: f32) {
        let mut row: u8 = 0;
        let mut col: u8 = 0;

        let (total_rows, total_cols) = self.get_rows_cols();

        let mut col_sizes: Vec<SizeGrouping> = Vec::with_capacity(total_cols as usize);
        // XXX resize_with is unstable, but would do what we want just fine
        for _i in 0..total_cols {
            col_sizes.push(Default::default());
        }

        // XXX resize_with is unstable, but would do what we want just fine
        let mut row_sizes: Vec<SizeGrouping> = Vec::with_capacity(total_cols as usize);
        for _i in 0..total_rows {
            row_sizes.push(Default::default());
        }

        // We determine size preferences for each column in the layout.
        for op in &self.opcodes {
            match op {
                LayoutOp::Cell(cp) => {
                    match cp.colspan {
                        // If a cell has a span of zero, that is kind of stupid and it basically doesn't exist.
                        0 => {},
                        // Single span column is easy; we just combine size preferences.
                        1 => {
                            row_sizes[row as usize] = SizeGrouping::join(&row_sizes[row as usize], &cp.size);
                            col_sizes[col as usize] = SizeGrouping::join(&col_sizes[col as usize], &cp.size);
                            col += 1;
                        }
                        // Multi column is derpy since we have to spread the constraints across each column.
                        _ => {
                            let midget = cp.size.spread(cp.colspan as f32);
                            row_sizes[row as usize] = SizeGrouping::join(&row_sizes[row as usize], &cp.size);
                            for _i in 0..cp.colspan {
                                col_sizes[col as usize] = SizeGrouping::join(&col_sizes[col as usize], &midget);
                                col += 1;
                            }
                        }
                    }
                }
                // flop to a new row
                LayoutOp::Row => {
                    row += 1;
                    col = 0;
                }
            }
        }

        let mut slack: Vec<f32> = Vec::new();

        // Calculate error along width distribution
        let mut error = width;
        for c in &col_sizes {
            // Error is what remains once we have given each column its preferred size.
            error -= c.preferred.width;
        }

        if error > 0.0 { // Extra space; relax the layout if we need to
            return; // TODO: done for now!
        } else if error < 0.0 { // Not enough space; tense up some more!
            // We need to find slack space for each column
            let mut total_slack: f32 = 0.0;
            slack.clear();
            slack.resize(total_cols as usize, 0.0);
            for (i, x) in col_sizes.iter().map(|x| x.preferred.width - x.minimum.width).enumerate() {
                slack[i] = x;
                total_slack += x;
            }

            // XXX if error > total_slack, it is impossible to solve this constraint

            // Spread error across slack space.
            for (i, x) in slack.iter().enumerate() {
                col_sizes[i].preferred.width =
                    f32::max(col_sizes[i].preferred.width + x * (col_sizes[i].preferred.width / total_slack), 0.0);
            }
        }

	// Calculate error along height distribution
	let mut error = height;
	for c in &row_sizes {
            // Error is what remains once we have given each row its preferred size.
            error -= c.preferred.height;
	}

        if error > 0.0 { // Extra space; relax the layout if we need to
            return; // TODO: done for now!
        } else if error < 0.0 { // Not enough space; tense up some more!
            // We need to find slack space for each column
            let mut total_slack: f32 = 0.0;
            slack.clear();
            slack.resize(total_rows as usize, 0.0);
            for (i, x) in row_sizes.iter().map(|x| x.preferred.height - x.minimum.height).enumerate() {
                slack[i] = x;
                total_slack += x;
            }

            // XXX if error > total_slack, it is impossible to solve this constraint

            // Spread error across slack space.
            for (i, x) in slack.iter().enumerate() {
                row_sizes[i].preferred.height =
                    f32::max(row_sizes[i].preferred.height + x * (row_sizes[i].preferred.height / total_slack), 0.0);
            }
        }

        // Preparations complete. Now we pass the news along to our client.

    }
}

