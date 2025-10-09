use std::collections::VecDeque;

use crate::cell::{Board, Cell};
use crate::config::{BOARD_HEIGHT, BOARD_WIDTH};

pub type Point = (usize, usize);

pub fn find_and_connect_adjacent_blocks(board: &mut Board, lines_to_clear: &[usize]) {
    let mut cells_to_connect: Vec<(usize, usize)> = Vec::new();
    let mut visited: Vec<Vec<bool>> = vec![vec![false; BOARD_WIDTH]; BOARD_HEIGHT];

    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            if lines_to_clear.contains(&y) {
                continue;
            }

            if let Cell::Occupied(color) = board[y][x] {
                if visited[y][x] {
                    continue;
                }

                let mut component: Vec<(usize, usize)> = Vec::new();
                let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

                visited[y][x] = true;
                queue.push_back((x, y));
                component.push((x, y));

                while let Some((cx, cy)) = queue.pop_front() {
                    let neighbors = [
                        (cx as i8 - 1, cy as i8),
                        (cx as i8 + 1, cy as i8),
                        (cx as i8, cy as i8 - 1),
                        (cx as i8, cy as i8 + 1),
                    ];

                    for (nx, ny) in neighbors {
                        if nx >= 0 && nx < BOARD_WIDTH as i8 && ny >= 0 && ny < BOARD_HEIGHT as i8 {
                            let (nx_usize, ny_usize) = (nx as usize, ny as usize);
                            if lines_to_clear.contains(&ny_usize) {
                                continue;
                            }
                            if !visited[ny_usize][nx_usize] {
                                let neighbor_color = match board[ny_usize][nx_usize] {
                                    Cell::Occupied(c) => Some(c),
                                    Cell::Connected { color: c, count: _ } => Some(c),
                                    _ => None,
                                };
                                if let Some(neighbor_color) = neighbor_color {
                                    if neighbor_color == color {
                                        visited[ny_usize][nx_usize] = true;
                                        queue.push_back((nx_usize, ny_usize));
                                        component.push((nx_usize, ny_usize));
                                    }
                                }
                            }
                        }
                    }
                }

                if component.len() > 1 {
                    cells_to_connect.extend(component);
                }
            }
        }
    }

    for (x, y) in cells_to_connect {
        if let Cell::Occupied(color) = board[y][x] {
            board[y][x] = Cell::Connected { color, count: 1 };
        }
    }
}

pub fn count_connected_blocks(board: &Board, cleared_line_y: usize) -> Vec<(Point, u32)> {
    let mut results = Vec::new();
    let mut visited = vec![vec![false; BOARD_WIDTH]; BOARD_HEIGHT];

    for y in (cleared_line_y + 1)..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            if let Some(color) = match board[y][x] {
                Cell::Occupied(c) => Some(c),
                Cell::Connected { color: c, count: _ } => Some(c),
                _ => None,
            } {
                if visited[y][x] {
                    continue;
                }

                let mut component = Vec::new();
                let mut queue = VecDeque::new();

                visited[y][x] = true;
                queue.push_back((x, y));
                component.push((x, y));

                while let Some((qx, qy)) = queue.pop_front() {
                    let neighbors = [
                        (qx as i8 - 1, qy as i8),
                        (qx as i8 + 1, qy as i8),
                        (qx as i8, qy as i8 - 1),
                        (qx as i8, qy as i8 + 1),
                    ];

                    for (nx, ny) in neighbors {
                        if nx >= 0 && nx < BOARD_WIDTH as i8 && ny >= 0 && ny < BOARD_HEIGHT as i8 {
                            let (nx_usize, ny_usize) = (nx as usize, ny as usize);
                            if !visited[ny_usize][nx_usize] {
                                let neighbor_color = match board[ny_usize][nx_usize] {
                                    Cell::Occupied(c) => Some(c),
                                    Cell::Connected { color: c, count: _ } => Some(c),
                                    _ => None,
                                };
                                if let Some(neighbor_color) = neighbor_color {
                                    if neighbor_color == color {
                                        visited[ny_usize][nx_usize] = true;
                                        queue.push_back((nx_usize, ny_usize));
                                        component.push((nx_usize, ny_usize));
                                    }
                                }
                            }
                        }
                    }
                }

                let component_size = component.len() as u32;
                for &(px, py) in &component {
                    results.push(((px, py), component_size));
                }
            }
        }
    }

    results
}

