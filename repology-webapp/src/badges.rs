// SPDX-FileCopyrightText: Copyright 2024 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;

use repology_common::PackageStatus;

use crate::font::{FontMeasurer, FontStyle};
use crate::xmlwriter::{XmlTag, xml};

const HEADER_HEIGHT: usize = 25;
const HEADER_FONT_SIZE: usize = 15;
const CELL_HEIGHT: usize = 20;
const CELL_FONT_SIZE: usize = 11;
const CELL_HORIZONTAL_PADDING: usize = 5;
const FONT_FAMILY: &str = "DejaVu Sans,Verdana,Geneva,sans-serif";

// visual length is implied here, as we use it to account for ellipsis in truncated string
const ELLIPSIS_VISUAL_WIDTH_CHARS: usize = 1;

#[derive(Default)]
#[expect(dead_code)]
pub enum CellAlignment {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Default)]
pub struct Cell {
    // TODO: switch to Cow here to avoid allocation
    pub text: String,
    pub color: Option<String>,
    pub min_width: usize,
    pub collapsible: bool,
    pub alignment: CellAlignment,
}

impl Cell {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            ..Default::default()
        }
    }

    pub fn empty() -> Self {
        Self {
            text: String::new(),
            ..Default::default()
        }
    }

    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.to_owned());
        self
    }

    pub fn truncate(mut self, length: usize) -> Self {
        let char_len = self.text.chars().count();
        if char_len <= length {
            return self;
        }
        for (char_pos, (byte_pos, _)) in self.text.char_indices().enumerate() {
            if char_pos + ELLIPSIS_VISUAL_WIDTH_CHARS >= length {
                self.text.truncate(byte_pos);
                while self.text.ends_with('.') {
                    self.text.pop();
                }
                self.text += "…";
                return self;
            }
        }
        self
    }

    pub fn align(mut self, alignment: CellAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn min_width(mut self, width: usize) -> Self {
        self.min_width = width;
        self
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }
}

#[derive(Clone)]
struct Column {
    pub width: usize,
    pub is_collapsed: bool,
    pub offset: usize,
}

impl Default for Column {
    fn default() -> Self {
        Self {
            width: 0,
            is_collapsed: true,
            offset: 0,
        }
    }
}

