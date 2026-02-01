#![forbid(unsafe_code)]

//! Drawing primitives for the buffer.

use crate::buffer::Buffer;
use crate::cell::Cell;
use ftui_core::geometry::Rect;

/// Extension trait for drawing on a Buffer.
pub trait Draw {
    /// Draw a horizontal line.
    fn draw_horizontal_line(&mut self, x: u16, y: u16, width: u16, cell: Cell);

    /// Draw a vertical line.
    fn draw_vertical_line(&mut self, x: u16, y: u16, height: u16, cell: Cell);

    /// Draw a filled rectangle.
    fn draw_rect_filled(&mut self, rect: Rect, cell: Cell);

    /// Draw a rectangle outline (border).
    fn draw_rect_outline(&mut self, rect: Rect, cell: Cell);

    /// Print text at the given coordinates.
    fn print_text(&mut self, x: u16, y: u16, text: &str, cell: Cell);
}

impl Draw for Buffer {
    fn draw_horizontal_line(&mut self, x: u16, y: u16, width: u16, cell: Cell) {
        for i in 0..width {
            self.set(x + i, y, cell);
        }
    }

    fn draw_vertical_line(&mut self, x: u16, y: u16, height: u16, cell: Cell) {
        for i in 0..height {
            self.set(x, y + i, cell);
        }
    }

    fn draw_rect_filled(&mut self, rect: Rect, cell: Cell) {
        self.fill(rect, cell);
    }

    fn draw_rect_outline(&mut self, rect: Rect, cell: Cell) {
        if rect.is_empty() {
            return;
        }

        // Top
        self.draw_horizontal_line(rect.x, rect.y, rect.width, cell);

        // Bottom
        if rect.height > 1 {
            self.draw_horizontal_line(rect.x, rect.bottom() - 1, rect.width, cell);
        }

        // Left
        if rect.height > 2 {
            self.draw_vertical_line(rect.x, rect.y + 1, rect.height - 2, cell);
        }

        // Right
        if rect.width > 1 && rect.height > 2 {
            self.draw_vertical_line(rect.right() - 1, rect.y + 1, rect.height - 2, cell);
        }
    }

    fn print_text(&mut self, x: u16, y: u16, text: &str, base_cell: Cell) {
        let mut cx = x;
        for c in text.chars() {
            let mut cell = base_cell;
            // TODO: Handle width correctly using unicode-width
            // For now assume width 1 for simplicity in this stub, but Cell handles it?
            // Cell::from_char handles the content.
            // We should use set() which handles scissor.
            
            // In a real implementation, we need to handle wide chars.
            // The Cell::from_char(c) constructor (from Investigator) creates a cell.
            // If we have a 'base_cell' with style, we should merge.
            
            cell.content = crate::cell::CellContent::from_char(c);
            self.set(cx, y, cell);
            
            // Increment cx. Note: set() handles bounds check.
            cx += 1;
        }
    }
}
