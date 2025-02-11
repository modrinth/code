import dayjs from 'dayjs'

export type Product = 'web' | 'servers' | 'api' | 'app'

export type VersionEntry = {
  date: dayjs.Dayjs
  product: Product
  version?: string
  body: string
}

const VERSIONS: VersionEntry[] = [
  {
    date: `2025-02-11T13:00:00-09:00`,
    product: 'web',
    body: `### Improvements
- Added project issues link to report page if present.
- Added relative times for all recent versions to changelog page.
- Added header to changelog sub-pages.
- Fixed various padding issues and changelog overlapping navbar on mobile.`,
  },
  {
    date: `2025-02-11T09:00:00-09:00`,
    product: 'web',
    body: `### Added
- Added a changelog page to view recent changes to Modrinth.`,
  },
  {
    date: `2025-02-10T14:00:00-09:00`,
    product: 'web',
    body: `### Improvements
- The license selector in project settings has been updated to make selecting a license a clearer process. (Contributed by [Erb3](https://github.com/modrinth/code/pull/3225))`,
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
    date: `2025-01-28T19:00:00-09:00`,
    product: 'web',
    body: `### Improvements
- The UI for analytics has been updated to make it way more useful. What was previously called "Last month" really meant "Previous 30 days". Now, many more options have been added such as:
  - "This month" which refers to the current calendar month (Jan 1st - Jan 28th, currently)
  - "Last month" which refers to the previous calendar month (currently, Dec 1st thru Dec 31st)

Contributed by [IMB11](https://github.com/modrinth/code/pull/1301).`,
  },
  {
    date: `2025-01-10T09:00:00-09:00`,
    product: 'servers',
    body: `### Improvements
- The content page layout has been enhanced, now showing the file name and author of each installed item.
- You can now upload directly from the content page, instead of having to go to the Files page.`,
  },
  {
    date: `2025-01-10T09:00:00-09:00`,
    product: 'web',
    body: `### Improvements
- Tags on project pages are now clickable to view other projects with that tag (Contributed by [Neddo](https://github.com/modrinth/code/pull/3126))
- You can now send someone a link to the download interface with a specific version and loader selected, like so: https://modrinth.com/mod/sodium?version=1.21.2&loader=quilt#download (Contributed by [AwakenedRedstone](https://github.com/modrinth/code/pull/3138))`,
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
  {
    date: `2024-12-23T22:16:00-09:00`,
    product: 'app',
    version: `0.9.0`,
    body: `### Improvements
- New, updated design that brings the app in-line with the website.
  - A dynamic sidebar that adjusts to the most relevant content for each page, and keeps the ad in a consistent place instead of moving all around your screen.
  - More organized settings interfaces that makes each option clearer and easier to find.
  - Managing your content is much easier with enhanced filters for content types or checking for updates.
  - Content discovery has been overhauled and now has fully-featured project pages that match the website.
  - Instances now show your total play time, and will show the last time you played on the Home screen.
  - The library page now gives responsive feedback as instances are installing.
- The beginnings of a Friends system. In the future, you will be able to share the instances you’re playing and invite them to servers.
- Access your most recent instances with ease with Quick Instances.
- Fixed “Database is locked” errors on devices with slow disks.
- Fixed a few edge cases where API downtime could lead to an invalid state.`,
  },
].map((x) => ({ ...x, date: dayjs(x.date) }) as VersionEntry)

export function getChangelog() {
  return VERSIONS
}
