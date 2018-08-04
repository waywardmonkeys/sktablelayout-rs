
#[macro_use]
extern crate bitflags;

use std::cmp::max;
use std::collections::BTreeMap;

/// Rectangle for padding and spacing constraints.
pub struct Rectangle {
    pub top:    f32,
    pub left:   f32,
    pub bottom: f32,
    pub right:  f32,
}

/// Individual size constraint for a cell.
pub struct Size {
    pub width:  f32,
    pub height: f32,
}

/// Combines the maximum, minimum and preferred sizes for a cell.
pub struct SizeGrouping {
    pub minimum:   Size,
    pub maximum:   Size,
    pub preferred: Size,
}

bitflags! {
    pub struct CellFlags: u16 {
        /// Expands cell to fill all remaining horizontal space.
        const ExpandHorizontal       = 0b0000000000000001;
        /// Expands cell to fill all remaining vertical space.
        const ExpandVertical         = 0b0000000000000010;
        /// Expands cell's contents to fill all remaining horizontal space.
        const FillHorizontal         = 0b0000000000000100;
        /// Expands cell's contents to fill all remaining vertical space.
        const FillVertical           = 0b0000000000001000;
        /// Anchors the cell to the top of its available space.
        const AnchorTop              = 0b0000000000010000;
        /// Anchors the cell to the bottom of its available space.
        const AnchorBottom           = 0b0000000000100000;
        /// Anchors the cell to the left of its available space.
        const AnchorLeft             = 0b0000000001000000;
        /// Anchors the cell to the right of its available space.
        const AnchorRight            = 0b0000000010000000;
        /// Anchors the cell to the center of its available space, horizontally.
        const AnchorHorizontalCenter = 0b0000000100000000;
        /// Anchors the cell to the center of its available space, vertically.
        const AnchorVerticalCenter   = 0b0000001000000000;
        /// Cell will be the same size as all cells which are uniform.
        const Uniform                = 0b0000010000000000;
    }
}

/// Encapsulates all properties for a cell; contributes to eventual layout decisions.
pub struct CellProperties {
    /// Controls the desired sizes for this cell.
    pub size: SizeGrouping,
    /// Controls various binary flags for the cell.
    pub flags: CellFlags,
    /// Controls how many columns this cell will occupy.
    pub colspan: u8,
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
}

impl TableLayout {
    /// Calculates the number of rows and columns which exist in this table layout.
    pub fn get_rows_cols(&self) -> (u8, u8) {
        let mut cols   = 0;
        let mut colcur = 0;
        let mut rows   = 0;

        for op in self.opcodes.iter() {
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
}

