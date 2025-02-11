export type Product = 'web' | 'servers' | 'api' | 'app'

export type VersionEntry = {
  date: string
  product: Product
  version?: string
  body: string
}

const VERSIONS: VersionEntry[] = [
  {
    date: `2025-02-11T14:00:00-09:00`,
    product: 'web',
    body: `### Added
- Added a changelog page to view recent changes to Modrinth.`,
  },
  {
    date: `2025-02-10T14:00:00-09:00`,
    product: 'web',
    body: `### Improvements
- The license selector in project settings has been updated to make selecting a license a clearer process. (Thanks to [Erb3](https://github.com/modrinth/code/pull/3225))`,
  },
  {
    date: `2025-02-10T08:00:00-09:00`,
    product: 'servers',
    body: `### Added
- You can now search and filter through your server's console in the Overview tab, jump to specific results to see the log in context, select them, and copy them.
- You can now drag and select any number of lines in the console, copy them. and view them formatted.
- Hide your server's \`.modrinth.gg\` custom URL using the new **Hide subdomain label** toggle in Options > Preferences.
- The Content page has been updated to make managing your server's mods and plugins easier than ever. Now, only versions that are available for your server's Minecraft version and platform are shown by default, and you can now show beta and alpha versions in the selector.
### Improvements
- The Overview page loads faster.
- The Options > Properties page loads faster.
- The server hardware graphs in the Overview page have been rewritten to improve power efficiency and fix rendering bugs.
- The modpack selector in Options > Platform now shows more information about a modpack, like its tags, downloads, and followers.
- Reinstalling your server no longer requires the browser to refresh the page in order to work properly. We now also lock more options while a server installs to prevent your server from bricking itself.
- The server console has been rewritten to implement proper batching. All performance issues with the console previously have now been fixed.
- An error state has been added in the server list if servers are unable to be fetched.
- Sorting in the Files tab is now accessible by clicking the column headers.
- Backing up a server and erasing all its data simultaneously in the Platform page now works as expected.
- Opening a platform modal, then opening another, no longer causes versions of that platform to fail to load.`,
  },
  {
    date: `2025-02-06T10:00:00-09:00`,
    product: 'app',
    version: `0.9.3`,
    body: `### Improvements
- Prevent ads from being able to open additional windows.
- Fixed update checking only checking for mod updates.
- Fixed issue importing newer Prism instances.
- Fixed issue where instances get stuck "Installing" forever when the app is closed during an install.
- Minecraft profile is now updated every time the user's token is refreshed.
- Improved ability for package managers to update Modrinth App by skipping the updater at runtime with an environment variable.`,
  },
  {
    date: `2025-02-02T14:00:00-09:00`,
    product: 'web',
    body: `### Improvements
- The report form has been updated to walk you through the report process better and clarify some things like that the form is for Modrinth rules and terms violations, not for bug reports or DMCA takedowns.

![A screenshot of the new report form on Modrinth, using Iris Shaders as an example. The title says "Report Iris Shaders to moderators". Below that, it says "Please report violations of Modrinth Rules or Terms of Use. Examples include malicious, spam, offensive, deceptive, misleading, and illegal content. This form is not for bug reports or DMCA takedowns (See our Copyright Policy)." Then, there is a form that asks "Which of Modrinth's rules is this project violating?" with many options: Spam, Reuploaded work, Inappropriate, Malicious, Name-squatting, Poor description, Invalid metadata, Other. Reuploaded work is selected. Below that, is a note in an orange box with a warning icon: "Please note that you are *not* submitting a DMCA takedown request, but rather a report of reuploaded content. If you meant to file a DMCA takedown request (which is a legal action) instead, please see our Copyright Policy." Then, it asks you to provide additional context, including links and images, with a text editor and a submit button at the bottom.](https://cdn-raw.modrinth.com/changelog/web/2025-02-02/reports.jpg)`,
  },
  {
    date: `2024-12-25T14:00:00-09:00`,
    product: 'app',
    version: `0.9.2`,
    body: `### Improvements
- Prevent ads from being able to play audio.`,
  },
  {
    date: `2024-12-24T22:00:00-09:00`,
    product: 'app',
    version: `0.9.1`,
    body: `### Added
- Added filter to filter projects by disabled.
- Re-added back/forward navigation buttons.
### Improvements
- Fixed environment tags missing from search.
- Fixed an issue where ads could play audio.
- Changed content enable/disable buttons to toggle switches.
- Show "install" button at all time on project cards.
- Fixed issue where cards would shrink when clicking button inside them causing click not to register.
- Made sidebar hide instantly.`,
  },
]

export function getChangelog() {
  return VERSIONS
}
