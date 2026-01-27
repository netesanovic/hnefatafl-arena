#!/bin/bash
# Test runner script for Brandubh rules

echo "=== Running Brandubh Rules Tests ==="
echo ""

# Run all tests
echo "Running all tests..."
cargo test --lib

echo ""
echo "=== Test Categories ==="
echo ""

# Board setup tests
echo "1. Board Setup Tests:"
cargo test --lib test_initial_setup_brandubh test_corner_identification test_throne_identification -- --nocapture

# Movement tests
echo ""
echo "2. Movement Rules Tests:"
cargo test --lib test_only_king_can_enter test_pieces_can_only_move test_piece_cannot_move -- --nocapture

# Capture tests
echo ""
echo "3. Regular Capture Tests:"
cargo test --lib test_regular test_attacker_capture test_corner_is_hostile -- --nocapture

# King capture tests
echo ""
echo "4. King Capture Tests:"
cargo test --lib test_king_capture test_king_not_captured test_king_with_corner test_king_wins -- --nocapture

# Throne hostility tests
echo ""
echo "5. Throne Hostility Tests:"
cargo test --lib test_throne -- --nocapture

# Game flow tests
echo ""
echo "6. Game Flow Tests:"
cargo test --lib test_game_starts test_turns_alternate test_move_count test_cannot_move -- --nocapture

echo ""
echo "=== All Tests Complete ==="
