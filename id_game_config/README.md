id_game_config
===

Each ID Tech 1 game (DOOM, Heretic, etc...) have certain "hardcoded values"
that aren't specified in WADs.

- Animated flats / walls.
- Thing types

There are also various mods, such as [DeHackEd](https://doomwiki.org/wiki/DeHackEd#DEHACKED_lump), that customize this behavior.

This crate aims to:
- Enumerate all hardcoded values **without looking at original code.**
- Provide, given a WAD, some sort of game detection.
- **TODO:** Support DeHackEd patches, etc...