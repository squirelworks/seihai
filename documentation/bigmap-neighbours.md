# Big-map neighbours

Laundry item 9: on a big map (`PLO_BIGMAP` 171, the `bigmap2.txt` grid), the level across an edge
should already be asked for when the player walks the edge link, so the crossing is seamless.

## The rule

`Gmap::big_map_ahead` (in `gmap.rs`) names the levels to ask for: the side neighbour of each edge the
middle of the player's body is within `BIGMAP_NEAR_EDGE` (10) tiles of, and the corner level when two
sides are near. Each name goes once per connection; a bare name gets `.nw`. The mae main loop sends
them as file requests in the same tick (`asking for X ahead (big map neighbour)` in the log).

In one tick the loop steps, checks the links, then makes the ahead request. The edge link fires only at
the edge, so the request goes out first:

| walk east from x 30 in `mid.nw` (half-tile steps) | step |
|---|---|
| `e.nw` asked for (body middle at 54) | 48 |
| link to `e.nw` fires (feet on tile 63) | 65 |

That is 17 steps, about 850 ms at the 50 ms walk. Test:
`offline::tests::a_big_map_neighbour_is_asked_for_before_its_edge_link_fires`.

## Confirmed, inferred, not tested

* Confirmed in tests: the request goes out before the link fires, once, and not again at the link.
* A Seihai choice, not a real-client rule: Graal_analysis `client-internals/feature-specs/06-gmap-overworld.md`
  says the real clients do nothing special for big-map levels (the server uses the grid for
  cross-edge visibility). The 10-tile distance is Seihai's.
* Known limit: file requests share a window of 4 outstanding `WANTFILE`s (`filecache::WINDOW`). With four
  asset requests still open, the ahead request waits for a free slot and could go out after the link.
  Not changed here.
* Not tested: a lab walk across a big-map edge with the server's `bigmap2.txt`.
