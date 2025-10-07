//! Core Game State - Unified State Management
//! 
//! CLI版とWASM版で共有する統合ゲーム状態構造
//! 純粋関数設計で借用チェッカー競合を完全回避

use crate::cell::Cell;
use crate::config::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::core::animation_logic::{AnimationState, AnimationType};
use crate::core::board_logic::{FixedBoard, Point};
use crate::game_color::GameColor;
use crate::tetromino::{Tetromino, TetrominoShape};

/// 統合ゲーム状態（CLI版とWASM版で共有）
#[derive(Debug, Clone)]
pub struct CoreGameState {
    /// 固定サイズボード（WASMセーフ）
    pub board: FixedBoard,
    
    /// 現在のボード高さ
    pub current_board_height: usize,
    
    /// アクティブなアニメーション群（固定サイズ配列）
    pub animations: [AnimationState; 20],  // ボード高さ分の同時アニメーション
    pub animations_count: usize,           // 実際に使用中のアニメーション数
    
    /// 現在のピース（Option for safety）
    pub current_piece: Option<Tetromino>,
    
    /// ゲームモード
    pub game_mode: CoreGameMode,
    
    /// スコア関連
    pub score: u64,
    pub lines_cleared: u32,
    pub chain_bonus: u32,
    
    /// スコア詳細（色別チェーン）
    pub max_chains: CoreColorMaxChains,
    
    /// その他の統計
    pub pieces_placed: u32,
    pub elapsed_time_ms: u64,
    
    /// EraseLineアニメーション有効フラグ
    pub enable_erase_line: bool,
}

/// 統合ゲームモード
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoreGameMode {
    Title = 0,
    Playing = 1,
    GameOver = 2,
}

/// 統合色別最大チェーン情報
#[derive(Debug, Clone)]
pub struct CoreColorMaxChains {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub cyan: u32,
    pub magenta: u32,
    pub yellow: u32,
    pub grey: u32,
}

impl Default for CoreColorMaxChains {
    fn default() -> Self {
        Self {
            red: 1, green: 1, blue: 1, cyan: 1, magenta: 1, yellow: 1, grey: 1,
        }
    }
}

impl CoreColorMaxChains {
    pub fn get(&self, color: GameColor) -> u32 {
        match color {
            GameColor::Red => self.red,
            GameColor::Green => self.green,
            GameColor::Blue => self.blue,
            GameColor::Cyan => self.cyan,
            GameColor::Magenta => self.magenta,
            GameColor::Yellow => self.yellow,
            GameColor::Grey => self.grey,
            // その他の色は基本値1を返す
            _ => 1,
        }
    }
    
    pub fn set(&mut self, color: GameColor, value: u32) {
        match color {
            GameColor::Red => self.red = value,
            GameColor::Green => self.green = value,
            GameColor::Blue => self.blue = value,
            GameColor::Cyan => self.cyan = value,
            GameColor::Magenta => self.magenta = value,
            GameColor::Yellow => self.yellow = value,
            GameColor::Grey => self.grey = value,
            // その他の色は無視
            _ => {},
        }
    }

    /// 【純粋関数】旧チェーンと新チェーンの増加分を計算（CLI版準拠）
    pub fn calculate_chain_increases(old_chains: &CoreColorMaxChains, new_chains: &CoreColorMaxChains) -> u32 {
        let mut total_increases = 0u32;

        // 各色のMAX-CHAIN増加量を計算（減少時は0）
        if new_chains.cyan > old_chains.cyan {
            total_increases = total_increases.saturating_add(new_chains.cyan - old_chains.cyan);
        }
        if new_chains.magenta > old_chains.magenta {
            total_increases = total_increases.saturating_add(new_chains.magenta - old_chains.magenta);
        }
        if new_chains.yellow > old_chains.yellow {
            total_increases = total_increases.saturating_add(new_chains.yellow - old_chains.yellow);
        }
        // 他の色も同様に処理
        if new_chains.red > old_chains.red {
            total_increases = total_increases.saturating_add(new_chains.red - old_chains.red);
        }
        if new_chains.green > old_chains.green {
            total_increases = total_increases.saturating_add(new_chains.green - old_chains.green);
        }
        if new_chains.blue > old_chains.blue {
            total_increases = total_increases.saturating_add(new_chains.blue - old_chains.blue);
        }
        if new_chains.grey > old_chains.grey {
            total_increases = total_increases.saturating_add(new_chains.grey - old_chains.grey);
        }

        total_increases
    }
}

