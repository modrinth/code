---
title: Generator plugin
---

The generator plugin creates static files from API responses to increase performance and perform tasks that would not be possible on the client. It regenerates files every 7 days, or when the plugin settings change.

### Current options

-   `projectColors` (false) generates colors for every project
-   `tags` (false) copies & parses tags from API
-   `gameVersions` copes game versions from API
-   `landingPage` gets icon urls for top 100 mods

> All options are disabled by default

## Configuration

```js
import Generator from 'omorphia/plugins/generator';

/** @type {import('@sveltejs/kit').Config} */
const config = {
    kit: {
        vite: {
            plugins: [
                Generator({
                    projectColors: true,
                    tags: true,
                    gameVersions: true,
                    landingPage: true,
                }),
            ],
        },
    },
};

export default config;
```
