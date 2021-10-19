use hc_mixin_turn_based_game::{GameOutcome, TurnBasedGame};
use hdk::prelude::holo_hash::AgentPubKeyB64;
use hdk::prelude::*;

pub const BOARD_SIZE: usize = 3;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TicTacToe {
    pub player_1: Vec<Piece>,
    pub player_2: Vec<Piece>,
    pub player_resigned: Option<AgentPubKeyB64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub enum TicTacToeMove {
    Place(Piece),
    Resign,
}

#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes)]
pub struct Winner(AgentPubKeyB64);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Piece {
    pub x: usize,
    pub y: usize,
}

impl Piece {
    fn is_in_bounds(&self) -> ExternResult<()> {
        if self.x < BOARD_SIZE && self.y < BOARD_SIZE
        // no need to check > 0 as usize is always positive
        {
            Ok(())
        } else {
            Err(WasmError::Guest("Position is not in bounds".into()))
        }
    }

    fn is_empty(&self, game_state: &TicTacToe) -> ExternResult<()> {
        match game_state.to_dense()[self.x][self.y] == 0 {
            true => Ok(()),
            false => Err(WasmError::Guest(
                "A piece already exists at that position".into(),
            )),
        }
    }
}

impl TurnBasedGame for TicTacToe {
    type GameMove = TicTacToeMove;
    type GameResult = Winner;

    fn min_players() -> Option<usize> {
        Some(2)
    }

    fn max_players() -> Option<usize> {
        Some(2)
    }

    fn initial(_players: &Vec<AgentPubKeyB64>) -> Self {
        TicTacToe {
            player_1: vec![],
            player_2: vec![],
            player_resigned: None,
        }
    }

    fn apply_move(
        &mut self,
        game_move: TicTacToeMove,
        author: AgentPubKeyB64,
        players: Vec<AgentPubKeyB64>,
    ) -> ExternResult<()> {
        match game_move {
            TicTacToeMove::Place(piece) => {
                piece.is_in_bounds()?;
                piece.is_empty(&self)?;

                match author.eq(&players[0]) {
                    true => self.player_1.push(piece.clone()),
                    false => self.player_2.push(piece.clone()),
                }
            }
            TicTacToeMove::Resign => self.player_resigned = Some(author),
        }

        Ok(())
    }

    fn outcome(&self, players: Vec<AgentPubKeyB64>) -> GameOutcome<Winner> {
        if let Some(resigned_player) = self.player_resigned.clone() {
            return match resigned_player.eq(&players[0]) {
                true => GameOutcome::Finished(Winner(players[1].clone())),
                false => GameOutcome::Finished(Winner(players[0].clone())),
            };
        }

        let board = self.to_dense();

        // check if this resulted in a player victory
        let mut diag_down = 0;
        let mut diag_up = 0;
        let mut across = [0; 3];
        let mut down = [0; 3];
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let delta = match board[x][y] {
                    1 => 1,
                    2 => -1,
                    _ => 0,
                };
                down[x] += delta;
                across[y] += delta;
                // diag down e.g. \
                if x == y {
                    diag_down += delta;
                }
                //diag up  e.g. /
                else if x == (BOARD_SIZE - 1 - y) {
                    diag_up += delta;
                }
            }
        }
        let player_1_victory = across.iter().any(|e| *e == (BOARD_SIZE as i32))
            || down.iter().any(|e| *e == (BOARD_SIZE as i32))
            || diag_down == (BOARD_SIZE as i32)
            || diag_up == (BOARD_SIZE as i32);

        let player_2_victory = across.iter().any(|e| *e == (-1 * BOARD_SIZE as i32))
            || down.iter().any(|e| *e == (-1 * BOARD_SIZE as i32))
            || diag_down == (-1 * BOARD_SIZE as i32)
            || diag_up == (-1 * BOARD_SIZE as i32);
        if player_1_victory {
            return GameOutcome::Finished(Winner(players[0].clone()));
        } else if player_2_victory {
            return GameOutcome::Finished(Winner(players[1].clone()));
        }
        return GameOutcome::Ongoing;
    }
}

impl TicTacToe {
    pub fn to_dense(&self) -> [[u8; 8]; 8] {
        let mut board = [[0u8; 8]; 8];
        self.player_1.iter().for_each(|piece| {
            board[piece.x][piece.y] = 1;
        });
        self.player_2.iter().for_each(|piece| {
            board[piece.x][piece.y] = 2;
        });
        board
    }

    pub fn _from_dense(board: [[u8; 8]; 8]) -> Self {
        let mut player_1_pieces = Vec::new();
        let mut player_2_pieces = Vec::new();
        board.iter().enumerate().for_each(|(x, row)| {
            row.iter().enumerate().for_each(|(y, square)| {
                if *square == 1 {
                    player_1_pieces.push(Piece { x, y });
                } else if *square == 2 {
                    player_2_pieces.push(Piece { x, y });
                }
            })
        });

        TicTacToe {
            player_1: player_1_pieces,
            player_2: player_2_pieces,
            player_resigned: None,
        }
    }
}