#[rustfmt::skip::macros(xml)]
pub fn render_generic_badge(
    cells: &[Vec<Cell>],
    header: Option<&str>,
    min_width: usize,
    font_measurer: &FontMeasurer,
) -> Result<String> {
    let num_rows = cells.len();
    let num_columns = if num_rows > 0 { cells[0].len() } else { 0 };

    let mut columns = vec![Column::default(); num_columns];

    // calculate column widths
    for row in cells {
        for (cell, column) in row.iter().zip(columns.iter_mut()) {
            column.width = column
                .width
                .max(
                    font_measurer.get_text_width(&cell.text, CELL_FONT_SIZE, FontStyle::Regular)?
                        + CELL_HORIZONTAL_PADDING * 2,
                )
                .max(cell.min_width);
            if !cell.text.is_empty() || !cell.collapsible {
                column.is_collapsed = false;
            }
        }
    }

    // handle collapsed columns
    columns
        .iter_mut()
        .filter(|column| column.is_collapsed)
        .for_each(|column| column.width = 0);

    // add header if specified
    let (min_width, header_height) =
        if let Some(header) = header.filter(|header| !header.is_empty()) {
            (
                min_width.max(
                    font_measurer.get_text_width(header, HEADER_FONT_SIZE, FontStyle::Bold)?
                        + CELL_HORIZONTAL_PADDING * 2,
                ),
                HEADER_HEIGHT,
            )
        } else {
            (min_width, 0)
        };

    // calculate total sizes
    let total_height = header_height + CELL_HEIGHT * num_rows;
    let mut total_width = columns.iter().map(|column| column.width).sum::<usize>();

    // force minimal width by expanding leftmost column
    if total_width < min_width {
        if let Some(first_column) = columns.first_mut() {
            first_column.width += min_width - total_width;
        }
        total_width = min_width;
    }

    // calculate column offsets
    let mut offset = 0;
    columns.iter_mut().for_each(|column| {
        column.offset = offset;
        offset += column.width;
    });

    // start SVG document
    let mut doc = xml!("svg", "xmlns" = "http://www.w3.org/2000/svg", "width" = total_width, "height" = total_height);

    // define clip path for rounded corners
    doc.add_child(
        xml!("clipPath", "id" = "clip").with_child(
            xml!("rect", "rx" = 3, "width" = "100%", "height" = "100%", "fill" = "#000"),
        ),
    );

    // define linear gradient for bevel effect
    doc.add_child(
        xml!("linearGradient", "id" = "grad", "x2" = 0, "y2" = "100%")
            .with_child(xml!("stop", "offset" = 0, "stop-color" = "#bbb", "stop-opacity" = ".1"))
            .with_child(xml!("stop", "offset" = 1, "stop-opacity" = ".1")),
    );

    // graphical data
    let mut g = xml!("g", "clip-path" = "url(#clip)");

    // background
    g.add_child(xml!("rect", "width" = "100%", "height" = "100%", "fill" = "#555"));

    // header
    if let Some(header) = header {
        g.add_child(
            xml!("g", "fill" = "#fff", "text-anchor" = "middle", "font-family" = FONT_FAMILY, "font-size" = 15, "font-weight" = "bold")
                .with_child(
                    xml!("text", "x" = total_width / 2, "y" = HEADER_HEIGHT / 2 + 1, "dominant-baseline" = "central", "fill" = "#010101", "fill-opacity" = ".3")
                        .with_text(header),
                )
                .with_child(
                    xml!("text", "x" = total_width / 2, "y" = HEADER_HEIGHT / 2, "dominant-baseline" = "central")
                        .with_text(header),
                )
        );
    }

    // rows
    for (nrow, row) in cells.iter().enumerate() {
        let row_y_offset = header_height + nrow * CELL_HEIGHT;

        // cell backgrounds
        for (cell, column) in row.iter().zip(columns.iter()) {
            if let Some(color) = &cell.color {
                g.add_child(
                    xml!("rect", "x" = column.offset, "y" = row_y_offset, "width" = column.width, "height" = CELL_HEIGHT, "fill" = color)
                );
            }
        }

        // gradient
        g.add_child(
            xml!("rect", "y" = row_y_offset, "width" = "100%", "height" = CELL_HEIGHT, "fill" = "url(#grad)")
        );

        // cell texts
        let mut cell_text_g =
            xml!("g", "fill" = "#fff", "font-family" = FONT_FAMILY, "font-size" = CELL_FONT_SIZE);

        for (cell, column) in row.iter().zip(columns.iter()) {
            if cell.text.is_empty() || column.is_collapsed {
                continue;
            }

            let (text_x, text_anchor) = match cell.alignment {
                CellAlignment::Left => (column.offset + CELL_HORIZONTAL_PADDING, "start"),
                CellAlignment::Center => (column.offset + column.width / 2, "middle"),
                CellAlignment::Right => (
                    column.offset + column.width - CELL_HORIZONTAL_PADDING,
                    "end",
                ),
            };

            let text_y = row_y_offset + CELL_HEIGHT / 2;
            cell_text_g.add_child(
                xml!("text", "x" = text_x, "y" = text_y + 1, "text-anchor" = text_anchor, "dominant-baseline" = "central", "fill" = "#010101", "fill-opacity" = ".3")
                    .with_text(&cell.text)
            );

            cell_text_g.add_child(
                xml!("text", "x" = text_x, "y" = text_y, "text-anchor" = text_anchor, "dominant-baseline" = "central")
                    .with_text(&cell.text)
            );
        }

        g.add_child(cell_text_g);
    }

    doc.add_child(g);

    Ok(doc.to_string())
}

#[derive(PartialEq)]
pub enum SpecialVersionStatus {
    LowerThanUserGivenThreshold,
}

#[allow(unused)]
pub fn badge_color_for_package_status(
    package_status: PackageStatus,
    special_status: Option<SpecialVersionStatus>,
) -> &'static str {
    if let Some(special_status) = special_status {
        use SpecialVersionStatus::*;
        match special_status {
            LowerThanUserGivenThreshold => "#e00000",
        }
    } else {
        use PackageStatus::*;
        match package_status {
            Outdated | Legacy => "#e05d44",
            Newest | Unique | Devel => "#4c1",
            _ => "#9f9f9f",
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_truncate() {
        // will need update if ELLIPSIS_VISUAL_WIDTH_CHARS is changed
        assert_eq!(Cell::new("abcde").truncate(0).text, "…");
        assert_eq!(Cell::new("abcde").truncate(1).text, "…");
        assert_eq!(Cell::new("abcde").truncate(2).text, "a…");
        assert_eq!(Cell::new("abcde").truncate(3).text, "ab…");
        assert_eq!(Cell::new("abcde").truncate(4).text, "abc…");
        assert_eq!(Cell::new("abcde").truncate(5).text, "abcde");
        assert_eq!(Cell::new("abcde").truncate(6).text, "abcde");
    }

    #[test]
    fn test_cell_truncate_dot() {
        // don't leave handing dot before ellipsis
        assert_eq!(
            Cell::new("ab.cdefg")
                .truncate(3 + ELLIPSIS_VISUAL_WIDTH_CHARS)
                .text,
            "ab…"
        );
    }

    #[test]
    fn test_cell_truncate_unicode() {
        // make sure we are unicode aware and don't truncate mid-codepoint
        assert_eq!(
            Cell::new("абвгдеж")
                .truncate(3 + ELLIPSIS_VISUAL_WIDTH_CHARS)
                .text,
            "абв…"
        );
    }
}
