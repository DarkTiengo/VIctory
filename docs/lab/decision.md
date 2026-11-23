# Development base decision

Selected: [Marvisak/VIctory](https://github.com/Marvisak/VIctory), initially
pinned at `1dd7afd4e68300e5dea41cd33a6e0b297798bb2f`. The prior XCL checkout,
fork, build and runtime evidence remain preserved separately.

| Criterion | XCL lab evidence | VIctory evidence |
| --- | --- | --- |
| Our build | Windows cross-build succeeded on Bazzite | Not yet built in this lab |
| Our runtime | Proton CLI/inspection worked; synthetic title aborted at a heap-vtable assertion after six initial checks | Proton unverified |
| Title progress | GTA VI never executed by this lab | Upstream README reports menus, intro audio/video and world loading on Windows; black world, 1.5–3 seconds per frame; bounded run without device loss |
| GPU boundary | Replaces native objects' vtables; conflicts with vkd3d's heap identity check | Separate console objects and native host D3D12 objects |
| Platform | Windows compatibility layer | Windows runtime; non-Windows run explicitly refuses execution |
| Gameplay target | Unverified | Unverified |

The selection is an engineering judgment based on title-specific services,
streaming and the GPU boundary, supported by source inspection. Upstream's
Windows report is not our own reproduction and does not demonstrate Linux
support or playable world rendering. The updated baseline includes upstream shader numerical fixes (36eb0b4/0b499d7), resource lifetime/descriptor fixes (8c26061), and the XMA zero-length guard (PR #14). Tasks focus on remaining gaps and validation. No 30 FPS gameplay result is claimed.

Primary references:

- [Pinned status and limitations](https://github.com/Marvisak/VIctory/blob/1dd7afd4e68300e5dea41cd33a6e0b297798bb2f/README.md)
- [Console graphics objects](https://github.com/Marvisak/VIctory/blob/1dd7afd4e68300e5dea41cd33a6e0b297798bb2f/crates/victory-gpu/src/object.rs)
- [Host D3D12 objects](https://github.com/Marvisak/VIctory/blob/1dd7afd4e68300e5dea41cd33a6e0b297798bb2f/crates/victory-gpu/src/host.rs)
- [Non-Windows execution gate](https://github.com/Marvisak/VIctory/blob/1dd7afd4e68300e5dea41cd33a6e0b297798bb2f/apps/victory/src/run.rs)
- [GPU hang reports](https://github.com/Marvisak/VIctory/issues/11), [additional report](https://github.com/Marvisak/VIctory/issues/16)

VIctory source remains GPL-3.0-or-later. XCL GPL-2.0-only source stays in its
own repository and is not imported into this fork.
