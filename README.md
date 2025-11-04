# iron-doom

> README & build instructions work in progress.

![](./example.png)

Iron DOOM is a DOOM wad renderer/viewer written using WGPU, notably: **with no code from the original DOOM project.**

All wad specifications have been implemented either via:

- The Unofficial DOOM Specs
- Doom Wiki
- ZDoom Wiki
- BOOM reference

I've tested this with:
- DOOM / DOOM2
- FreeDOOM
- Heretic
- Chex Quest

## Running the code

The last Rust nightly is required:

```sh
rustup default nightly
rustup upgrade nightly
```

Then, run `cargo run --release` from the `id_viewer` project.

Soon this will be accessible via a Web interface.

## AI Disclosure

> I feel in Nov 2025, this is necessary.

I did not use agentic Large Language Models (LLMs) to create this software. This would have gone against my goals for the software:

1. Learn Rust.
2. Reimplement DOOM in a "clean-room" environment. Even if LLMs aren't the original code
   which is a source of endless debate, they likely have DOOM in their pretraining mix and thus that'd be
   somewhat cheating.

I had GitHub Copilot (the tab autocomplete) on while creating this project.

I also may have asked ChatGPT a question or two. I rationalized this as similar to the **"Game Engine Black Book DOOM"** which sometimes pulls out code snippets.