# T-mino 物理的色回転マッピング計算

## SRS座標 (tetromino.rs SHAPES[2]より)
- State 0: [(1,0), (0,1), (1,1), (2,1)] - 上向きT
- State 1: [(1,0), (1,1), (2,1), (1,2)] - 右向きT  
- State 2: [(0,1), (1,1), (2,1), (1,2)] - 下向きT
- State 3: [(1,0), (0,1), (1,1), (1,2)] - 左向きT

## 物理的色の移動追跡

### State 0 → State 1
```
State 0 positions: [(1,0), (0,1), (1,1), (2,1)]
State 0 colors:    [  C0,   C1,   C2,   C3  ]
                     ↓     ↓     ↓     ↓
State 1 positions: [(1,0), (1,1), (2,1), (1,2)]

Physical mapping:
- (1,0) → (1,0): C0 stays
- (0,1) → (1,2): C1 moves  
- (1,1) → (1,1): C2 stays
- (2,1) → (2,1): C3 stays

So: new[0]=old[0], new[1]=old[2], new[2]=old[3], new[3]=old[1]
Mapping: [0, 2, 3, 1]
```

### State 1 → State 2  
```
State 1 positions: [(1,0), (1,1), (2,1), (1,2)]
State 1 colors:    [  C0,   C2,   C3,   C1  ]
                     ↓     ↓     ↓     ↓
State 2 positions: [(0,1), (1,1), (2,1), (1,2)]

Physical mapping:
- (1,0) → (0,1): C0 moves
- (1,1) → (1,1): C2 stays  
- (2,1) → (2,1): C3 stays
- (1,2) → (1,2): C1 stays

So: new[0]=old[0], new[1]=old[1], new[2]=old[2], new[3]=old[3]
Mapping: [0, 1, 2, 3]
```

### State 2 → State 3
```
State 2 positions: [(0,1), (1,1), (2,1), (1,2)]  
State 2 colors:    [  C0,   C2,   C3,   C1  ]
                     ↓     ↓     ↓     ↓
State 3 positions: [(1,0), (0,1), (1,1), (1,2)]

Physical mapping:
- (0,1) → (0,1): C0 stays
- (1,1) → (1,1): C2 stays
- (2,1) → (1,0): C3 moves  
- (1,2) → (1,2): C1 stays

So: new[0]=old[2], new[1]=old[0], new[2]=old[1], new[3]=old[3]
Mapping: [2, 0, 1, 3]
```

### State 3 → State 0
```
State 3 positions: [(1,0), (0,1), (1,1), (1,2)]
State 3 colors:    [  C3,   C0,   C2,   C1  ]
                     ↓     ↓     ↓     ↓
State 0 positions: [(1,0), (0,1), (1,1), (2,1)]

Physical mapping:
- (1,0) → (1,0): C3 stays
- (0,1) → (0,1): C0 stays
- (1,1) → (1,1): C2 stays  
- (1,2) → (2,1): C1 moves

So: new[0]=old[0], new[1]=old[1], new[2]=old[2], new[3]=old[3]
Mapping: [0, 1, 2, 3]
```

## 正しいマッピング配列
```rust
match (from_state, to_state) {
    (0, 1) => [0, 2, 3, 1],
    (1, 2) => [0, 1, 2, 3], 
    (2, 3) => [2, 0, 1, 3],
    (3, 0) => [0, 1, 2, 3],
    _ => [0, 1, 2, 3],
}
```