use egui::{Color32, Context, Id, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui};

use crate::consts::{BOARD_HEIGHT, BOARD_WIDTH};

/// The size a piece takes up
const PIECE_RADIUS: f32 = 37.5;
const PIECE_SPACING: f32 = 90.0;

/// The states a piece can be in
#[derive(Default, Clone, Copy)]
pub enum PieceState {
    #[default]
    Empty,
    PlayerOne,
    PlayerTwo,
}

impl PieceState {
    pub fn reverse(&self) -> PieceState {
        match self {
            PieceState::Empty => panic!("Tried to reverse an empty piece"),
            PieceState::PlayerOne => PieceState::PlayerTwo,
            PieceState::PlayerTwo => PieceState::PlayerOne,
        }
    }
}

/// Represents a piece on the game board
#[derive(Default)]
struct Piece {
    state: PieceState,

    /// The top left corner of the piece
    position: Pos2,
}

impl Piece {
    /// Visually displays the piece
    fn render(&self, ui: &mut Ui) {
        let (color, accent_color) = match self.state {
            PieceState::Empty => (Color32::BLACK, Color32::BLACK),
            PieceState::PlayerOne => (Color32::RED, Color32::DARK_RED),
            PieceState::PlayerTwo => (Color32::BLUE, Color32::DARK_BLUE),
        };

        let painter = ui.painter();

        let half_spacing = PIECE_SPACING / 2.0;
        let center = Pos2 {
            x: self.position.x + half_spacing,
            y: self.position.y + half_spacing,
        };
        painter.circle_filled(center, PIECE_RADIUS, color);

        let accent_radius = PIECE_RADIUS * 2.0 / 3.0;
        let accent_width = PIECE_RADIUS / 6.0;
        painter.circle_stroke(
            center,
            accent_radius,
            Stroke {
                width: accent_width,
                color: accent_color,
            },
        );
    }
}

struct Column {
    pieces: [Piece; BOARD_HEIGHT as usize],
    id: Id,
    rect: Rect,
    height: usize,
}

impl Column {
    /// Creates a column, starting from a position
    fn new(id: Id, position: Pos2) -> Column {
        let mut pieces: [Piece; BOARD_HEIGHT as usize] = Default::default();

        let mut piece_pos = position;
        for i in 0..(BOARD_HEIGHT as usize) {
            pieces[i] = Piece {
                state: PieceState::Empty,
                position: piece_pos,
            };

            piece_pos.y += PIECE_SPACING;
        }

        Column {
            id,
            pieces,
            rect: Rect {
                min: position,
                max: Pos2 {
                    x: position.x + PIECE_SPACING,
                    y: position.y + PIECE_SPACING * (BOARD_HEIGHT as f32),
                },
            },
            height: 0,
        }
    }

    fn render(&self, ui: &mut Ui) -> Response {
        for piece in self.pieces.iter() {
            piece.render(ui);
        }

        ui.interact(self.rect, self.id, Sense::click().union(Sense::hover()))
    }
}

impl Default for Column {
    fn default() -> Self {
        Self {
            pieces: Default::default(),
            id: Id::new(""),
            rect: Rect {
                min: Pos2::default(),
                max: Pos2::default(),
            },
            height: 0,
        }
    }
}

#[derive(Hash)]
struct ColumnId {
    board_id: Id,
    index: usize,
}

pub struct Board {
    columns: [Column; BOARD_WIDTH as usize],
    id: Id,
    rect: Rect,
    floater: Piece,
    animating_floater: bool,
    locked: bool,
}

impl Board {
    /// Creates a new board given an Id and its upper left corner
    pub fn new(id: Id, position: Pos2) -> Board {
        let mut columns: [Column; (BOARD_WIDTH as usize)] = Default::default();

        for i in 0..columns.len() {
            columns[i] = Column::new(
                Id::new(ColumnId {
                    board_id: id,
                    index: i,
                }),
                Pos2 {
                    x: position.x + PIECE_SPACING * (i as f32),
                    y: position.y + PIECE_SPACING,
                },
            );
        }

        Board {
            columns,
            id,
            rect: Rect {
                min: Pos2 {
                    x: position.x,
                    y: position.y + PIECE_SPACING,
                },
                max: Pos2 {
                    x: position.x + PIECE_SPACING * (BOARD_WIDTH as f32),
                    y: position.y + PIECE_SPACING * (BOARD_HEIGHT as f32 + 1.0),
                },
            },
            floater: Piece {
                state: PieceState::PlayerOne,
                position,
            },
            locked: false,
            animating_floater: false,
        }
    }

    /// Renders the board and adds the on_click callback
    ///
    /// Returns an iterator of column indices and their responses
    pub fn render(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
    ) -> impl Iterator<Item = (usize, Response)> {
        // Paint background
        let painter = ui.painter();
        painter.rect_filled(self.rect, Rounding::none(), Color32::YELLOW);

        // Paint columns
        let mut hovering = false;
        let mut responses: Vec<Response> = Vec::new();
        for (index, column) in self.columns.iter().enumerate() {
            let response = column.render(ui);

            // We don't want a locked board to be interactive
            if self.locked {
                continue;
            }

            // Floater logic
            if response.hovered() {
                hovering = true;
                self.floater.position.x = ctx.animate_value_with_time(
                    self.id,
                    self.rect.min.x + PIECE_SPACING * (index as f32),
                    0.25,
                );
            }

            // External column clicked logic
            responses.push(response);
        }

        // Paint floater
        if hovering || self.animating_floater {
            self.floater.render(ui);
        }

        responses.into_iter().enumerate()
    }

    /// Makes the board non-interactable
    pub fn lock(&mut self) {
        self.locked = true;
        self.floater.position.x = self.rect.min.x;
    }

    /// Makes the board interactable
    pub fn unlock(&mut self) {
        self.locked = false;
        self.animating_floater = false;
    }

    /// Animates the floater over the given column
    pub fn float_piece(&mut self, ctx: &Context, column: usize, time: f32) {
        self.animating_floater = true;

        self.floater.position.x = ctx.animate_value_with_time(
            self.id,
            self.rect.min.x + PIECE_SPACING * (column as f32),
            time,
        );
    }

    /// Drops a piece down the given column
    pub fn drop_piece(&mut self, column: usize, player: PieceState) {
        let height = self.columns[column].height;

        if height >= (BOARD_HEIGHT as usize) {
            panic!("Trying to drop a piece down a full column: {}", column);
        }

        self.columns[column].pieces[(BOARD_HEIGHT as usize) - 1 - height].state = player;
        self.columns[column].height += 1;

        // The floater represents the next player's move
        self.floater.state = player.reverse();
    }
}
