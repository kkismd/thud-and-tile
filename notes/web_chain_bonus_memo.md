# Web Chain-Bonus Migration Memo

- **Date:** 2025-10-10
- **Context:** Port the CLI chain-bonus system to the Web/WASM build so that both platforms share the same scoring and solid-line consumption behavior.

## CLI Behavior Snapshot
- `GameState::lock_piece` updates connected counts, max chains, and calculates total chain bonus via `board_logic::calculate_chain_bonus`, then stores it in `CustomScoreSystem::chain_bonus` using `set_chain_bonus_from_total`.
- `GameState::consume_chain_bonus_for_solid_lines` checks for trailing solid rows beneath the shrinking playable field, deletes up to `chain_bonus` rows, and reinserts empty rows at the top while reducing `chain_bonus` accordingly.
- Rendering shows the accumulated bonus as the `10-CHAIN` label in the sidebar (`render_chain_bonus_value`).
- Tests cover accumulation, consumption, and UI rendering (`tests/game_state_tests.rs`, `render.rs` tests).

## Web Gaps Identified
- `WasmGameState::lock_piece` does not call `calculate_chain_bonus` nor sync the value into the score system.
- No equivalent of `consume_chain_bonus_for_solid_lines` runs after animations on the Web path.
- WASM bindings lack accessors for `chain_bonus`, so the Web UI cannot display it.
- Web UI (HTML/TS) currently has no `10-CHAIN` panel.

## Planned Commit Steps
1. **Score Sync Update**
   - Update `WasmGameState::lock_piece` to compute `total_chain_bonus` just like CLI and store it via `set_chain_bonus_from_total`.
   - Add any helper parity needed (e.g., reuse `calculate_chain_bonus`).

2. **Solid-Line Consumption in WASM**
   - Add a `consume_chain_bonus_for_solid_lines` equivalent to `WasmGameState` and invoke it alongside animation completion (mirroring CLI `handle_animation`).
   - Ensure board height adjustments and logging align with existing conventions.

3. **Bindings & UI Exposure** *(done 2025-10-10)*
   - Extended `WasmCustomScoreSystem` / `WasmGameState` with `get_chain_bonus` so JS can read the accumulated value.
   - Regenerated the WASM bindings via `wasm-pack build -- --features wasm` and confirmed native tests stay green.
   - Surface the new getter through the web debug panel for quick sanity checks.

4. **Web UI & Tests** *(in progress)*
   - Added a dedicated `10-CHAIN` info panel in `index.html` and refresh logic in `src/main.ts` to show the live bonus and reuse `get_score_details()` for color readouts.
   - Still need to decide on automated coverage (e.g. Playwright smoke or wasm-bindgen tests) before closing the step; manual `npm run build` currently serves as the confidence check.

## Open Questions / Follow-ups
- Decide whether the web scoreboard should expose true per-color totals; if so, extend `CustomScoreSystem` to track them and export a proper `get_color_scores` binding.
- Determine if integration tests should cover chain-bonus-driven solid line removal in WASM (may require new wasm-bindgen test harness).
- Remember to run `wasm-pack build` before release to regenerate the `pkg` bundle (already done post-Step 3, but keep for future iterations).