/// ゲーム状態更新結果
#[derive(Debug, Clone)]
pub struct CoreGameStateUpdateResult {
    pub new_state: CoreGameState,
    pub events: Vec<CoreGameEvent>,
    pub render_required: bool,
}

/// ゲームイベント（状態変更通知）
#[derive(Debug, Clone, PartialEq)]
pub enum CoreGameEvent {
    /// ピースがロックされた
    PieceLocked { position: Point, shape: TetrominoShape },
    
    /// ラインクリアが発生
    LinesCleared { lines: Vec<usize>, is_bottom: bool },
    
    /// アニメーション開始
    AnimationStarted { animation_type: AnimationType },
    
    /// アニメーション完了
    AnimationCompleted { animation_type: AnimationType },
    
    /// スコア更新
    ScoreUpdated { new_score: u64, added_points: u32 },
    
    /// ゲームモード変更
    GameModeChanged { new_mode: CoreGameMode },
    
    /// ゲームオーバー
    GameOver,
}

impl Default for CoreGameState {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreGameState {
    /// 固定配列にアニメーションを追加するヘルパー
    fn add_animation(&mut self, animation: AnimationState) {
        if self.animations_count < 20 {
            self.animations[self.animations_count] = animation;
            self.animations_count += 1;
        }
    }
    
    /// 現在のアニメーション配列をVecとして取得するヘルパー
    fn get_active_animations(&self) -> Vec<AnimationState> {
        self.animations[..self.animations_count].to_vec()
    }
    
    /// アニメーション配列をVecから更新するヘルパー
    fn update_animations_from_vec(&mut self, animations: Vec<AnimationState>) {
        let count = animations.len().min(20);
        for (i, animation) in animations.into_iter().take(20).enumerate() {
            self.animations[i] = animation;
        }
        self.animations_count = count;
    }
    
