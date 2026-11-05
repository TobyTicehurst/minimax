== Mancala Ruleset ==

Kalah(6, 4), as described at https://en.wikipedia.org/wiki/Kalah

Each player has 6 pits, 1 store (to the right of their pits)

Players start with 4 stones in each pit. The first player to move is considered player 1.

1. On a player's turn, they select a pit with at least one stone in it. Pick up all stones from that pit.
2. "Sow" those stones, one at a time, starting from the pit to the right of the one selected and moving counter-clockwise. The opponent's store is skipped.
3. If the final sowed stone is placed in one of the current player's own pits which was previously empty, and the opponent's opposite pit contains stones, "capture" both the final sowed stone and all stones in the opposite pit and move them to the current player's store. If the opposite pit is empty, nothing is captured. This can only happen when moving 13 or fewer stones since any more would result in the pit not being empty when the final stone is played.
4. If the final sowed stone is placed in the current player's store, the current player takes another turn. This can happen repeatedly.
5. If rule 4 doesn't come into effect then play passes to the opponent.
6. The game ends as soon as either player has no stones left in their pits (this can happen after either player's move, including straight after a capture). The other player then moves all their remaining stones into their store and whoever has the most stones in their store wins.

== TODO ==

1. Flame graph is showing reverse() as a potential candidate for optimisation
2. smart_ordering() - will also solve the reverse issue
3. Call handle_game_over() less if possible
4. Implement Endgame database now that we can reach 30+ depth (average game is around 48 moves)

== Flame Graph ==

cargo flamegraph --release

(Need to run as admin)
