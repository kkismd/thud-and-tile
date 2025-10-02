// ThumperBlocks用のデバイス独立カラーシステム
// WASM移植のためにcrossterm::style::Colorからの独立を実現

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameColor {
    // メインカラー（テトロミノ用）
    Cyan,
    Magenta,
    Yellow,
    Grey,
    Red,
    Green,
    Blue,
    
    // UIカラー
    White,
    Black,
    DarkGrey,
    
    // 特殊カラー（エフェクト用）
    DarkRed,
    DarkGreen,
    DarkBlue,
    DarkYellow,
    DarkMagenta,
    DarkCyan,
}

impl GameColor {
    /// RGB値を取得（Web/WASM用）
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            GameColor::Cyan => (0, 255, 255),
            GameColor::Magenta => (255, 0, 255),
            GameColor::Yellow => (255, 255, 0),
            GameColor::Grey => (128, 128, 128),
            GameColor::Red => (255, 0, 0),
            GameColor::Green => (0, 255, 0),
            GameColor::Blue => (0, 0, 255),
            GameColor::White => (255, 255, 255),
            GameColor::Black => (0, 0, 0),
            GameColor::DarkGrey => (64, 64, 64),
            GameColor::DarkRed => (128, 0, 0),
            GameColor::DarkGreen => (0, 128, 0),
            GameColor::DarkBlue => (0, 0, 128),
            GameColor::DarkYellow => (128, 128, 0),
            GameColor::DarkMagenta => (128, 0, 128),
            GameColor::DarkCyan => (0, 128, 128),
        }
    }

    /// CSS色名を取得（Web用）
    pub fn to_css_name(&self) -> &'static str {
        match self {
            GameColor::Cyan => "cyan",
            GameColor::Magenta => "magenta",
            GameColor::Yellow => "yellow",
            GameColor::Grey => "gray",
            GameColor::Red => "red",
            GameColor::Green => "lime",
            GameColor::Blue => "blue",
            GameColor::White => "white",
            GameColor::Black => "black",
            GameColor::DarkGrey => "dimgray",
            GameColor::DarkRed => "darkred",
            GameColor::DarkGreen => "darkgreen",
            GameColor::DarkBlue => "darkblue",
            GameColor::DarkYellow => "goldenrod",
            GameColor::DarkMagenta => "darkmagenta",
            GameColor::DarkCyan => "darkcyan",
        }
    }

    /// HEX色コードを取得（Web用）
    pub fn to_hex(&self) -> String {
        let (r, g, b) = self.to_rgb();
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

// ターミナル環境用のcrossterm::style::Colorへの変換
#[cfg(not(target_arch = "wasm32"))]
impl From<GameColor> for crossterm::style::Color {
    fn from(color: GameColor) -> Self {
        use crossterm::style::Color;
        match color {
            GameColor::Cyan => Color::Cyan,
            GameColor::Magenta => Color::Magenta,
            GameColor::Yellow => Color::Yellow,
            GameColor::Grey => Color::Grey,
            GameColor::Red => Color::Red,
            GameColor::Green => Color::Green,
            GameColor::Blue => Color::Blue,
            GameColor::White => Color::White,
            GameColor::Black => Color::Black,
            GameColor::DarkGrey => Color::DarkGrey,
            GameColor::DarkRed => Color::DarkRed,
            GameColor::DarkGreen => Color::DarkGreen,
            GameColor::DarkBlue => Color::DarkBlue,
            GameColor::DarkYellow => Color::DarkYellow,
            GameColor::DarkMagenta => Color::DarkMagenta,
            GameColor::DarkCyan => Color::DarkCyan,
        }
    }
}

// crossterm::style::Colorからの変換（既存コードの移行用）
#[cfg(not(target_arch = "wasm32"))]
impl From<crossterm::style::Color> for GameColor {
    fn from(color: crossterm::style::Color) -> Self {
        use crossterm::style::Color;
        match color {
            Color::Cyan => GameColor::Cyan,
            Color::Magenta => GameColor::Magenta,
            Color::Yellow => GameColor::Yellow,
            Color::Grey => GameColor::Grey,
            Color::Red => GameColor::Red,
            Color::Green => GameColor::Green,
            Color::Blue => GameColor::Blue,
            Color::White => GameColor::White,
            Color::Black => GameColor::Black,
            Color::DarkGrey => GameColor::DarkGrey,
            Color::DarkRed => GameColor::DarkRed,
            Color::DarkGreen => GameColor::DarkGreen,
            Color::DarkBlue => GameColor::DarkBlue,
            Color::DarkYellow => GameColor::DarkYellow,
            Color::DarkMagenta => GameColor::DarkMagenta,
            Color::DarkCyan => GameColor::DarkCyan,
            // その他の色は適切にマッピング
            Color::Reset => GameColor::White,
            Color::Rgb { r, g, b } => {
                // RGB値から最も近い色を選択（簡易実装）
                match (r, g, b) {
                    (255, 255, 255) => GameColor::White,
                    (0, 0, 0) => GameColor::Black,
                    (255, 0, 0) => GameColor::Red,
                    (0, 255, 0) => GameColor::Green,
                    (0, 0, 255) => GameColor::Blue,
                    (255, 255, 0) => GameColor::Yellow,
                    (255, 0, 255) => GameColor::Magenta,
                    (0, 255, 255) => GameColor::Cyan,
                    _ => GameColor::Grey, // デフォルト
                }
            }
            _ => GameColor::Grey, // その他の色はグレーにマッピング
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_conversion() {
        assert_eq!(GameColor::Red.to_rgb(), (255, 0, 0));
        assert_eq!(GameColor::Green.to_rgb(), (0, 255, 0));
        assert_eq!(GameColor::Blue.to_rgb(), (0, 0, 255));
    }

    #[test]
    fn test_hex_conversion() {
        assert_eq!(GameColor::Red.to_hex(), "#ff0000");
        assert_eq!(GameColor::Green.to_hex(), "#00ff00");
        assert_eq!(GameColor::Blue.to_hex(), "#0000ff");
        assert_eq!(GameColor::White.to_hex(), "#ffffff");
        assert_eq!(GameColor::Black.to_hex(), "#000000");
    }

    #[test]
    fn test_css_names() {
        assert_eq!(GameColor::Red.to_css_name(), "red");
        assert_eq!(GameColor::Green.to_css_name(), "lime");
        assert_eq!(GameColor::Blue.to_css_name(), "blue");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_crossterm_conversion() {
        use crossterm::style::Color;
        let game_color = GameColor::Red;
        let crossterm_color: Color = game_color.into();
        assert_eq!(crossterm_color, Color::Red);
        
        let back_to_game: GameColor = crossterm_color.into();
        assert_eq!(back_to_game, GameColor::Red);
    }
}