use egui::{Color32, Context, Id, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui};

use crate::consts::{BOARD_HEIGHT, BOARD_WIDTH};

/// The size a piece takes up.
const PIECE_RADIUS: f32 = 38.0;
/// The space between pieces.
const PIECE_SPACING: f32 = 90.0;
/// Half of the piece spacing, used for centering things.
const HALF_SPACING: f32 = PIECE_SPACING / 2.0;

/// How fast a piece falls down a single row.
const FALLING_SPEED: f32 = 0.12;

/// The set of points for triangles used to display the background.
const BACKGROUND_TRIANGLES: [[Pos2; 3]; 4] = [
    [
        Pos2 { x: 0.0, y: 0.0 },
        Pos2 {
            x: HALF_SPACING,
            y: 0.0,
        },
        Pos2 {
            x: 0.0,
            y: HALF_SPACING,
        },
    ],
    [
        Pos2 {
            x: PIECE_SPACING,
            y: 0.0,
        },
        Pos2 {
            x: HALF_SPACING,
            y: 0.0,
        },
        Pos2 {
            x: PIECE_SPACING,
            y: HALF_SPACING,
        },
    ],
    [
        Pos2 {
            x: PIECE_SPACING,
            y: PIECE_SPACING,
        },
        Pos2 {
            x: HALF_SPACING,
            y: PIECE_SPACING,
        },
        Pos2 {
            x: PIECE_SPACING,
            y: HALF_SPACING,
        },
    ],
    [
        Pos2 {
            x: 0.0,
            y: PIECE_SPACING,
        },
        Pos2 {
            x: HALF_SPACING,
            y: PIECE_SPACING,
        },
        Pos2 {
            x: 0.0,
            y: HALF_SPACING,
        },
    ],
];

/// A piece (or lack thereof) on the gameboard.
///
/// A piece can correspond to either player one or two.
#[derive(Default, Clone, Copy)]
pub enum PieceState {
    #[default]
    Empty,
    PlayerOne,
    PlayerTwo,
}

impl PieceState {
    /// Returns a piece corresponding to the opposite player.
    ///
    /// Panics if used on an empty piece.
    pub fn reverse(&self) -> PieceState {
        match self {
            PieceState::Empty => panic!("Tried to reverse an empty piece"),
            PieceState::PlayerOne => PieceState::PlayerTwo,
            PieceState::PlayerTwo => PieceState::PlayerOne,
        }
    }
}

/// Represents a piece hole on the game board.
#[derive(Default)]
struct Piece {
    state: PieceState,
    /// The top left corner of where the piece will end up.
    board_position: Pos2,
    /// The top left corner of where the piece currently is.
    ///
    /// This will change as a piece falls into the final board position.
    piece_position: Pos2,
}

