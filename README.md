# omoio-portraits

Reads the figure pictures out of your own copy of Skylanders Spyro's
Adventure, Giants, SWAP Force, Trap Team or SuperChargers, so
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

From SuperChargers it writes every toy its Collection screen shows, as the
game's own toy data names them: each vehicle (ids 3220 to 3241), SuperCharger
(3400 to 3428), trophy (3500 to 3503) and older figure, with every variant
the game knows. The ten element symbols are white shapes as for the other
games, the bolt it shows for a SuperCharger is `class-supercharger.png`, and
its Land, Sea and Sky symbols are white shapes for Omoio to colour,
`terrain-land.png`, `terrain-sea.png` and `terrain-sky.png`. A swapper's
bottom gets its top's picture, as the game shows a swapper by its top. The
figures of some variants carry a bit, 0x100, that the game's toy data has no
field for, so each variant's picture is written with that bit and without
it.

Spyro's Adventure and Giants on the PS3, SWAP Force and Trap Team on the PS3
and the Wii U, and SuperChargers on the Wii U are known so far. SWAP Force and
Trap Team give the same pictures from either.

`survey` is for working out a game that isn't known yet, and writes nothing.
It prints the game's title id, its files by kind, each archive with its size,
its format version and whether it opens here, and the pictures inside that
look like a menu's, with their size and pixel format. A file in the game's
own format that isn't a picture is told by the kinds of object it holds, and
one of a format version not read here is tried as the newest that is, which
the survey says. Long lists are cut short. Given a word, it prints only the
entries whose names hold that word, all of them, with the names of the
outside things each file points to, such as a toy's materials.

### Skylanders Imaginators

Imaginators on the Wii U is told by its title ids, but its pictures can't be
read yet: `pictures` stops on it with a message and writes nothing. Once they
can, they will be written under these names, so Omoio can look for them
already:

- each Sensei (ids 601 to 631, such as `601-5000.png`), Creation Crystal (680
  to 689, such as `680-5208.png`) and older figure the game knows, as
  `<id>-<variant>.png`, as for the other games;
- the element symbols as `element-<name>.png`, and `element-kaos.png` if the
  game has a Kaos symbol;
- the game's badge for a Sensei as `class-sensei.png`, and for a villain
  Sensei as `class-villain_sensei.png` if it has a badge of its own;
- the symbol of each battle class, if the game has them, as white shapes for
  Omoio to colour: `class-knight.png`, `class-bowslinger.png`,
  `class-quickshot.png`, `class-ninja.png`, `class-brawler.png`,
  `class-smasher.png`, `class-sorcerer.png`, `class-swashbuckler.png`,
  `class-sentinel.png`, `class-bazooker.png` and `class-kaos.png`.

## Build

Needs Rust.

```
cargo test
cargo build --release
```

## Licence

Not decided yet.
