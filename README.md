# omoio-portraits

Reads the figure pictures out of your own copy of Skylanders Spyro's
Adventure, Giants, SWAP Force or Trap Team, so
[Omoio](https://github.com/Bertrram/omoio)'s portal menu can show each figure
as the game draws it. Omoio downloads this program only when you ask for the
pictures, and checks it against a fingerprint before it runs.

It comes with no pictures and downloads none. Everything it writes comes from
the game files you already have: for a Wii U game, a `.wua` archive from
Cemu's Title Manager or an unpacked game folder; for a PS3 game, the game's
folder, the one that holds `PS3_GAME`. It decrypts nothing.

## Use

```
omoio-portraits title <game.wua or game folder>
omoio-portraits pictures <game.wua or game folder> <folder>
omoio-portraits survey <game.wua or game folder> [<word>]
```

`title` prints the game's title id, a PS3 game's from its `PARAM.SFO`.
`pictures` writes one PNG per figure, named `<id>-<variant>.png` with the
variant as four hex digits. From Giants it writes every figure, magic item
and sidekick its Collection screen shows, the eight element symbols as
`element-<name>.png`, and the badge it shows for a Giant as
`class-giant.png`. From SWAP Force it also writes the element symbols and
the Swap Zone badge of each way a swapper moves as `movement-<name>.png`.
From Trap Team it writes every trap the same way as the figures, and each
villain as the Villain Vault shows it, in a trap (`villain-<number>.png`) and
out of one (`villain-<number>-loose.png`). From Spyro's Adventure it writes
each Skylander whole, as its versus screen shows them, and the element
symbols. It prints `progress <done> <of>` as it goes and `done <written>` at
the end.

Spyro's Adventure and Giants on the PS3, and SWAP Force and Trap Team on the
PS3 and the Wii U, are known so far. SWAP Force and Trap Team give the same
pictures from either.

`survey` is for working out a game that isn't known yet, and writes nothing.
It prints the game's title id, its files by kind, each archive with its size,
its format version and whether it opens here, and the pictures inside that
look like a menu's, with their size and pixel format. Long lists are cut
short. Given a word, it prints only the entries whose names hold that word,
all of them.

### Skylanders SuperChargers

SuperChargers on the Wii U isn't known yet, so `pictures` stops on it with a
message rather than read it as another game. Once it is known, it will write
its pictures under these names, so Omoio can look for them already:

- each vehicle (ids 3220 to 3241), SuperCharger (3400 to 3428), trophy (3500
  to 3503) and older figure as `<id>-<variant>.png`, as for the other games;
- the element symbols as `element-<name>.png`;
- the game's badge for a SuperCharger, if it has one, as
  `class-supercharger.png`;
- its Land, Sea and Sky symbols, if it has them, as `terrain-land.png`,
  `terrain-sea.png` and `terrain-sky.png`, white shapes for Omoio to colour,
  as the element symbols are.

## Build

Needs Rust.

```
cargo test
cargo build --release
```

## Licence

Not decided yet.
