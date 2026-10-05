# omoio-portraits

Reads the figure pictures out of your own copy of Skylanders SWAP Force or
Trap Team, so
[Omoio](https://github.com/Bertrram/omoio)'s portal menu can show each figure
as the game draws it. Omoio downloads this program only when you ask for the
pictures, and checks it against a fingerprint before it runs.

It comes with no pictures and downloads none. Everything it writes comes from
the game files you already have, in a form Cemu made them readable in: a
`.wua` archive from Cemu's Title Manager, or an unpacked game folder. It
decrypts nothing.

## Use

```
omoio-portraits title <game.wua>
omoio-portraits pictures <game.wua or game folder> <folder>
```

`title` prints the game's title id. `pictures` writes one PNG per figure,
named `<id>-<variant>.png` with the variant as four hex digits. From SWAP
Force it also writes the eight element symbols as `element-<name>.png` and
the Swap Zone badge of each way a swapper moves as `movement-<name>.png`.
From Trap Team it writes every trap the same way as the figures, and each
villain as the Villain Vault shows it, in a trap (`villain-<number>.png`) and
out of one (`villain-<number>-loose.png`). It prints `progress <done> <of>`
as it goes and `done <written>` at the end.

SWAP Force and Trap Team are known so far.

## Build

Needs Rust.

```
cargo test
cargo build --release
```

## Licence

Not decided yet.
