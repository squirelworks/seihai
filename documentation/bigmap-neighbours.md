# Big-map neighbours

Laundry item 9: on a big map (`PLO_BIGMAP` 171, the `bigmap2.txt` grid), the level across an edge
should already be asked for when the player walks the edge link, so the crossing is seamless.

## The rule

`Gmap::big_map_ahead_within` (in `gmap.rs`) names the levels to ask for: the side neighbour of each
edge the middle of the player's body is near, and the corner level when two sides are near. Each name
goes once per connection; a bare name gets `.nw`. The mae main loop sends them as file requests in the
same tick (`asking for X ahead (big map neighbour)` in the log).

"Near" is `big_map_ahead_distance(view_w, view_h)`: half the view plus 4 tiles on each axis, never
under `BIGMAP_NEAR_EDGE` (10). With mae's 40 x 30 view that is 24 tiles across and 19 down, so the file
is asked for by the time the edge can come into view. It was a flat 10 tiles before; with a 40-tile
view the edge then showed black for about 10 tiles of walking (lab, 2026-10-31).

In one tick the loop steps, checks the links, then makes the ahead request. The edge link fires only at
the edge, so the request goes out first:

| walk east from x 30 in `mid.nw` (half-tile steps) | step |
|---|---|
| `e.nw` asked for (body middle at 54) | 48 |
| link to `e.nw` fires (feet on tile 63) | 65 |

That is 17 steps, about 850 ms at the 50 ms walk (with the old 10-tile distance; the view-based one
asks earlier still). Test:
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

## One world: neighbours drawn past the edges (2026-10-31)

The point of the big map is that it reads as one world. Seihai now draws it that way. Drawing only:
no new message goes out, and the level change still goes to the server as the edge link it always was.

* **The levels next door are drawn past the edges.** `gmap::BigMapView` (on `OfflineSession` as
  `big_map_view`) holds the up to eight levels around the player's level on the big-map grid.
  `OfflineSession::sync_big_map` runs every tick: when the player's level changes it drops the levels
  no longer next door and loads the missing ones, each name tried once until a file comes in
  (`BigMapView::retry`). Offline the loader reads the local files; online only what the server already
  sent (this connection's downloads, or the server's cache folder). `render_view` draws their tiles
  and chest records past the edge, as it does for a gmap's neighbours. Not drawn: their NPCs, items
  and baddies (they appear at the crossing).
* **The level left behind stays.** `load_level` and the online level change hand the current level
  to the view (`keep_level_for_big_map`) before it is replaced, with the chests opened on the visit
  open, so after a crossing the old level is still drawn on its side without loading it again.
* **No jump at the edge link.** An edge link lands where the real client does (`edge_link_arrival`):
  east, feet on tile 63, puts the player at x -0.5 of the next level, about 2 to 3 tiles further on
  than the seam. The position and what goes to the server stay as they are; `SeamGlide` draws the
  player and the view from the old place and catches up over `SEAM_GLIDE_MS` (200 ms). Only a jump of
  at most 4 tiles on each axis glides; a server warp elsewhere is drawn at once.
* **No black frame online.** Between the server naming the new level and its board arriving, mae keeps
  the last frame up (`hold_frame_for_board`, at most `BOARD_HOLD`, 1 s).
* **Fixed: online edge links ignored the big map.** A level drawn from the server's board has no file
  name, so `check_links` had no "here" for `edge_link_arrival` and used the link's own new-x (x 0
  going east). `OfflineSession::online_level` now carries the server's level name. The `PLI_LEVELWARP`
  bytes did not change in the lab (x -0.5 and 0 encode the same); the position the client holds after
  arriving, and so its next props, is now the -0.5 the rule gives.

Tests: `render::tiles::tests::a_big_map_edge_link_shows_no_seam` (the frame drawn when the link fires
and the one after the next level is in are the same pixels), `gmap::tests::the_big_map_view_follows_the_player`,
`the_ahead_distance_follows_the_view`, `the_last_frame_waits_for_a_big_map_board_but_not_forever`,
`offline::tests::an_edge_link_glides_only_when_it_lands_by_the_seam`,
`the_level_left_on_a_big_map_is_kept_with_its_opened_chests`.

Lab (2026-10-31, GServer-v2 beta4 loopback, `bigmap = bigmap2.txt` with three levels in a row, Seihai
2.22 walking east from `lab_mid.nw` x 40): on main the east edge showed black from about x 44 until the
crossing and the view jumped at the link; on this branch the next level was drawn the whole way and the
crossing scrolled through with no black frame. Client-to-server messages were the same kinds in the
same order; the differences were the `lab_e.nw` `WANTFILE` going out earlier and, after the arrival,
props half a tile apart (the arrival fix above). Not tested: a real 2.05 or 2.22 client on this map,
north/south crossings in the lab (covered by the unit tests' rules only), and gmaps (unchanged).
