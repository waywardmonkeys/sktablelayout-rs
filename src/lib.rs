
#![feature(test)]
#[macro_use]
extern crate bitflags;

use std::f32;
use std::cmp::max;
use std::collections::BTreeMap;

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
        other.width > self.width && other.height > self.height
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
    pub fn box_fit(&self, area: &Size, flags: CellFlags) -> (f32, f32, f32, f32) {
        // combine maximum width and area width, depending on if fill has been actiated
        let w = if flags.contains(CellFlags::FillHorizontal) {
            f32::min(self.maximum.width, area.width)
        } else {
            f32::min(self.preferred.width, area.width)
        };

        // combine maximum height and area height, depending on if fill has been actiated
        let h = if flags.contains(CellFlags::FillVertical) {
            f32::min(self.maximum.height, area.height)
        } else {
            f32::min(self.preferred.height, area.height)
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
        const None                   = 0b0000_0000_0000_0000;
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

/// Allows a closure to ensure a layout item has been placed where the
/// layout engine decided it should go. The parameters are the `x`,
/// `y` coordinates, and the `width`/`height` respectively.
pub type PositioningFn = FnMut(f32, f32, f32, f32);

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
            flags: self.flags,
            colspan: self.colspan,
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
    pub fn new() -> Self {
        Default::default()
    }

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

    pub fn minimum_size(mut self, minimum: Size) -> Self {
        self.size.minimum = minimum;
        self
    }

    pub fn maximum_size(mut self, maximum: Size) -> Self {
        self.size.maximum = maximum;
        self
    }

    pub fn preferred_size(mut self, preferred: Size) -> Self {
        self.size.preferred = preferred;
        self
    }

    pub fn expand(mut self) -> Self {
        self.flags |= CellFlags::ExpandHorizontal | CellFlags::ExpandVertical;
        self
    }

    pub fn expand_horizontal(mut self) -> Self {
        self.flags |= CellFlags::ExpandHorizontal;
        self
    }

    pub fn expand_vertical(mut self) -> Self {
        self.flags |= CellFlags::ExpandVertical;
        self
    }

    pub fn fill(mut self) -> Self {
        self.flags |= CellFlags::FillHorizontal | CellFlags::FillVertical;
        self
    }

    pub fn fill_horizontal(mut self) -> Self {
        self.flags |= CellFlags::FillHorizontal;
        self
    }

    pub fn fill_vertical(mut self) -> Self {
        self.flags |= CellFlags::FillVertical;
        self
    }

    pub fn anchor_top(mut self) -> Self {
        self.flags |= CellFlags::AnchorTop;
        self
    }

    pub fn anchor_bottom(mut self) -> Self {
        self.flags |= CellFlags::AnchorBottom;
        self
    }

    pub fn anchor_left(mut self) -> Self {
        self.flags |= CellFlags::AnchorLeft;
        self
    }

    pub fn anchor_right(mut self) -> Self {
        self.flags |= CellFlags::AnchorRight;
        self
    }

    pub fn anchor_center(mut self) -> Self {
        self.flags |= CellFlags::AnchorHorizontalCenter | CellFlags::AnchorVerticalCenter;
        self
    }

    pub fn anchor_horizontal_center(mut self) -> Self {
        self.flags |= CellFlags::AnchorHorizontalCenter;
        self
    }

    pub fn anchor_vertical_center(mut self) -> Self {
        self.flags |= CellFlags::AnchorVerticalCenter;
        self
    }

    pub fn uniform(mut self) -> Self {
        self.flags |= CellFlags::Uniform;
        self
    }

    pub fn colspan(mut self, span: u8) -> Self {
        self.colspan = span;
        self
    }

    pub fn callback(mut self, fun: Box<PositioningFn>) -> Self {
        self.callback = Option::Some(fun);
        self
    }
}

impl TableLayout {
    pub fn new() -> TableLayout {
        TableLayout {
            cell_defaults:   Default::default(),
            row_defaults:    BTreeMap::new(),
            column_defaults: BTreeMap::new(),
            opcodes:         Vec::new(),
            row: 0,
            column: 0,
        }
    }

    /// Calculates the number of rows and columns which exist in this table layout.
    pub fn get_rows_cols(&self) -> (u8, u8) {
        let mut cols   = 0;
        let mut colcur = 0;
        let mut rows   = 0;

        for op in &self.opcodes {
            match op {
                LayoutOp::Cell(cp) => { colcur += cp.colspan },
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
    pub fn with_row(&mut self) -> &mut Self {
        self.opcodes.push(LayoutOp::Row);
        self.row += 1;
        self.column = 0;
        self
    }

    /// Hands the cell off to the layout.
    pub fn with_cell(&mut self, properties: CellProperties) -> &mut Self {
        self.column += properties.colspan;
        self.opcodes.push(LayoutOp::Cell(properties));
        self
    }

    pub fn impose(&mut self, width: f32, height: f32) {
        let mut row: u8 = 0;
        let mut col: u8 = 0;

        let (total_rows, total_cols) = self.get_rows_cols();
        if total_cols == 0 {return} // short-circuiting opportunity
        eprintln!("Imposing matrix: {}x{}", total_rows, total_cols);

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

        let mut has_xexpand: Vec<bool> = Vec::with_capacity(total_cols as usize);
        for _i in 0..total_cols {
            has_xexpand.push(false);
        }

        let mut has_yexpand: Vec<bool> = Vec::with_capacity(total_rows as usize);
        for _i in 0..total_rows {
            has_yexpand.push(false);
        }

        // We determine size preferences for each column in the layout.
        for op in &self.opcodes {
            match op {
                LayoutOp::Cell(cp) => {
                    match cp.colspan {
                        // If a cell has a span of zero, that is kind of stupid and it basically doesn't exist.
                        0 => {},
                        _ => {
                            let midget = cp.size.spread(f32::from(cp.colspan));
                            eprintln!("{:#?}", cp.flags);
                            row_sizes[row as usize] =
                                SizeGrouping::join(&row_sizes[row as usize], &cp.size);
                            if cp.flags.contains(CellFlags::ExpandVertical) {
                                eprintln!("flagging row {} for x-expansion", row);
                                has_yexpand[row as usize] = true
                            }
                            for _i in 0..cp.colspan {
                                if cp.flags.contains(CellFlags::ExpandHorizontal) {
                                    eprintln!("flagging col {} for x-expansion", col);
                                    has_xexpand[col as usize] = true
                                }
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
            // Figure out how many columns are expanding horizontally.
            let expansions = has_xexpand.iter().filter(|x| **x).count();
            if expansions > 0 {
                let amount = error / expansions as f32;
                for (i, e) in has_xexpand.iter().enumerate() {
                    eprintln!("Expanding column {} = {}", i, e);
                    if *e {
                        col_sizes[i].preferred.width += amount;
                    }
                }
            }
        } else if error < 0.0 { // Not enough space; tense up some more!
            let error = -error;
            eprintln!("Error {}", error);
            // We need to find slack space for each column
            let mut total_slack: f32 = 0.0;
            slack.clear();
            slack.resize(total_cols as usize, 0.0);
            for (i, x) in col_sizes.iter().map(|x| x.preferred.width - x.minimum.width).enumerate() {
                slack[i] = x;
                total_slack += x;
            }
            eprintln!("Total width slack: {}", total_slack);

            // XXX if error > total_slack, it is impossible to solve this constraint
            // spread error across slack space, proportionate to this areas slack participation
            for mut s in &mut slack {
                let norm = *s / total_slack;
                let error_over_slack = error * norm;
                eprintln!("slack contribution {}", norm);
                eprintln!("error over slack {}", error_over_slack);
                *s -= error_over_slack
            }

            // Spread error across slack space.
            for (i, x) in slack.iter().enumerate() {
                col_sizes[i].preferred.width =
                    f32::max(col_sizes[i].minimum.width + *x, 0.0);
            }
        }

	// Calculate error along height distribution
	let mut error = height;
	for c in &row_sizes {
            // Error is what remains once we have given each row its preferred size.
            error -= c.preferred.height;
	}

        if error > 0.0 { // Extra space; relax the layout if we need to
            // Figure out how many columns are expanding horizontally.
            let expansions = has_yexpand.iter().filter(|y| **y).count();
            if expansions > 0 {
                let amount = error / expansions as f32;
                for (i, e) in has_yexpand.iter().enumerate() {
                    eprintln!("Expanding row {} = {}", i, e);
                    if *e {
                        row_sizes[i].preferred.height += amount;
                    }
                }
            }
        } else if error < 0.0 { // Not enough space; tense up some more!
            let error = -error;
            eprintln!("Error {}", error);
            // We need to find slack space for each row
            let mut total_slack: f32 = 0.0;
            slack.clear();
            slack.resize(total_rows as usize, 0.0);
            for (i, y) in row_sizes.iter().map(|y| y.preferred.height - y.minimum.height).enumerate() {
                slack[i] = y;
                total_slack += y;
            }
            eprintln!("Total height slack: {}", total_slack);

            // XXX if error > total_slack, it is impossible to solve this constraint
            // spread error across slack space, proportionate to this areas slack participation
            for mut s in &mut slack {
                let norm = *s / total_slack;
                let error_over_slack = error * norm;
                eprintln!("slack contribution {}", norm);
                eprintln!("error over slack {}", error_over_slack);
                *s -= error_over_slack
            }

            // Spread error across slack space.
            for (i, y) in slack.iter().enumerate() {
                row_sizes[i].preferred.height =
                    f32::max(row_sizes[i].minimum.height + *y, 0.0);
            }
        }

        // Preparations complete. Now we pass the news along to our client.
        let mut x = 0.0;
        let mut y = 0.0;
        row = 0;
        col = 0;
        for mut op in &mut self.opcodes {
            // NB can probably make this mutable, and update it only when the row changes
            let height = row_sizes[row as usize].preferred.height;
            match op {
                // Something that needs to be placed.
                LayoutOp::Cell(cp) => match &cp.colspan {
                    0 => {}, // Ignore this cell.
                    _ => {
                        let mut width: f32 = 0.0;
                        for _i in 0..cp.colspan {
                            width += col_sizes[col as usize].preferred.width;
                            col += 1;
                        }
                        let s = Size{width, height};
                        let (bx, by, bw, bh) = cp.size.box_fit(&s, cp.flags);

                        // Run callback to impose layout.
                        match &mut cp.callback {
                            Some(cb) => {
                                (*cb)(x+bx, y+by, bw, bh);
                            }
                            None => {},
                        }

                        x += width;
                    }
                },
                // Increment to next row; reset placement cursors.
                LayoutOp::Row => {
                    x = 0.0;
                    y += height;
                    row += 1;
                    col = 0;
                }
            }
        }
    }
}


#[cfg(test)]
mod test {
    extern crate test;

    use ::*;
    #[test]
    fn expanding_layout() {
        let mut engine = TableLayout::new();
        engine.with_cell(CellProperties::new()
                        .callback(Box::new(|x, y, w, h| {
                            println!("{} {} {} {}", x, y, w, h);
                            assert_eq!(x, 0.0);
                            assert_eq!(y, 0.0);
                            // these are expand, not fill, so the
                            // cell takes up extra space but the
                            // child item actually doesn't use it
                            assert_eq!(w, 64.0);
                            assert_eq!(h, 64.0);
                        }))
                        .anchor_right()
                        .anchor_bottom()
                        .preferred_size(Size{width: 64.0, height: 64.0}));
        engine.with_cell(CellProperties::new()
                        .callback(Box::new(|x, y, w, h| {
                            println!("{} {} {} {}", x, y, w, h);
                            // first column is neither expand nor
                            // fill horizontal, so should be packed
                            // as preferred width
                            assert_eq!(x, 64.0);
                            assert_eq!(y, 0.0);
                            // these are expand, not fill, so the
                            // cell takes up extra space but the
                            // child item actually doesn't use it
                            assert_eq!(w, 64.0);
                            assert_eq!(h, 64.0);
                        }))
                        .anchor_top()
                        .anchor_left()
                        .expand_horizontal()
                        .preferred_size(Size{width: 64.0, height: 64.0}));
        engine.with_cell(CellProperties::new()
                        .callback(Box::new(|x, y, w, h| {
                            println!("{} {} {} {}", x, y, w, h);
                            assert_eq!(y, 0.0);
                            assert_eq!(h, 64.0);
                        }))
                        .anchor_right()
                        .expand_horizontal()
                        .fill_horizontal()
                        .preferred_size(Size{width: 64.0, height: 64.0}));
        engine.with_row();
        engine.with_cell(CellProperties::new()
                        .callback(Box::new(|x, y, w, h| {
                            println!("{} {} {} {}", x, y, w, h);
                            assert_eq!(x, 0.0);
                            assert_eq!(y, 240.0 - 64.0);
                            assert_eq!(w, 320.0);
                        }))
                        .colspan(3)
                        .expand_vertical()
                        .anchor_bottom()
                        .fill_horizontal()
                        .preferred_size(Size{width: 64.0, height: 64.0}));
        engine.impose(320.0, 240.0);
    }

    #[test]
    fn shrinking_layout() {
        let mut engine = TableLayout::new();
        engine.with_cell(CellProperties::new()
                        .callback(Box::new(|x, y, w, h| {
                            println!("{} {} {} {}", x, y, w, h);
                            assert_eq!(x, 0.0);
                            assert_eq!(y, 0.0);
                            assert_eq!(w, 16.0);
                            assert_eq!(h, 16.0);
                        }))
                        .preferred_size(Size{width: 64.0, height: 64.0}));
        engine.with_cell(CellProperties::new()
                        .callback(Box::new(|x, y, w, h| {
                            println!("{} {} {} {}", x, y, w, h);
                            assert_eq!(x, 16.0);
                            assert_eq!(y, 0.0);
                            assert_eq!(w, 16.0);
                            assert_eq!(h, 16.0);
                        }))
                        .preferred_size(Size{width: 64.0, height: 64.0}));
        engine.with_row();
        engine.with_cell(CellProperties::new()
                        .callback(Box::new(|x, y, w, h| {
                            println!("{} {} {} {}", x, y, w, h);
                            assert_eq!(x, 0.0);
                            assert_eq!(y, 16.0);
                            assert_eq!(w, 32.0);
                            assert_eq!(h, 16.0);
                        }))
                        .colspan(2)
                        .preferred_size(Size{width: 64.0, height: 64.0}));
        engine.impose(32.0, 32.0);
    }

    #[test]
    fn centered_layout() {
        let mut engine = TableLayout::new();
        engine.with_cell(CellProperties::new()
                        .callback(Box::new(|x, y, w, h| {
                            println!("{} {} {} {}", x, y, w, h);
                            assert_eq!(x, 16.0);
                            assert_eq!(y, 16.0);
                            assert_eq!(w, 32.0);
                            assert_eq!(h, 32.0);
                        }))
                        .anchor_horizontal_center()
                        .anchor_vertical_center()
                        .expand()
                        .preferred_size(Size{width: 32.0, height: 32.0}));
        engine.impose(64.0, 64.0);
    }

    #[bench]
    fn impose2x3(b: &mut test::Bencher) {
        // We only test the speed of layout calculation here, not
        // the overhead of communicating the layout back to the client.
        let mut engine = TableLayout::new();
        engine.with_cell(CellProperties::new()
                        .anchor_right()
                        .anchor_bottom()
                        .preferred_size(Size{width: 64.0, height: 64.0}));
        engine.with_cell(CellProperties::new()
                        .anchor_top()
                        .anchor_left()
                        .expand_horizontal()
                        .preferred_size(Size{width: 64.0, height: 64.0}));
        engine.with_cell(CellProperties::new()
                        .anchor_right()
                        .expand_horizontal()
                        .fill_horizontal()
                        .preferred_size(Size{width: 64.0, height: 64.0}));
        engine.with_row();
        engine.with_cell(CellProperties::new()
                        .colspan(3)
                        .expand_vertical()
                        .anchor_bottom()
                        .fill_horizontal()
                        .preferred_size(Size{width: 64.0, height: 64.0}));
        b.iter(|| engine.impose(test::black_box(320.0), test::black_box(240.0)))
    }
}

