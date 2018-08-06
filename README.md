# Table Layout
A framework-free table-based layout system, written in pure Rust. You feed in constraints for the desired layout and provide a boxed closure to realize the layout on a given layout element. Once you need to place everything, you call `impose` with the dimensions of the layout object. Based around Esoteric Software's *tablelayout* package, and maybe someday MIGLayout, but is built purely from the public specification and not from the source in any form.

Closures are given the `x`, `y`, `width` and `height` of the layout item. These are relevant to *that item within the table* and do not include any translations that might be applied to the table itself. This means you need to offset `x` and `y` if the table is not placed at `(0, 0)`.

Currently no `unsafe` blocks are used by the engine.

# Layout
Create cells with `CellProperties::new`, then populate them by using the builder pattern. If you wish to use cell, row or column defaults in the layout, use `CellProperties::with_defaults`.

## Expansion
Cells may expand either vertically or horizontally. Expansion means that if there is space left over after all cells receive their preferred size, extra space is distributed to rows and columns with an expand style set.

`.expand_horizontal`, `.expand_vertical` and `.expand` set these policies.

## Fill
If a column or row is made larger than expected (due to expansion rules of other cells in the same column or row), this leaves extra usable space within other cells. By default this space is wasted and the layout elements will be placed in this white space according to anchoring rules. A *fill* says that should extra space become available somehow, that space will be claimed. A fill is not an expand, it will not *cause* extra space to be used. Only space that serendipitously became available is claimed by a fill.

`.fill_horizontal`, `.fill_vertical` and `.fill` set these policies.

Note that fills respect maximum sizes of layout items. Should an element reach its maximum size and fill space is available, the layout item is subject to anchoring rules (albeit at its larger size.)

## Anchor
When a layout item is for any reason smaller than its available space, an anchor defines where it will be located within that white space.

`.anchor_top`, `.anchor_bottom`, and `.anchor_vertical_center` handle sticking layout items along the vertical domain.

`.anchor_left`, `.anchor_right`, and `.anchor_horizontal_center` handle sticking layout items along the horizontal domain.

Conflicting anchor specifications are not an error, but the layout engine is free to ignore conflicting requests as it sees fit.

## Uniform

All cells that are set uniform will have the same size. In practice, this policy is not actually implemented right now.

# Internals
You should use the builder pattern to prepare layouts and cells. Tampering with the internals directly is not advised (and they might be made non-public in a more stable version.)

Bit flags for centered placement might be removed; one can argue that attempting to anchor to ex. the left and right side simultaneously *is identical* to center anchoring, which frees up two flags for use elsewhere.

# Performance

A benchmark is provided (`cargo bench`) to test layout calculations. On my *AMD FX(tm)-6300 Six-Core Processor*:

```
test test::impose2x3 ... bench:       8,468 ns/iter (+/- 790)
```

Note that imposing layouts does *not* rely on multi-threading and only needs to be done when the table's width or height is disturbed.

# Reference
https://github.com/EsotericSoftware/tablelayout