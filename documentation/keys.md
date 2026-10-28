# Game keys

`client/src/keys.rs`, read by the game loop in `client/src/bin/main.rs`.

## Defaults: the readme key map

Every readme from 1.33 to 2.05 lists the same keys for player 1, and
Seihai uses them for every version:

| action (`graalconfig.txt` name) | keys | in Seihai |
|---|---|---|
| `up`, `left`, `down`, `right` | arrows, or I, J, K, L | walk; turn sign pages back with Up |
| `action` | A | read a sign, open a chest, lift, throw what is carried |
| `attack` | S or Space | swing the sword; throw what is carried |
| `item` | D | closes the item list (weapons are not fired yet) |
| `map` | M | closes a sign (there is no map screen yet) |
| `itemlist` | Q | opens or closes the item list |
| `switchtochat` | Tab | opens the chat bar |
| `pause` | P | not read yet |

Two of these changed from what Seihai did before. Space was a second
action key and is now a sword key, as the readmes say. Tab opened the
player list. It now opens the chat bar, and the player list moved to F7.

The keys on the HUD buttons (action, attack, item and the small item-list
key) show the first key bound to each action.

## Configurable keys (1.38 and later)

From 1.38 each action can have up to three keys (`Ui.key_config` in the
version profile). For those versions, lines in Seihai's settings file
replace the defaults. Two forms are read:

- the `graalconfig.txt` form, one slot a line: `keyset_attack_key0=S`,
  `keyset_attack_key1=Space`, `keyset_attack_key2=`. An empty value
  leaves that slot unbound. An action with any slot line uses only its
  slot lines. `keyset_<action>_joystickbutton` lines are ignored.
- a short form: `keyset_attack=S,Space`.

Key names: letters, digits, `Up`, `Down`, `Left`, `Right`, `Space`, `Tab`,
`Enter`, `Shift`, `P_Up`, `P_Down`, `Num 0` to `Num 9` (also `N0` or
`numpad0`), `Add`, `Sub`, `Mul`, `Div`, `Dec` and `F9` to `F15`, matched
without regard to case. For 1.33 to 1.37 the lines are ignored, because
those builds have fixed keys.

Not done: the options window that changes the keys inside the client, and
joystick buttons. The exact spelling of every 2.x key name is taken from the
configuration notes and was not checked against a saved `graalconfig.txt`.
