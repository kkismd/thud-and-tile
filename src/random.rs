// Thud & Tile用のデバイス独立ランダム数生成システム
// WASM移植のためにrand::thread_rng()からの独立を実現

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// JavaScript Math.random()をインポート（ブラウザ環境のみ）
#[cfg(all(target_arch = "wasm32", feature = "wasm", not(test)))]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Math, js_name = random)]
    fn js_math_random() -> f64;
}

// Node.js環境またはテスト環境でのポリフィル
#[cfg(any(
    all(target_arch = "wasm32", not(feature = "wasm")),
    all(target_arch = "wasm32", test)
))]
pub fn js_math_random() -> f64 {
    // Node.js環境では決定論的なランダム値を生成
    use std::cell::RefCell;
    thread_local! {
        static SEED: RefCell<u64> = RefCell::new(12345);
    }

    SEED.with(|s| {
        let mut seed = s.borrow_mut();
        *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        (*seed % (1u64 << 31)) as f64 / (1u64 << 31) as f64
    })
}

/// プラットフォーム独立なランダム数プロバイダー
pub trait RandomProvider: Send + Sync {
    /// 指定された範囲の整数を生成
    fn gen_range(&mut self, min: usize, max: usize) -> usize;

    /// スライスから要素をランダムに選択
    fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T>;

    /// スライスをシャッフル
    fn shuffle<T>(&mut self, slice: &mut [T]);

    /// bool値をランダムに生成
    fn gen_bool(&mut self) -> bool;

    /// f64値を[0.0, 1.0)の範囲で生成
    fn gen_f64(&mut self) -> f64;
}

/// ネイティブ環境用のランダム数プロバイダー（rand crateベース）
#[cfg(not(target_arch = "wasm32"))]
pub struct NativeRandomProvider {
    rng: rand::rngs::StdRng,
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeRandomProvider {
    pub fn new() -> Self {
        use rand::SeedableRng;
        use std::time::{SystemTime, UNIX_EPOCH};

        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        Self {
            rng: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }

    /// シード付きで作成（テスト用）
    pub fn with_seed(seed: u64) -> Self {
        use rand::SeedableRng;
        Self {
            rng: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl RandomProvider for NativeRandomProvider {
    fn gen_range(&mut self, min: usize, max: usize) -> usize {
        use rand::Rng;
        self.rng.gen_range(min..max)
    }

    fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        use rand::seq::SliceRandom;
        slice.choose(&mut self.rng)
    }

    fn shuffle<T>(&mut self, slice: &mut [T]) {
        use rand::seq::SliceRandom;
        slice.shuffle(&mut self.rng);
    }

    fn gen_bool(&mut self) -> bool {
        use rand::Rng;
        self.rng.gen_bool(0.5)
    }

    fn gen_f64(&mut self) -> f64 {
        use rand::Rng;
        self.rng.gen_range(0.0..1.0)
    }
}

/// Web/WASM環境用のランダム数プロバイダー
#[cfg(target_arch = "wasm32")]
pub struct WebRandomProvider {
    // WebAssembly環境では、Web Crypto APIまたは
    // JavaScript Math.random()を使用
    state: u64, // シンプルなXorshift64状態
}

#[cfg(target_arch = "wasm32")]
impl WebRandomProvider {
    pub fn new() -> Self {
        // 初期化時にJavaScriptからシードを取得
        let seed = Self::get_random_seed();
        Self { state: seed }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed }, // 0は避ける
        }
    }

    /// JavaScript環境からランダムシードを取得（テスト時は固定値）
    fn get_random_seed() -> u64 {
        #[cfg(all(target_arch = "wasm32", not(test)))]
        {
            // JavaScriptのMath.random()を使用してランダムシードを生成
            let random_value = js_math_random();
            // 0.0-1.0の範囲を64bit整数に変換
            (random_value * u64::MAX as f64) as u64
        }

        #[cfg(any(not(target_arch = "wasm32"), test))]
        {
            // テスト環境や非WASM環境では固定シードを使用
            12345u64
        }
    }

    /// Xorshift64アルゴリズムで次の値を生成
    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }
}

#[cfg(target_arch = "wasm32")]
impl RandomProvider for WebRandomProvider {
    fn gen_range(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        let range = max - min;

        #[cfg(all(target_arch = "wasm32", not(test)))]
        {
            // JavaScriptのMath.random()を直接使用してより良いランダム性を確保
            let random_value = js_math_random(); // 0.0-1.0の範囲
            min + (random_value * range as f64) as usize
        }

        #[cfg(any(not(target_arch = "wasm32"), test))]
        {
            // テスト環境では線形合同法で疑似ランダム
            let random_value = self.next_u64() as f64 / u64::MAX as f64;
            min + (random_value * range as f64) as usize
        }
    }

    fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            return None;
        }
        let index = self.gen_range(0, slice.len());
        slice.get(index)
    }

    fn shuffle<T>(&mut self, slice: &mut [T]) {
        // Fisher-Yatesシャッフルアルゴリズム
        for i in (1..slice.len()).rev() {
            let j = self.gen_range(0, i + 1);
            slice.swap(i, j);
        }
    }

    fn gen_bool(&mut self) -> bool {
        (self.next_u64() & 1) == 1
    }

    fn gen_f64(&mut self) -> f64 {
        // 64bitの値を[0.0, 1.0)に正規化
        (self.next_u64() >> 11) as f64 * (1.0 / ((1u64 << 53) as f64))
    }
}

/// デフォルトのRandomProviderを作成する便利関数
pub fn create_default_random_provider() -> RandomProviderImpl {
    #[cfg(not(target_arch = "wasm32"))]
    {
        RandomProviderImpl::Native(NativeRandomProvider::new())
    }

    #[cfg(target_arch = "wasm32")]
    {
        RandomProviderImpl::Web(WebRandomProvider::new())
    }
}

/// RandomProviderの具象実装のenum
pub enum RandomProviderImpl {
    #[cfg(not(target_arch = "wasm32"))]
    Native(NativeRandomProvider),
    #[cfg(target_arch = "wasm32")]
    Web(WebRandomProvider),
    Deterministic(DeterministicRandomProvider),
}

impl RandomProvider for RandomProviderImpl {
    fn gen_range(&mut self, low: usize, high: usize) -> usize {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            RandomProviderImpl::Native(provider) => provider.gen_range(low, high),
            #[cfg(target_arch = "wasm32")]
            RandomProviderImpl::Web(provider) => provider.gen_range(low, high),
            RandomProviderImpl::Deterministic(provider) => provider.gen_range(low, high),
        }
    }

    fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            RandomProviderImpl::Native(provider) => provider.choose(slice),
            #[cfg(target_arch = "wasm32")]
            RandomProviderImpl::Web(provider) => provider.choose(slice),
            RandomProviderImpl::Deterministic(provider) => provider.choose(slice),
        }
    }

    fn shuffle<T>(&mut self, slice: &mut [T]) {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            RandomProviderImpl::Native(provider) => provider.shuffle(slice),
            #[cfg(target_arch = "wasm32")]
            RandomProviderImpl::Web(provider) => provider.shuffle(slice),
            RandomProviderImpl::Deterministic(provider) => provider.shuffle(slice),
        }
    }

    fn gen_bool(&mut self) -> bool {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            RandomProviderImpl::Native(provider) => provider.gen_bool(),
            #[cfg(target_arch = "wasm32")]
            RandomProviderImpl::Web(provider) => provider.gen_bool(),
            RandomProviderImpl::Deterministic(provider) => provider.gen_bool(),
        }
    }

    fn gen_f64(&mut self) -> f64 {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            RandomProviderImpl::Native(provider) => provider.gen_f64(),
            #[cfg(target_arch = "wasm32")]
            RandomProviderImpl::Web(provider) => provider.gen_f64(),
            RandomProviderImpl::Deterministic(provider) => provider.gen_f64(),
        }
    }
}

