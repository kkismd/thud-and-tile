//! Core Board Logic - Pure Functions
//! 
//! CLI版とWASM版で共有するボード処理の純粋関数群
//! 固定サイズ配列とデータコピーパターンで借用チェッカー競合を回避

use crate::cell::Cell;
use crate::config::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::game_color::GameColor;

/// 固定サイズボード定義（WASMセーフ）
pub type FixedBoard = [[Cell; BOARD_WIDTH]; BOARD_HEIGHT];

/// 座標点（tuple struct for pattern matching）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point(pub usize, pub usize);

/// 連結ブロック検索結果
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectedComponent {
    pub color: GameColor,
    pub positions: Vec<Point>,
    pub size: u32,
}

/// ライン検索結果
#[derive(Debug, Clone, PartialEq)]
pub struct LineAnalysis {
    pub complete_lines: Vec<usize>,
    pub bottom_lines: Vec<usize>,
    pub non_bottom_lines: Vec<usize>,
}

/// 【純粋関数】連結ブロック群の検出
/// 既存のfind_and_connect_adjacent_blocksから抽出
pub fn find_connected_components(
    board: FixedBoard,
    exclude_lines: &[usize],
) -> Vec<ConnectedComponent> {
    let mut components = Vec::new();
    let mut visited = [[false; BOARD_WIDTH]; BOARD_HEIGHT];

    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            if exclude_lines.contains(&y) || visited[y][x] {
                continue;
            }

            let cell_color = match board[y][x] {
                Cell::Occupied(color) => Some(color),
                Cell::Connected { color, .. } => Some(color),
                _ => None,
            };

            if let Some(color) = cell_color {
                let component = flood_fill_component(board, Point(x, y), color, &mut visited, exclude_lines);
                
                // すべてのコンポーネントを記録（孤立ブロックも含む）
                if !component.is_empty() {
                    components.push(ConnectedComponent {
                        color,
                        positions: component.clone(),
                        size: component.len() as u32,
                    });
                }
            }
        }
    }

    components
}

/// 【純粋関数】同色ブロックの連結成分を Flood Fill で検出
fn flood_fill_component(
    board: FixedBoard,
    start: Point,
    target_color: GameColor,
    visited: &mut [[bool; BOARD_WIDTH]; BOARD_HEIGHT],
    exclude_lines: &[usize],
) -> Vec<Point> {
    let mut component = Vec::new();
    let mut stack = vec![start];
    
    while let Some(Point(x, y)) = stack.pop() {
        if visited[y][x] || exclude_lines.contains(&y) {
            continue;
        }
        
        let cell_color = match board[y][x] {
            Cell::Occupied(color) => Some(color),
            Cell::Connected { color, .. } => Some(color),
            _ => None,
        };
        
        if cell_color != Some(target_color) {
            continue;
        }
        
        visited[y][x] = true;
        component.push(Point(x, y));
        
        // 隣接セルをスタックに追加
        let neighbors = get_neighbors(Point(x, y));
        for neighbor in neighbors {
            if !visited[neighbor.1][neighbor.0] {
                stack.push(neighbor);
            }
        }
    }
    
    component
}

/// 【純粋関数】隣接セルの取得
fn get_neighbors(point: Point) -> Vec<Point> {
    let Point(x, y) = point;
    let mut neighbors = Vec::new();
    
    let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    
    for (dx, dy) in directions {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        
        if nx >= 0 && nx < BOARD_WIDTH as i32 && ny >= 0 && ny < BOARD_HEIGHT as i32 {
            neighbors.push(Point(nx as usize, ny as usize));
        }
    }
    
    neighbors
}

/// 【純粋関数】完成ライン（全セルが埋まっているライン）の検出
pub fn find_complete_lines(board: FixedBoard, current_height: usize) -> Vec<usize> {
    let mut complete_lines = Vec::new();
    
    for y in 0..current_height {
        let mut is_complete = true;
        
        for x in 0..BOARD_WIDTH {
            match board[y][x] {
                Cell::Empty => {
                    is_complete = false;
                    break;
                }
                _ => {} // Occupied, Connected, Solid はすべて「埋まっている」扱い
            }
        }
        
        if is_complete {
            complete_lines.push(y);
        }
    }
    
    complete_lines
}

/// 【純粋関数】ライン分析（bottom/non-bottom分類）
pub fn analyze_lines(lines: &[usize], current_height: usize) -> LineAnalysis {
    let mut bottom_lines = Vec::new();
    let mut non_bottom_lines = Vec::new();
    
    let bottom_line = current_height.saturating_sub(1);
    
    for &line_y in lines {
        if line_y == bottom_line {
            bottom_lines.push(line_y);
        } else {
            non_bottom_lines.push(line_y);
        }
    }
    
    LineAnalysis {
        complete_lines: lines.to_vec(),
        bottom_lines,
        non_bottom_lines,
    }
}

/// 【純粋関数】連結ブロック数の計算
/// count_connected_blocksから抽出
pub fn count_connected_blocks_above_line(
    board: FixedBoard,
    cleared_line_y: usize,
) -> Vec<(Point, u32)> {
    let mut results = Vec::new();
    let mut visited = [[false; BOARD_WIDTH]; BOARD_HEIGHT];

    for y in (cleared_line_y + 1)..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            if visited[y][x] {
                continue;
            }

            let cell_color = match board[y][x] {
                Cell::Occupied(color) => Some(color),
                Cell::Connected { color, .. } => Some(color),
                _ => None,
            };

            if let Some(color) = cell_color {
                let component = flood_fill_component(
                    board, 
                    Point(x, y), 
                    color, 
                    &mut visited, 
                    &[]
                );

                let component_size = component.len() as u32;
                for point in component {
                    results.push((point, component_size));
                }
            }
        }
    }

    results
}

