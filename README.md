# omoio-portraits

Reads the figure pictures out of your own copy of Skylanders Giants, SWAP
Force or Trap Team, so
[Omoio](https://github.com/Bertrram/omoio)'s portal menu can show each figure
as the game draws it. Omoio downloads this program only when you ask for the
pictures, and checks it against a fingerprint before it runs.

It comes with no pictures and downloads none. Everything it writes comes from
the game files you already have: for a Wii U game, a `.wua` archive from
Cemu's Title Manager or an unpacked game folder; for Giants on the PS3, the
game's folder, the one that holds `PS3_GAME`. It decrypts nothing.

## Use

```
omoio-portraits title <game.wua or game folder>
omoio-portraits pictures <game.wua or game folder> <folder>
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
out of one (`villain-<number>-loose.png`). It prints `progress <done> <of>`
as it goes and `done <written>` at the end.

Giants on the PS3, SWAP Force and Trap Team are known so far.

## Build

Needs Rust.

```
cargo test
cargo build --release
```

## Licence

Not decided yet.
