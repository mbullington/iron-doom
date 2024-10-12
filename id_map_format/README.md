id_map_format
===

This crate provides a high-quality map parser/writer for DOOM Engine games—used by Iron DOOM.

Goals:
- Read, modify, and write WAD files.
- Incremental parsing of individual lumps.
- **TODO:** Support for DOOM/Boom/MBF/MBF21, Heretic, Hexen, Strife, and DOOM 64 (2020 rerelease).

Non-goals:
- Support for UDMF.
- Support ZDoom/GZDoom-specific lumps.

Additions:
- Free-name maps ala ZDoom.
- "Tall wall" hack used by Boom/ZDoom.
- **TODO:** Optionally parses PNGs for flats/walls/sprites.