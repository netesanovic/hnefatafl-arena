# Brandubh Rules Test Suite

This document describes the comprehensive test suite for the Brandubh (7x7 Hnefatafl) game rules implementation.

## Rules Implementation

The game follows Copenhagen rules with the following modifications:
- ❌ No shieldwall rule (4b)
- ❌ No exit forts (6b)
- ❌ No surrounding (7b)

### Key Rules Tested

1. **Board Setup (7x7)**
   - 8 black pieces (attackers)
   - 4 white pieces (defenders)
   - 1 King
   - Cross formation

2. **King Capture Rules**
   - **Away from throne**: Captured like any piece (two enemies on opposite sides)
   - **On throne**: Must be surrounded on all 4 sides
   - **Next to throne**: Must be surrounded on remaining 3 sides (throne counts as occupied)

3. **Hostile Squares**
   - **Corners**: Hostile to all pieces including the King
   - **Throne**: 
     - Always hostile to attackers (black)
     - Hostile to defenders (white) only when not occupied

## Test Suite (38 Tests)

### Board Setup & Configuration (3 tests)
- ✅ `test_initial_setup_brandubh` - Verifies correct initial placement of all pieces
- ✅ `test_corner_identification` - Tests corner square detection
- ✅ `test_throne_identification` - Tests throne (center) square detection

### Movement Rules (6 tests)
- ✅ `test_only_king_can_enter_throne` - Verifies only King can move to throne
- ✅ `test_only_king_can_enter_corners` - Verifies only King can move to corners
- ✅ `test_king_can_enter_corners` - Confirms King is allowed in corners
- ✅ `test_pieces_can_only_move_orthogonally` - No diagonal moves
- ✅ `test_pieces_cannot_jump_over_others` - Pieces blocked by others
- ✅ `test_piece_cannot_move_onto_occupied_square` - Can't move onto occupied squares

### Regular Piece Captures (8 tests)
- ✅ `test_regular_piece_capture_by_sandwich` - Basic custodian capture
- ✅ `test_regular_defender_capture` - Attacker captures defender
- ✅ `test_attacker_capture_by_defenders` - Defender captures attacker
- ✅ `test_corner_is_hostile_to_all` - Corner acts as hostile square for capture
- ✅ `test_corner_is_hostile_to_king` - King can be captured against corner
- ✅ `test_no_capture_without_sandwich` - No capture without both sides
- ✅ `test_capture_requires_opposite_side_hostility` - Both sides must be hostile
- ✅ `test_vertical_and_horizontal_captures` - Captures work in both directions

### King Capture Rules (9 tests)
- ✅ `test_king_capture_away_from_throne_requires_two_sides` - 2 attackers on opposite sides
- ✅ `test_king_capture_on_throne_requires_four_sides` - All 4 sides must be surrounded
- ✅ `test_king_capture_next_to_throne_requires_three_sides` - 3 attackers + throne
- ✅ `test_king_not_captured_with_one_side_on_throne` - 1-3 attackers insufficient on throne
- ✅ `test_king_not_captured_with_two_sides_next_to_throne` - 2 attackers insufficient next to throne
- ✅ `test_king_with_corner_hostility` - King captured between corner and attacker
- ✅ `test_king_between_two_attackers_on_different_axes_not_captured` - Must be opposite sides
- ✅ `test_king_wins_by_reaching_corner` - Defenders win when King reaches corner
- ✅ `test_all_four_corners_win_for_defenders` - Any corner results in defender victory

### Throne Hostility Rules (4 tests)
- ✅ `test_throne_hostile_to_attackers` - Throne hostile to black pieces
- ✅ `test_throne_hostile_to_defenders_when_empty` - Empty throne hostile to white
- ✅ `test_throne_empty_hostile_to_defenders` - Confirms empty throne acts hostile
- ✅ `test_throne_not_hostile_when_occupied_by_king` - Occupied throne not hostile to defenders

### Friendly Fire Prevention (2 tests)
- ✅ `test_king_and_defender_cannot_capture_each_other` - Same team can't capture
- ✅ `test_attackers_cannot_capture_each_other` - Same team can't capture

### Game Flow & Win Conditions (6 tests)
- ✅ `test_game_starts_with_attackers_turn` - Black moves first
- ✅ `test_turns_alternate` - Players alternate turns
- ✅ `test_move_count_increments` - Move counter works correctly
- ✅ `test_attackers_win_by_capturing_king` - Attackers win on King capture
- ✅ `test_cannot_move_after_game_over` - No moves allowed after game ends
- ✅ `test_multiple_captures_in_one_move` - Single move can capture multiple pieces

## Running the Tests

```bash
# Run all tests
cargo test --lib

# Run specific test
cargo test test_king_capture_on_throne_requires_four_sides

# Run tests sequentially
cargo test --lib -- --test-threads=1

# Run with output
cargo test --lib -- --nocapture
```

## Test Results

```
running 38 tests
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured
```

All tests pass successfully, confirming that the Brandubh rules are correctly implemented according to the Copenhagen rules with the specified modifications.

## Test Coverage Summary

| Category | Tests | Status |
|----------|-------|--------|
| Board Setup | 3 | ✅ All Pass |
| Movement Rules | 6 | ✅ All Pass |
| Regular Captures | 8 | ✅ All Pass |
| King Capture | 9 | ✅ All Pass |
| Throne Hostility | 4 | ✅ All Pass |
| Friendly Fire | 2 | ✅ All Pass |
| Game Flow | 6 | ✅ All Pass |
| **TOTAL** | **38** | **✅ All Pass** |