pub fn remove_isolated_blocks(board: &mut Board, cleared_line_y: usize) {
    let mut blocks_to_remove = Vec::new();

    for y in (cleared_line_y + 1)..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            if let Some(color) = match board[y][x] {
                Cell::Occupied(c) => Some(c),
                Cell::Connected { color: c, count: _ } => Some(c),
                _ => None,
            } {
                let mut is_isolated = true;
                let neighbors = [
                    (x as i8 - 1, y as i8),
                    (x as i8 + 1, y as i8),
                    (x as i8, y as i8 - 1),
                    (x as i8, y as i8 + 1),
                ];

                for (nx, ny) in neighbors {
                    if nx >= 0 && nx < BOARD_WIDTH as i8 && ny >= 0 && ny < BOARD_HEIGHT as i8 {
                        let neighbor_color = match board[ny as usize][nx as usize] {
                            Cell::Occupied(c) => Some(c),
                            Cell::Connected { color: c, count: _ } => Some(c),
                            _ => None,
                        };
                        if let Some(neighbor_color) = neighbor_color {
                            if neighbor_color == color {
                                is_isolated = false;
                                break;
                            }
                        }
                    }
                }

                if is_isolated {
                    blocks_to_remove.push((x, y));
                }
            }
        }
    }

    for (x, y) in blocks_to_remove {
        board[y][x] = Cell::Empty;
    }
}

/// 盤面全体をスキャンし、10個以上の連結グループを検出して、獲得可能なボーナス段数を計算する
/// 各グループの連結数から floor(count / 10) 段を計算し、合計を返す
pub fn calculate_chain_bonus(board: &Board) -> u32 {
    let mut total_bonus = 0;
    let mut visited = vec![vec![false; BOARD_WIDTH]; BOARD_HEIGHT];

    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            if visited[y][x] {
                continue;
            }

            if let Some(color) = match board[y][x] {
                Cell::Occupied(c) => Some(c),
                Cell::Connected { color: c, count: _ } => Some(c),
                _ => None,
            } {
                // BFSで連結グループを検出
                let mut component = Vec::new();
                let mut queue = VecDeque::new();

                visited[y][x] = true;
                queue.push_back((x, y));
                component.push((x, y));

                while let Some((qx, qy)) = queue.pop_front() {
                    let neighbors = [
                        (qx as i8 - 1, qy as i8),
                        (qx as i8 + 1, qy as i8),
                        (qx as i8, qy as i8 - 1),
                        (qx as i8, qy as i8 + 1),
                    ];

                    for (nx, ny) in neighbors {
                        if nx >= 0 && nx < BOARD_WIDTH as i8 && ny >= 0 && ny < BOARD_HEIGHT as i8 {
                            let (nx_usize, ny_usize) = (nx as usize, ny as usize);
                            if !visited[ny_usize][nx_usize] {
                                let neighbor_color = match board[ny_usize][nx_usize] {
                                    Cell::Occupied(c) => Some(c),
                                    Cell::Connected { color: c, count: _ } => Some(c),
                                    _ => None,
                                };
                                if let Some(neighbor_color) = neighbor_color {
                                    if neighbor_color == color {
                                        visited[ny_usize][nx_usize] = true;
                                        queue.push_back((nx_usize, ny_usize));
                                        component.push((nx_usize, ny_usize));
                                    }
                                }
                            }
                        }
                    }
                }

                // 10個以上ならボーナス計算
                let group_size = component.len() as u32;
                if group_size >= 10 {
                    total_bonus += group_size / 10;
                }
            }
        }
    }

    total_bonus
}
