
extern crate sktablelayout;
use sktablelayout::*;

fn main() {
    let mut engine = TableLayout::new();
    engine.with_cell(CellProperties::new()
                    .callback(Box::new(|x, y, w, h| println!("{} {} {} {}", x, y, w, h)))
                    .anchor_right()
                    .preferred_size(Size{width: 64.0, height: 64.0}));
    engine.with_cell(CellProperties::new()
                    .callback(Box::new(|x, y, w, h| println!("{} {} {} {}", x, y, w, h)))
                    .anchor_right()
                    .expand_horizontal()
                    .preferred_size(Size{width: 64.0, height: 64.0}));
    engine.with_cell(CellProperties::new()
                    .callback(Box::new(|x, y, w, h| println!("{} {} {} {}", x, y, w, h)))
                    .anchor_right()
                    .expand_horizontal()
                    .fill_horizontal()
                    .preferred_size(Size{width: 64.0, height: 64.0}));
    engine.with_row();
    engine.with_cell(CellProperties::new()
                    .callback(Box::new(|x, y, w, h| println!("{} {} {} {}", x, y, w, h)))
                    .colspan(3)
                    .expand_vertical()
                    .anchor_bottom()
                    .fill_horizontal()
                    .preferred_size(Size{width: 64.0, height: 64.0}));
    engine.impose(320.0, 240.0);
}