impl Piece {
    /// Paints a piece onto the board.
    fn render_piece(&self, painter: &Painter) {
        let (color, accent_color) = match self.state {
            PieceState::Empty => return,
            PieceState::PlayerOne => (Color32::RED, Color32::DARK_RED),
            PieceState::PlayerTwo => (Color32::BLUE, Color32::DARK_BLUE),
        };

        let center = Pos2 {
            x: self.piece_position.x + HALF_SPACING,
            y: self.piece_position.y + HALF_SPACING,
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

    /// Paints the a single piece hole of the board.
    ///
    /// A piece hole consists of four triangles, plus a border used to
    /// smooth the edges of the triangles into a circular shape.
    fn render_background(&self, painter: &Painter) {
        let center = Pos2 {
            x: self.board_position.x + HALF_SPACING,
            y: self.board_position.y + HALF_SPACING,
        };

        painter.circle_stroke(
            center,
            PIECE_RADIUS,
            Stroke {
                width: 2.0 * (HALF_SPACING - PIECE_RADIUS),
                color: Color32::YELLOW,
            },
        );

        // Offseting the paths by the piece's position on the board
        for mut path in BACKGROUND_TRIANGLES.clone() {
            for point in path.iter_mut() {
                point.x += self.board_position.x;
                point.y += self.board_position.y;
            }

            let shape = Shape::convex_polygon(path.into(), Color32::YELLOW, Stroke::NONE);
            painter.add(shape);
        }
    }
}

/// A column of the board.
struct Column {
    pieces: [Piece; BOARD_HEIGHT as usize],
    id: Id,
    /// The rectangular region that the column occupies.
    rect: Rect,
    /// How many pieces currently are in the column.
    height: usize,
}

impl Column {
    /// Creates a column, given a position that will serve as its
    /// upper left corner.
    fn new(id: Id, position: Pos2) -> Column {
        let mut new_column = Column {
            id,
            pieces: Default::default(),
            rect: Rect {
                min: position,
                max: Pos2 {
                    x: position.x + PIECE_SPACING,
                    y: position.y + PIECE_SPACING * (BOARD_HEIGHT as f32),
                },
            },
            height: 0,
        };

        let mut piece_pos = position;
        for i in 0..(BOARD_HEIGHT as usize) {
            piece_pos.y = new_column.get_y_position_of_piece(i as f32);

            new_column.pieces[i] = Piece {
                state: PieceState::Empty,
                board_position: piece_pos.clone(),
                piece_position: position,
            };
        }

        new_column
    }

    /// Renders a column and all the pieces contained in the column.
    fn render(&self, ui: &mut Ui) {
        let painter = ui.painter();

        for piece in self.pieces.iter() {
            piece.render_piece(painter);
        }
        for piece in self.pieces.iter() {
            piece.render_background(painter);
        }
    }

    /// Returns a response that allows for click and hover checking.
    ///
    /// Will only have hover checking if the column is already full.
    fn response(&self, ui: &mut Ui) -> Response {
        let mut sense = Sense::hover();
        if self.height < BOARD_HEIGHT as usize {
            sense = sense.union(Sense::click());
        }

        ui.interact(self.rect, self.id, sense)
    }

    /// Returns the y position that a piece should occupy given that it is
    /// in a particular row of the column.
    fn get_y_position_of_piece(&self, row: f32) -> f32 {
        row * PIECE_SPACING + self.rect.min.y
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

/// A hashable struct used to create a distinct id for a column,
/// based on a board ID and a column index.
#[derive(Hash)]
struct ColumnId {
    board_id: Id,
    index: usize,
}

/// A game board, consisting of six rows and seven columns.
pub struct Board {
    columns: [Column; BOARD_WIDTH as usize],
    id: Id,
    rect: Rect, // TODO: Possibly just change this to the position of the upper left corner
    /// A piece that floats above the board to show where the user is pointing.
    floater: Piece,
    /// Whether the floating piece is currently being externally animated.
    animating_floater: bool,
    /// Whether the Board is currently interactable.
    locked: bool,
    /// Contains the indices of a piece that is falling down the board.
    falling_piece: Option<[usize; 2]>,
}

impl Board {
    /// Creates a new board given an Id and its upper left corner.
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
                board_position: position,
                piece_position: position,
            },
            locked: false,
            animating_floater: false,
            falling_piece: None,
        }
    }

    /// Renders the board and its corresponding pieces, as well as any piece animations.
    ///
    /// Returns an iterator of column indices and their responses. Full columns will only
    /// allow for hover checking, while non-full columns will allow for both click and
    /// hover checking.
    pub fn render(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
    ) -> impl Iterator<Item = (usize, Response)> {
        // Updating the position of a piece that is falling
        if let Some([column, row]) = self.falling_piece {
            let final_y_position = self.columns[column].get_y_position_of_piece(row as f32);
            let current_y_position = ctx.animate_value_with_time(
                Id::new(ColumnId {
                    board_id: self.id,
                    index: column,
                }),
                final_y_position,
                // + 1.0 for the fact that the piece is falling from above the board
                FALLING_SPEED * (row as f32 + 1.0),
            );

            self.columns[column].pieces[row].piece_position.y = current_y_position;

            if current_y_position == final_y_position {
                self.falling_piece = None;
            }
        }

        // Paint columns
        let mut hovering = false;
        let mut responses = Vec::new();
        for (index, column) in self.columns.iter().enumerate() {
            column.render(ui);

            // We don't want a locked board to be interactive
            if self.locked || self.falling_piece.is_some() {
                continue;
            }

            let response = column.response(ui);

            // Floater logic
            if response.hovered() {
                hovering = true;
                self.floater.piece_position.x = ctx.animate_value_with_time(
                    self.id,
                    self.rect.min.x + PIECE_SPACING * (index as f32),
                    0.25,
                );
            }

            // External column clicked logic
            responses.push((index, response));
        }

        // Paint floater
        if hovering || self.animating_floater {
            self.floater.render_piece(ui.painter());
        }

        responses.into_iter()
    }

    /// Makes the board non-interactable.
    pub fn lock(&mut self) {
        self.locked = true;
        self.floater.piece_position.x = self.rect.min.x;
    }

    /// Makes the board interactable.
    pub fn unlock(&mut self) {
        self.locked = false;
        self.animating_floater = false;
    }

    /// Animates the floater over the given column.
    pub fn float_piece(&mut self, ctx: &Context, column: usize, time: f32) {
        self.animating_floater = true;

        self.floater.piece_position.x = ctx.animate_value_with_time(
            self.id,
            self.rect.min.x + PIECE_SPACING * (column as f32),
            time,
        );
    }

    /// Drops a piece down the given column.
    pub fn drop_piece(&mut self, ctx: &Context, column: usize, player: PieceState) {
        if self.falling_piece.is_some() {
            panic!("Tried to drop a piece down the board, when one was already falling!");
        }

        let height = self.columns[column].height;

        if height >= (BOARD_HEIGHT as usize) {
            panic!("Trying to drop a piece down a full column: {}", column);
        }

        let row_index = (BOARD_HEIGHT as usize) - 1 - height;
        self.columns[column].pieces[row_index].state = player;
        self.columns[column].height += 1;

        self.falling_piece = Some([column, row_index]);

        // Setting the initial animation state for the piece
        ctx.animate_value_with_time(
            Id::new(ColumnId {
                board_id: self.id,
                index: column,
            }),
            // -1.0 due to the fact that the piece is falling from above the board
            self.columns[column].get_y_position_of_piece(-1.0),
            0.0,
        );

        // The floater represents the next player's move
        self.floater.state = player.reverse();
    }
}
