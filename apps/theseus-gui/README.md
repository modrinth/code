<img src="https://github.com/modrinth/theseus/assets/6166773/51d1ca87-05c0-445a-bd18-ddd1117f7f12" alt="modrinth app: theseus (desktop app)">

# Modrinth App

<img src="https://cdn-raw.modrinth.com/app-landing/app-screenshot.webp" alt="Screenshot of the Modrinth App's home page" align="right" width="50%">

The Modrinth App, codenamed theseus, is a modern launcher for Minecraft: Java Edition with a clean look, easy-to-use interface, and deep integration into Modrinth services.

### Features
- One-click installation of modpacks
- Automatic management of Java versions
- Windows, Mac, and Linux[^1] support
- Import your instances from CurseForge, Prism[^2], ATLauncher, MultiMC[^2], or GDLauncher
- Supports offline play once you've authenticated with your Minecraft account at least once
- Fully open source under GPLv3[^3]!

[^1]: While Linux is supported, due to the wide range of distributions out there, your mileage may vary with how well the Modrinth App works on your system. We officially distribute `.deb` and `.AppImage` packages, but third party packages have been created for a number of other package platforms. Additionally, some have reported lag issues running on Linux, we believe this to be due to an upstream Tauri issue, which we hope improves with further development.

[^2]: Certain features of the OneSix format used by Prism and MultiMC are not yet supported, so some instances may not import correctly, primarily on older Minecraft versions or unsupported mod loaders.

[^3]: Modrinth's logos, branding, and other trademarks are not free for use, see the [licensing section](#license) for more information.

## Contributing
You're welcome to help contribute to the Modrinth App if you'd like! Please review our [contribution guide](https://support.modrinth.com/en/articles/8802215-contributing-to-modrinth) before attempting to contribute or make a pull request though.

## Development
To get started, install [pnpm](https://pnpm.io/), [Rust](https://www.rust-lang.org/tools/install), and the [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites/#installing) for your system. Then, run the following commands:

```
cd theseus_gui
pnpm install
pnpm run tauri dev
```

Once the commands finish, you'll be viewing a Tauri window with Nuxt.js hot reloading.

You can use `pnpm run lint` to find any eslint problems, and `pnpm run fix` to try automatically fixing those problems.

## License
The source code of the theseus repository is licensed under the GNU General Public License, Version 3 only, which is provided in the file [LICENSE](https://github.com/modrinth/theseus/blob/master/LICENSE). However, some files are licensed under a different license.

Any files depicting the Modrinth branding, including the wrench-in-labyrinth logo, the landing image, and variations thereof, are licensed as follows:
> All rights reserved. © 2020-2024 Rinth, Inc.

Forking is permitted under the GPLv3, however do be aware that you must remove all Modrinth branding, including logos, brand colors, background images, or anything else that is related to trademarks or copyrights held by Rinth, Inc.