/// テスト用の決定的RandomProvider
pub struct DeterministicRandomProvider {
    values: Vec<usize>,
    index: usize,
}

impl DeterministicRandomProvider {
    pub fn new(values: Vec<usize>) -> Self {
        Self { values, index: 0 }
    }
}

impl RandomProvider for DeterministicRandomProvider {
    fn gen_range(&mut self, low: usize, high: usize) -> usize {
        if self.index >= self.values.len() {
            self.index = 0; // ループ
        }
        let value = self.values[self.index];
        self.index += 1;
        low + (value % (high - low))
    }

    fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            return None;
        }
        let index = self.gen_range(0, slice.len());
        slice.get(index)
    }

    fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.gen_range(0, i + 1);
            slice.swap(i, j);
        }
    }

    fn gen_bool(&mut self) -> bool {
        self.gen_range(0, 2) == 1
    }

    fn gen_f64(&mut self) -> f64 {
        self.gen_range(0, 1000000) as f64 / 1000000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_random_provider() {
        let mut provider = DeterministicRandomProvider::new(vec![0, 1, 2, 3]);

        // gen_range
        assert_eq!(provider.gen_range(0, 10), 0);
        assert_eq!(provider.gen_range(5, 10), 6); // 5 + (1 % 5)
        assert_eq!(provider.gen_range(0, 3), 2);

        // choose
        let items = [10, 20, 30, 40];
        assert_eq!(provider.choose(&items), Some(&40)); // index 3

        // gen_bool
        assert_eq!(provider.gen_bool(), false); // 0 % 2 == 0
        assert_eq!(provider.gen_bool(), true); // 1 % 2 == 1
    }

    #[test]
    fn test_deterministic_shuffle() {
        let mut provider = DeterministicRandomProvider::new(vec![0, 1, 0, 1]);
        let mut items = vec![1, 2, 3, 4];

        provider.shuffle(&mut items);
        // 結果は決定的であるべき
        assert_eq!(items.len(), 4);
        assert!(items.contains(&1));
        assert!(items.contains(&2));
        assert!(items.contains(&3));
        assert!(items.contains(&4));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_native_random_provider() {
        let mut provider = NativeRandomProvider::new();

        // 基本的な動作テスト
        let val = provider.gen_range(0, 10);
        assert!(val < 10);

        let items = [1, 2, 3, 4, 5];
        let chosen = provider.choose(&items);
        assert!(chosen.is_some());
        assert!(items.contains(chosen.unwrap()));

        let bool_val = provider.gen_bool();
        assert!(bool_val == true || bool_val == false);

        let f_val = provider.gen_f64();
        assert!(f_val >= 0.0 && f_val < 1.0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_native_random_provider_with_seed() {
        let mut provider1 = NativeRandomProvider::with_seed(42);
        let mut provider2 = NativeRandomProvider::with_seed(42);

        // 同じシードなら同じ結果
        assert_eq!(provider1.gen_range(0, 100), provider2.gen_range(0, 100));
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_web_random_provider() {
        let mut provider = WebRandomProvider::with_seed(42);

        // 基本的な動作テスト
        let val = provider.gen_range(0, 10);
        assert!(val < 10);

        let items = [1, 2, 3, 4, 5];
        let chosen = provider.choose(&items);
        assert!(chosen.is_some());
        assert!(items.contains(chosen.unwrap()));

        let bool_val = provider.gen_bool();
        assert!(bool_val == true || bool_val == false);

        let f_val = provider.gen_f64();
        assert!(f_val >= 0.0 && f_val < 1.0);
    }

    #[test]
    fn test_create_default_random_provider() {
        let mut provider = create_default_random_provider();

        // 基本的な動作テスト
        let val = provider.gen_range(0, 10);
        assert!(val < 10);

        let items = [1, 2, 3];
        let chosen = provider.choose(&items);
        assert!(chosen.is_some());
    }
}