    /// 新しいゲーム状態を作成
    pub fn new() -> Self {
        Self {
            board: [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            current_board_height: 0,
            animations: [AnimationState::default(); 20],
            animations_count: 0,
            current_piece: None,
            game_mode: CoreGameMode::Title,
            score: 0,
            lines_cleared: 0,
            chain_bonus: 0,
            max_chains: CoreColorMaxChains::default(),
            pieces_placed: 0,
            elapsed_time_ms: 0,
            enable_erase_line: false, // デフォルトで無効
        }
    }
    
    /// 【純粋関数】時間経過による状態更新
    pub fn update_with_time(mut self, current_time_ms: u64) -> CoreGameStateUpdateResult {
        use crate::core::animation_logic::update_all_animation_states;
        
        let mut events = Vec::new();
        let mut render_required = false;
        
        // 経過時間更新
        self.elapsed_time_ms = current_time_ms;
        
        // アニメーション状態更新
        let active_animations = self.get_active_animations();
        let animation_result = update_all_animation_states(active_animations, current_time_ms);
        self.update_animations_from_vec(animation_result.updated_animations);
        
        // 完了したアニメーション処理
        for _completed_lines in animation_result.completed_line_blinks {
            events.push(CoreGameEvent::AnimationCompleted {
                animation_type: AnimationType::LineBlink,
            });
            render_required = true;
        }
        
        for _completed_gray_line in animation_result.completed_push_downs {
            events.push(CoreGameEvent::AnimationCompleted {
                animation_type: AnimationType::PushDown,
            });
            render_required = true;
        }
        
        for (_completed_lines, erased_count) in animation_result.completed_erase_lines {
            events.push(CoreGameEvent::AnimationCompleted {
                animation_type: AnimationType::EraseLine,
            });
            self.chain_bonus = self.chain_bonus.saturating_sub(erased_count);
            render_required = true;
        }
        
        CoreGameStateUpdateResult {
            new_state: self,
            events,
            render_required,
        }
    }
    
    /// 【純粋関数】LineBlink アニメーション開始
    pub fn start_line_blink(mut self, lines: Vec<usize>, start_time_ms: u64) -> Self {
        use crate::core::animation_logic::create_line_blink_animation;
        
        let animation = create_line_blink_animation(lines, start_time_ms);
        self.add_animation(animation);
        self
    }
    
    /// 【純粋関数】PushDown アニメーション開始
    pub fn start_push_down(mut self, gray_line_y: usize, start_time_ms: u64) -> Self {
        use crate::core::animation_logic::create_push_down_animation;
        
        let animation = create_push_down_animation(gray_line_y, start_time_ms);
        self.add_animation(animation);
        self
    }
    
    /// 【純粋関数】EraseLine アニメーション開始
    pub fn start_erase_line(mut self, target_lines: Vec<usize>, start_time_ms: u64) -> Self {
        use crate::core::animation_logic::create_erase_line_animation;
        
        let animation = create_erase_line_animation(target_lines, start_time_ms);
        self.add_animation(animation);
        self
    }
    
    /// 【純粋関数】ライン消去処理
    pub fn clear_lines(mut self, lines: &[usize]) -> Self {
        use crate::core::board_logic::analyze_lines;
        
        let analysis = analyze_lines(lines, self.current_board_height);
        
        // 底辺ラインの標準クリア処理
        for &line_y in &analysis.bottom_lines {
            // ライン削除と上からの補充
            for y in (1..=line_y).rev() {
                self.board[y] = self.board[y - 1];
            }
            self.board[0] = [Cell::Empty; BOARD_WIDTH];
            
            self.lines_cleared += 1;
        }
        
        // 非底辺ラインをグレー化
        for &line_y in &analysis.non_bottom_lines {
            for x in 0..BOARD_WIDTH {
                self.board[line_y][x] = Cell::Occupied(GameColor::Grey);
            }
        }
        
        // ボード高さ再計算
        self.current_board_height = self.calculate_current_height();
        
        self
    }
    
    /// 【純粋関数】スコア加算
    pub fn add_score(mut self, points: u32) -> Self {
        self.score += points as u64;
        self
    }

    /// 【純粋関数】チェーンボーナス追加（EraseLineアニメーション用）
    pub fn add_chain_bonus(mut self, bonus: u32) -> Self {
        self.chain_bonus = self.chain_bonus.saturating_add(bonus);
        self
    }

    /// 【純粋関数】チェーンボーナス消費（EraseLineアニメーション用）
    pub fn consume_chain_bonus(mut self, amount: u32) -> (Self, u32) {
        let consumed = self.chain_bonus.min(amount);
        self.chain_bonus = self.chain_bonus.saturating_sub(consumed);
        (self, consumed)
    }
    
    /// 【純粋関数】チェーン倍率更新
    pub fn update_chain_multiplier(mut self, color: GameColor, chain_count: u32) -> Self {
        let current_max = self.max_chains.get(color);
        if chain_count > current_max {
            self.max_chains.set(color, chain_count);
        }
        self
    }

    /// 【純粋関数】ボード上のConnectedブロックからMAX-CHAIN更新（CLI版準拠）
    pub fn update_max_chains_from_board(mut self) -> Self {
        use crate::cell::Cell;
        
        // ボード全体をスキャンして各色の最大Connected数を見つける
        for y in 0..self.current_board_height {
            for x in 0..crate::config::BOARD_WIDTH {
                if let Cell::Connected { color, count } = self.board[y][x] {
                    let current_max = self.max_chains.get(color);
                    if count as u32 > current_max {
                        self.max_chains.set(color, count as u32);
                    }
                }
            }
        }
        self
    }

    /// 【純粋関数】連結コンポーネントをボードに適用（CLI版準拠）
    pub fn apply_connected_components(mut self, components: Vec<crate::core::board_logic::ConnectedComponent>) -> Self {
        use crate::cell::Cell;
        
        self.board = crate::core::board_logic::apply_connected_components(self.board, &components);
        
        // 各Connectedブロックに正確なカウント数を設定
        for component in &components {
            let count = component.positions.len() as u8;
            for &crate::core::board_logic::Point(x, y) in &component.positions {
                if let Cell::Connected { color, count: _ } = self.board[y][x] {
                    self.board[y][x] = Cell::Connected { color, count };
                }
            }
        }
        
        self
    }
    
    /// 【純粋関数】現在のボード高さを計算
    pub fn calculate_current_height(&self) -> usize {
        use crate::core::board_logic::calculate_board_height;
        calculate_board_height(self.board)
    }
    
    /// 【純粋関数】ピース配置可能性チェック
    pub fn can_place_piece(&self, piece: &Tetromino) -> bool {
        for ((bx, by), _) in piece.iter_blocks() {
            let bx = bx as usize;
            let by = by as usize;
            
            if bx >= BOARD_WIDTH || by >= BOARD_HEIGHT {
                return false;
            }
            
            if self.board[by][bx] != Cell::Empty {
                return false;
            }
        }
        true
    }
    
    /// 【純粋関数】ピースをボードに配置
    pub fn place_piece(mut self, piece: &Tetromino) -> Self {
        for ((bx, by), color) in piece.iter_blocks() {
            let bx = bx as usize;
            let by = by as usize;
            
            if bx < BOARD_WIDTH && by < BOARD_HEIGHT {
                self.board[by][bx] = Cell::Occupied(color);
            }
        }
        
        self.current_piece = None;
        self.pieces_placed += 1;
        self.current_board_height = self.calculate_current_height();
        
        self
    }
    
    /// 【純粋関数】新しいピースをスポーン
    pub fn spawn_piece(mut self) -> Self {
        self.current_piece = Some(Tetromino::new_random());
        self
    }
    
    /// アニメーション実行中かどうか
    pub fn has_animations(&self) -> bool {
        self.animations_count > 0
    }
    
    /// アクティブなアニメーション数
    pub fn animation_count(&self) -> usize {
        self.animations_count
    }
    
    /// 指定タイプのアニメーションが実行中かどうか
    pub fn has_animation_type(&self, animation_type: AnimationType) -> bool {
        self.animations[..self.animations_count].iter().any(|anim| anim.animation_type == animation_type)
    }
}

/// 簡易レンダリング情報（WASM用）
#[derive(Debug, Clone)]
pub struct CoreRenderInfo {
    pub board: FixedBoard,
    pub current_piece_blocks: Vec<(Point, GameColor)>,
    pub ghost_piece_blocks: Vec<Point>,
    pub score: u64,
    pub lines_cleared: u32,
    pub chain_bonus: u32,
    pub game_mode: u8,
    pub has_animations: bool,
}

impl CoreGameState {
    /// レンダリング情報の生成（WASM境界安全）
    pub fn generate_render_info(&self) -> CoreRenderInfo {
        let current_piece_blocks = if let Some(ref piece) = self.current_piece {
            piece.iter_blocks()
                .map(|((x, y), color)| (Point(x as usize, y as usize), color))
                .collect()
        } else {
            Vec::new()
        };
        
        // ゴーストピース計算（簡易版）
        let ghost_piece_blocks = Vec::new(); // TODO: 実装
        
        CoreRenderInfo {
            board: self.board,
            current_piece_blocks,
            ghost_piece_blocks,
            score: self.score,
            lines_cleared: self.lines_cleared,
            chain_bonus: self.chain_bonus,
            game_mode: self.game_mode as u8,
            has_animations: self.has_animations(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::animation_logic::AnimationType;

    #[test]
    fn test_core_game_state_creation() {
        let state = CoreGameState::new();
        assert_eq!(state.game_mode, CoreGameMode::Title);
        assert_eq!(state.score, 0);
        assert!(!state.has_animations());
        assert_eq!(state.current_board_height, 0);
    }

    #[test]
    fn test_animation_start() {
        let state = CoreGameState::new();
        let state = state.start_line_blink(vec![5, 10], 1000);
        
        assert!(state.has_animations());
        assert_eq!(state.animation_count(), 1);
        assert!(state.has_animation_type(AnimationType::LineBlink));
    }

    #[test]
    fn test_score_addition() {
        let state = CoreGameState::new();
        let state = state.add_score(1000);
        
        assert_eq!(state.score, 1000);
    }

    #[test]
    fn test_chain_multiplier_update() {
        let mut state = CoreGameState::new();
        state = state.update_chain_multiplier(GameColor::Red, 5);
        
        assert_eq!(state.max_chains.get(GameColor::Red), 5);
        
        // より低い値では更新されない
        state = state.update_chain_multiplier(GameColor::Red, 3);
        assert_eq!(state.max_chains.get(GameColor::Red), 5);
    }

    #[test]
    fn test_piece_spawning() {
        let state = CoreGameState::new();
        let state = state.spawn_piece();
        
        assert!(state.current_piece.is_some());
    }

    #[test]
    fn test_render_info_generation() {
        let state = CoreGameState::new();
        let render_info = state.generate_render_info();
        
        assert_eq!(render_info.score, 0);
        assert_eq!(render_info.game_mode, 0); // Title
        assert!(!render_info.has_animations);
    }
}