/// 【純粋関数】孤立ブロック（隣接する同色ブロックがない）の検出
pub fn find_isolated_blocks(board: FixedBoard, above_line: usize) -> Vec<Point> {
    let mut isolated_blocks = Vec::new();

    for y in (above_line + 1)..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            let cell_color = match board[y][x] {
                Cell::Occupied(color) => Some(color),
                Cell::Connected { color, .. } => Some(color),
                _ => None,
            };

            if let Some(color) = cell_color {
                let neighbors = get_neighbors(Point(x, y));
                let mut has_same_color_neighbor = false;

                for Point(nx, ny) in neighbors {
                    let neighbor_color = match board[ny][nx] {
                        Cell::Occupied(nc) => Some(nc),
                        Cell::Connected { color: nc, .. } => Some(nc),
                        _ => None,
                    };

                    if neighbor_color == Some(color) {
                        has_same_color_neighbor = true;
                        break;
                    }
                }

                if !has_same_color_neighbor {
                    isolated_blocks.push(Point(x, y));
                }
            }
        }
    }

    isolated_blocks
}

/// 【純粋関数】ボードに連結ブロック情報を適用
pub fn apply_connected_components(
    mut board: FixedBoard,
    components: &[ConnectedComponent],
) -> FixedBoard {
    for component in components {
        for &Point(x, y) in &component.positions {
            if let Cell::Occupied(_) = board[y][x] {
                board[y][x] = Cell::Connected {
                    color: component.color,
                    count: 1, // 基本的には1、実際のカウントは別途計算
                };
            }
        }
    }
    board
}

/// 【純粋関数】孤立ブロックを除去したボードを生成
pub fn remove_isolated_blocks_from_board(
    mut board: FixedBoard,
    isolated_blocks: &[Point],
) -> FixedBoard {
    for &Point(x, y) in isolated_blocks {
        board[y][x] = Cell::Empty;
    }
    board
}

/// 【純粋関数】底辺からのSolidライン数をカウント
/// animation.rsのcount_solid_lines_from_bottomから抽出
pub fn count_solid_lines_from_bottom(board: FixedBoard, current_height: usize) -> usize {
    let mut solid_count = 0;
    
    for y in (0..current_height).rev() {
        let mut is_solid_line = true;
        
        for x in 0..BOARD_WIDTH {
            if board[y][x] != Cell::Solid {
                is_solid_line = false;
                break;
            }
        }
        
        if is_solid_line {
            solid_count += 1;
        } else {
            break; // 底辺から連続していないSolidラインは無視
        }
    }
    
    solid_count
}

/// 【純粋関数】指定座標がボード範囲内かどうか
pub fn is_valid_position(x: i32, y: i32) -> bool {
    x >= 0 && x < BOARD_WIDTH as i32 && y >= 0 && y < BOARD_HEIGHT as i32
}

/// 【純粋関数】ボードの高さを再計算
pub fn calculate_board_height(board: FixedBoard) -> usize {
    for y in (0..BOARD_HEIGHT).rev() {
        for x in 0..BOARD_WIDTH {
            if board[y][x] != Cell::Empty {
                return y + 1;
            }
        }
    }
    0 // 空のボード
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_board() -> FixedBoard {
        let mut board = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        
        // テスト用パターン作成
        board[BOARD_HEIGHT - 1][0] = Cell::Occupied(GameColor::Red);
        board[BOARD_HEIGHT - 1][1] = Cell::Occupied(GameColor::Red);
        board[BOARD_HEIGHT - 2][0] = Cell::Occupied(GameColor::Blue);
        
        board
    }

    #[test]
    fn test_find_connected_components() {
        let board = create_test_board();
        let components = find_connected_components(board, &[]);
        
        // 赤ブロックが連結している
        assert!(components.iter().any(|c| 
            c.color == GameColor::Red && c.size == 2
        ));
        
        // 青ブロックは孤立
        assert!(components.iter().any(|c| 
            c.color == GameColor::Blue && c.size == 1
        ));
    }

    #[test]
    fn test_find_complete_lines() {
        let mut board = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        
        // 最下行を完全に埋める
        for x in 0..BOARD_WIDTH {
            board[BOARD_HEIGHT - 1][x] = Cell::Occupied(GameColor::Red);
        }
        
        let complete_lines = find_complete_lines(board, BOARD_HEIGHT);
        assert_eq!(complete_lines, vec![BOARD_HEIGHT - 1]);
    }

    #[test]
    fn test_analyze_lines() {
        let current_height = 15;
        let lines = vec![10, 14]; // 14が底辺ライン
        
        let analysis = analyze_lines(&lines, current_height);
        assert_eq!(analysis.bottom_lines, vec![14]);
        assert_eq!(analysis.non_bottom_lines, vec![10]);
    }

    #[test]
    fn test_find_isolated_blocks() {
        let board = create_test_board();
        let isolated = find_isolated_blocks(board, 0);
        
        // 青ブロックが孤立している
        assert!(isolated.contains(&Point(0, BOARD_HEIGHT - 2)));
    }

    #[test]
    fn test_count_solid_lines_from_bottom() {
        let mut board = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        
        // 底辺2ラインをSolidにする
        for x in 0..BOARD_WIDTH {
            board[BOARD_HEIGHT - 1][x] = Cell::Solid;
            board[BOARD_HEIGHT - 2][x] = Cell::Solid;
        }
        
        let count = count_solid_lines_from_bottom(board, BOARD_HEIGHT);
        assert_eq!(count, 2);
    }
}