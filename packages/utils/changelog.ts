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
    date: `2025-06-30T19:15:00-07:00`,
    product: 'web',
    body: `### Added
- Added news page, with all our old blog posts now hosted on our website.

### Improvements
- Changed download count rounding to be more precise.
- Fixed Creator Monetization Program page to show accurate information again.`,
  },
  {
    date: `2025-06-30T19:15:00-07:00`,
    product: 'servers',
    body: `### Improvements
- Progress will now show when installing Modrinth Pack (.mrpack) files.
- Fixed storage stats not linking to Files page.
- Fixed missing icons in some places.`,
  },
  {
    date: `2025-06-29T16:30:00-07:00`,
    product: 'web',
    body: `### Improvements
- Removed ads for logged in users.
- Fixed tooltips being unreadable sometimes.`,
  },
  {
    date: `2025-06-26T11:00:00-07:00`,
    product: 'servers',
    body: `### Improvements
- Fixed support bubble overlapping notifications sometimes.
- Fixed race condition when creating backups.`,
  },
  {
    date: `2025-06-26T11:00:00-07:00`,
    product: 'web',
    body: `### Added
- Added a dismissable Modrinth Servers promotion to project Download interface to inform users of the service's availability.

### Improvements
- Added colors for the newly added legacy mod loaders
- Improved file upload error message in some places.`,
  },
  {
    date: `2025-06-16T11:00:00-07:00`,
    product: 'web',
    body: `### Improvements
- Rolled out hotfixes with the previous days' updates.
- Failed subscriptions can now be cancelled.`,
  },
  {
    date: `2025-06-16T11:00:00-07:00`,
    product: 'servers',
    body: `### Improvements
- Improved error handling.
- Rolled out hotfixes with the previous days' updates.'`,
  },
  {
    date: `2025-06-15T16:25:00-07:00`,
    product: 'servers',
    body: `### Improvements
- Fixed installing modpacks from search.
- Fixed setting subdomains.`,
  },
  {
    date: `2025-06-15T14:30:00-07:00`,
    product: 'servers',
    body: `### Improvements
- Fixed various issues with the panel loading improperly in certain cases.
- Fixed CPU icon being smaller than the rest.
- Server panel performance should be a little faster now.`,
  },
  {
    date: `2025-06-15T14:30:00-07:00`,
    product: 'web',
    body: `### Improvements
- Creator analytics charts will now show up to 15 projects in a tooltip instead of 5.
- Made certain scrollable containers not have a fixed height, and allow them to be smaller if they have fewer items. (Contributed by [Erb3](https://github.com/modrinth/code/pull/2898))
- Made organizations sort consistently alphabetically. (Contributed by [WorldWidePixel](https://github.com/modrinth/code/pull/3755))
- Clarified the 'File too large' error message when uploading an image larger than 1MiB in the text editor. (Contributed by [IThundxr](https://github.com/modrinth/code/pull/3774))`,
  },
  {
    date: `2025-06-03T14:35:00-07:00`,
    product: 'servers',
    body: `### Added
- Added support for servers in Europe.
- Added server setup for new servers upon opening the panel for the first time.`,
  },
  {
    date: `2025-06-03T14:35:00-07:00`,
    product: 'web',
    body: `### Improvements
- Overhauled Modrinth Servers purchase flow.
- Added the ability to donate creator rewards to charity.`,
  },
  {
    date: `2025-05-08T09:00:00-07:00`,
    product: 'servers',
    body: `### Added
- Added the ability to extract .zip files in the Files page.
- Added the ability to extract a remote .zip file from a URL, or from a CurseForge modpack version URL.
- Dependencies will now automatically be installed when installing a mod from Modrinth.`,
  },
  {
    date: `2025-05-08T09:00:00-07:00`,
    product: 'web',
    body: `### Improvements
- Fixed game version and loader selection when editing a version only showing up to 6 selected items, making it frustrating to remove a specific version.
- Fixed organization URLs being case sensitive. (Contributed by [IMB11](https://github.com/modrinth/code/pull/3621))
- Fixed notifications not loading sometimes. (Contributed by [IMB11](https://github.com/modrinth/code/pull/3624))
- Fixed marking all notifications as read. (Contributed by [IMB11](https://github.com/modrinth/code/pull/3624))
- Fixed relative time sometimes having poor rounding, and sometimes showing future dates unexpectedly. (Contributed by [IMB11](https://github.com/modrinth/code/pull/3612))
- Fixed localized pricing not showing in some cases for Modrinth+. (Contributed by [IMB11](https://github.com/modrinth/code/pull/3623))
- Fixed Modrinth changelog page link back to the full changelog not being clickable sometimes. (Contributed by [ThatGravyBoat](https://github.com/modrinth/code/pull/3593))
- Fixed analytics 'Views' tab not having the updated display for 'Other' regions.`,
  },
  {
    date: `2025-05-01T18:30:00-07:00`,
    product: 'web',
    body: `### Improvements
- Added a button to switch Modrinth+ billing between monthly and yearly.
- Updated Modrinth App marketing page screenshots.`,
  },
  {
    date: `2025-05-01T18:10:00-07:00`,
    product: 'app',
    version: `0.9.5`,
    body: `### Improvements
- Fixed certain mods with a large number of versions not being able to be installed or load their versions.
- Fixed server descriptions not being rendered with the Minecraft font.
- When installing a modpack, the page will now change to the instance once it's created.
- Last played time for existing servers will now be loaded from log files instead of displaying "Never played".
- Home page's "Jump back in" section will now display in multiple columns when the screen is very large.
- Added the ability to launch the instance from a World entry on the Home page.
- Added the ability to hide a world from the Home page.
- Added an appearance setting to only show instances instead of worlds on the Home page.
- Fixed Home page rendering being blocked by server pings.
- Fixed Home page sometimes not sending the correct protocol version when fetching server data.
- Fixed server data not being loaded immediately after adding it.
- Fixed Worlds tab filters showing up in the wrong situations.
- Fixed new Singleplayer worlds not being added to the Worlds tab without a refresh.
- Fixed an excessive number of items showing up in "Jump back in" sometimes.
- Fixed critical error modal cutting off long single-line error messages.`,
  },
  {
    date: `2025-04-29T08:20:00-07:00`,
    product: 'web',
    body: `### Improvements
- Fixed multiple 'Hidden' countries showing up in analytics, updated flag and changed to 'Other'.`,
  },
  {
    date: `2025-04-28T19:45:00-07:00`,
    product: 'servers',
    body: `### Added
- Added support for installing snapshot versions of Minecraft.

### Improvements
- Fixed a bug where the loader version wouldn't update after changing Minecraft version if the old loader version did not support the newly selected Minecraft version.
- Improved wording of unprovisioned servers, to inform people that newly purchased servers may be unprovisioned for a small amount of time.`,
  },
  {
    date: `2025-04-28T19:45:00-07:00`,
    product: 'web',
    body: `### Improvements
- Combined Apple Silicon and Intel download links for Modrinth App into one link.
- Added an option to copy a permanent link (using IDs instead of changeable slugs) to projects, users, and organizations.
- Fixed overflow issue with dev-mode Maven coordinates widget, and changed wording.`,
  },
  {
    date: `2025-04-26T19:15:00-07:00`,
    product: 'app',
    version: `0.9.4`,
    body: `### Added
- Added a Worlds tab to instances that allow you to manage your worlds and servers directly from the app
  - Server ping, play count, and descriptions will all show before having to open the game.
  - Add, edit, and remove servers
  - Edit and delete singleplayer worlds
- Added 'Quick Play' functionality, allowing you to jump straight into worlds and servers from the app
- Added new launcher-log.txt file to log stdout to a file, similar to the Minecraft Launcher which can be helpful for debugging.

### Improvements
- Overhauled the 'Jump back in' section of the Home page, showing recently played worlds instead of just instances.
  - This supports the new 'Quick Play' functionality, allowing you to jump back into your recent worlds (on supported Minecraft versions 1.20+)
- Improved the fatal error dialog to show debug information more prominently, and allow you to copy it to share with support.
- Fixed the display of critical announcements that has been broken since 0.9.0.
- Fixed content authors not displaying properly on Content page if the content was owned by an organization.
- Fixed random errors on the Library page, causing no instances to load.
- Fixed .mrpack file association on Linux.
- Fixed occasional failures when moving app directory between disks.`,
  },
  {
    date: `2025-04-25T19:45:00-07:00`,
    product: 'web',
    body: `### Improvements
- Added feedback survey upon Modrinth Servers cancellation.
- Added FAQ question about the currency for Modrinth Servers subscription payments.`,
  },
  {
    date: `2025-04-18T22:30:00-07:00`,
    product: 'web',
    body: `### Improvements
- Updated Modrinth Servers marketing page to be accurate to post-Pyro infrastructure.`,
  },
  {
    date: `2025-04-17T02:25:00-07:00`,
    product: 'servers',
    body: `### Improvements
- Completely overhauled the Backups interface and fixed them being non-functional.
  - Backups will now show progress when creating and restoring.
  - Backups now have a "Prepare download" phase, which will prepare a backup file for downloading.
  - You can now cancel a backup in progress and retry a failed backup.
- When a backup is in progress, you will no longer be allowed to modify the modpack or loader.
- Removed the ability to create backups on install automatically, and replaced with a notice that you may want to create a backup before installing a new modpack or loader. This is because the previous implementation of backup on install was unreliable and buggy. We are working on a better implementation for this feature and plan for it to return in the future.
- Temporarily disabled auto backups button, since they are currently not working.`,
  },
  {
    date: `2025-04-15T16:35:00-07:00`,
    product: 'servers',
    body: `### Added
- Added ability to send surveys to customers in the panel via notices.

### Improvements
- Added titles to notices.`,
  },
  {
    date: `2025-04-12T22:10:00-07:00`,
    product: 'servers',
    body: `### Added
- Added ability to notify customers in the panel with notices concerning their servers.`,
  },
  {
    date: `2025-04-12T22:10:00-07:00`,
    product: 'web',
    body: `### Improvements
- Fix missing dropdown icon in publishing checklist.`,
  },
  {
    date: `2025-04-01T21:15:00-07:00`,
    product: 'web',
    body: `### Added
- Reverted Modrinth Pizza due to issues in pizza processing leading to users being redirected to [this page](https://modrinth.com/pizza/error).
- Fixed bug causing theme color to turn orange and corners turn sharp.`,
  },
  {
    date: `2025-04-01T12:00:00`,
    product: 'web',
    body: `### Added
- Added Modrinth Pizza.`,
  },
  {
    date: `2025-04-01T00:00:00`,
    product: 'web',
    body: `### Improvements
- Fixed project version field accepting more than 32 characters.`,
  },
  {
    date: `2025-03-25T18:25:00-07:00`,
    product: 'web',
    body: `### Improvements
- Fixed random 'displayName' error on search pages on some browsers such as Firefox.
- Fixed 'Resubmit' icon in publishing checklist showing up when it hasn't been submitted before.`,
  },
  {
    date: `2025-03-25T10:40:00-07:00`,
    product: 'web',
    body: `### Improvements
- Fixed error with links on error pages.`,
  },
  {
    date: `2025-03-24T22:30:00-07:00`,
    product: 'servers',
    body: `### Improvements
- Fixed server plugin loaders not being populated when browsing for plugins
- Fixed modpack search being filtered by Minecraft version when browsing for modpacks.`,
  },
  {
    date: `2025-03-24T22:30:00-07:00`,
    product: 'web',
    body: `### Improvements
- Improved error handling, especially when the Modrinth API is down.`,
  },
  {
    date: `2025-03-13T19:30:00-07:00`,
    product: 'web',
    body: `### Improvements
- Updated Modrinth Servers marketing page, removing Pyro branding.`,
  },
  {
    date: `2025-03-12T10:15:00-07:00`,
    product: 'web',
    body: `### Improvements
- Fixed low-res icons being pixelated.
- Fixed mobile navbar hiding bottom of footer.
- Updated CMP info page to correct some incorrect information.
- Updated CCPA notice with updated information since Modrinth Servers and Modrinth+.
- Fixed review page failing under edge case.`,
  },
  {
    date: `2025-03-05T17:40:00-08:00`,
    product: 'web',
    body: `### Improvements
- Fixed moderation-end pages failing under edge cases.`,
  },
  {
    date: `2025-03-05T12:40:00-08:00`,
    product: 'web',
    body: `### Improvements
- Fixed various errors with modals for some users.
- Fixed hold R button not working on some systems.`,
  },
  {
    date: `2025-03-03T22:30:00-08:00`,
    product: 'web',
    body: `### Added
- Hold R for a random project :D

### Improvements
- Improved admin navigation and admin panels.`,
  },
  {
    date: `2025-03-02T18:45:00-08:00`,
    product: 'web',
    body: `### Improvements
- Added option to copy version IDs from the version list for project members and developer mode.
- Fixed the staff moderation checklist going off the screen.`,
  },
  {
    date: `2025-02-25T10:20:00-08:00`,
    product: 'servers',
    body: `### Improvements
- Fixed server upgrades being allowed when out of stock, despite warning.`,
  },
  {
    date: `2025-02-25T10:20:00-08:00`,
    product: 'web',
    body: `### Improvements
- Moved Minecraft brand disclaimer to bottom of footer.
- Improved clarity of the ongoing revenue period footnote on the Revenue page.
- Fixed collections without a summary being unable to be edited.`,
  },
  {
    date: `2025-02-21T13:30:00-08:00`,
    product: 'web',
    body: `### Improvements
- Collections are now sorted by creation date. (Contributed by [worldwidepixel](https://github.com/modrinth/code/pull/3286))
- Collections are no longer required to have summaries. (Contributed by [Erb3](https://github.com/modrinth/code/pull/3281))
- Fixed padding issue on revenue page.
- Fixed last modified date on Rewards Program Info page. (Contributed by [IMB11](https://github.com/modrinth/code/pull/3287))`,
  },
  {
    date: `2025-02-20T18:15:00-08:00`,
    product: 'web',
    body: `### Improvements
- Revenue page has been updated to more clearly display pending revenue and when it will be available to withdraw. (Contributed by [IMB11](https://github.com/modrinth/code/pull/3250))
- Footer will now be forced to the bottom of the page on short pages.
- Styling fixes to moderation checklist proof form.`,
  },
  {
    date: `2025-02-19T22:20:00-08:00`,
    product: 'web',
    body: `### Added
- All-new site footer with more links, better organization, and a new aesthetic.

### Improvements
- Added Dallas location to Modrinth Servers landing page.
- Updated staff moderation checklist to be more visually consistent and more dynamic.`,
  },
  {
    date: `2025-02-18T14:30:00-08:00`,
    product: 'servers',
    body: `### Added
- Links will now be detected in console line viewer modal.

### Improvements
- Initial loading of pages in the server panel are now up to 400% faster.
- Syncing and uploading new server icons no longer requires a full page refresh.
- Fix a case where opening the platform modal, closing it, and reopening it would cause the loader version to be unselected.
- Prevents an issue where, if crash log analysis fails, the Overview page would unrender.
- Suspended server listings now have a copy ID button.
- Fixed bugs from Modrinth Servers February Release.`,
  },
  {
    date: `2025-02-16T19:10:00-08:00`,
    product: 'web',
    body: `### Improvements
- Fixed spacing issue on confirmation modals.`,
  },
  {
    date: `2025-02-16T19:10:00-08:00`,
    product: 'servers',
    body: `### Improvements
- Check for availability before allowing a server upgrade.`,
  },
  {
    date: `2025-02-12T19:10:00-08:00`,
    product: 'web',
    body: `### Improvements
- Servers out of stock link now links to Modrinth Discord instead of support page.`,
  },
  {
    date: `2025-02-12T19:10:00-08:00`,
    product: 'servers',
    body: `### Added
- Added server upgrades to switch to a larger plan as an option in billing settings.`,
  },
  {
    date: `2025-02-12T12:10:00-08:00`,
    product: 'web',
    body: `### Added
- Added a 3D globe to visualize node locations to Modrinth Servers marketing page.
- Added an indicator to show when certain server plans are running low on availability.

### Improvements
- Improved out-of-stock notifications on Modrinth Servers page to be more accurate.`,
  },
  {
    date: `2025-02-11T13:00:00-08:00`,
    product: 'web',
    body: `### Improvements
- Added project issues link to report page if present.
- Added relative times for all recent versions to changelog page.
- Added header to changelog sub-pages.
- Fixed various padding issues and changelog overlapping navbar on mobile.`,
  },
  {
    date: `2025-02-11T09:00:00-08:00`,
    product: 'web',
    body: `### Added
- Added a changelog page to view recent changes to Modrinth.`,
  },
  {
    date: `2025-02-10T14:00:00-08:00`,
    product: 'web',
    body: `### Improvements
- The license selector in project settings has been updated to make selecting a license a clearer process. (Contributed by [Erb3](https://github.com/modrinth/code/pull/3225))`,
  },
  {
    date: `2025-02-10T08:00:00-08:00`,
    product: 'servers',
    version: `February Release`,
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
    date: `2025-02-06T10:00:00-08:00`,
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
    date: `2025-02-02T14:00:00-08:00`,
    product: 'web',
    body: `### Improvements
- The report form has been updated to walk you through the report process better and clarify some things like that the form is for Modrinth rules and terms violations, not for bug reports or DMCA takedowns.

![A screenshot of the new report form on Modrinth, using Iris Shaders as an example. The title says "Report Iris Shaders to moderators". Below that, it says "Please report violations of Modrinth Rules or Terms of Use. Examples include malicious, spam, offensive, deceptive, misleading, and illegal content. This form is not for bug reports or DMCA takedowns (See our Copyright Policy)." Then, there is a form that asks "Which of Modrinth's rules is this project violating?" with many options: Spam, Reuploaded work, Inappropriate, Malicious, Name-squatting, Poor description, Invalid metadata, Other. Reuploaded work is selected. Below that, is a note in an orange box with a warning icon: "Please note that you are *not* submitting a DMCA takedown request, but rather a report of reuploaded content. If you meant to file a DMCA takedown request (which is a legal action) instead, please see our Copyright Policy." Then, it asks you to provide additional context, including links and images, with a text editor and a submit button at the bottom.](https://cdn-raw.modrinth.com/changelog/web/2025-02-02/reports.jpg)`,
  },
  {
    date: `2025-01-28T19:00:00-08:00`,
    product: 'web',
    body: `### Improvements
- The UI for analytics has been updated to make it way more useful. What was previously called "Last month" really meant "Previous 30 days". Now, many more options have been added such as:
  - "This month" which refers to the current calendar month (Jan 1st - Jan 28th, currently)
  - "Last month" which refers to the previous calendar month (currently, Dec 1st thru Dec 31st)

Contributed by [IMB11](https://github.com/modrinth/code/pull/1301).`,
  },
  {
    date: `2025-01-10T09:00:00-08:00`,
    product: 'servers',
    version: 'January Release',
    body: `### Added
- Added drag & drop upload support for mod and plugin files on the content page.
- Added a button to upload files to the content page.
- Added extra info (file name, author) to each mod on the content page.
- Show number of mods in search box.
- Adds a "No mods/plugins found for your query!" message if nothing is found, with a button to show everything again.

### Improvements
- The content page layout has been enhanced, now showing the file name and author of each installed item.
- You can now upload directly from the content page, instead of having to go to the Files page.
- Auto-backup now lists options in a dropdown instead of number input.
- Auto-backup 'Save changes' button now disables when no changes are made and backups are off.
- Servers dropdowns now have rounded corners on the last elements for consistency.
- Added support for more suspension reasons.
- Will now show resubscribe button on servers when payment status is "failed" instead of just "cancelled".
- Tweak button styles for consistency.
- Only scroll to the top of the mod/plugin list when searching if already scrolled down.
- Tweak content page mobile UI.`,
  },
  {
    date: `2025-01-10T09:00:00-08:00`,
    product: 'web',
    body: `### Improvements
- Tags on project pages are now clickable to view other projects with that tag (Contributed by [Neddo](https://github.com/modrinth/code/pull/3126))
- You can now send someone a link to the download interface with a specific version and loader selected, like so: https://modrinth.com/mod/sodium?version=1.21.2&loader=quilt#download (Contributed by [AwakenedRedstone](https://github.com/modrinth/code/pull/3138))`,
  },
  {
    date: `2024-12-26T22:05:00-08:00`,
    product: 'servers',
    body: `### Added
- Added ability for users to clean install modpacks when switching versions.

### Improvements
- New status bar in ServerListing that shows suspension reasons/upgrade status.
- Displays a new screen for servers that are being upgraded.`,
  },
  {
    date: `2024-12-25T14:00:00-08:00`,
    product: 'app',
    version: `0.9.2`,
    body: `### Improvements
- Prevent ads from being able to play audio.`,
  },
  {
    date: `2024-12-24T22:00:00-08:00`,
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
    date: `2024-12-23T22:16:00-08:00`,
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
  {
    date: `2024-12-21T16:00:00-08:00`,
    product: 'servers',
    body: `### Added
- Drag and drop anything in the file manager.
- Added file upload queue status bar.
- Added support for parallel file uploads to upload multiple files faster.
- Added ability to cancel in-progress file uploads.
- Creation dates are now displayed for files.
- Can now sort by most recently created files
- YAML and TOML files now support syntax highlighting
- Find and replace support in files editor

### Improvements
- Files list renders up to 200% faster.
- Image viewer performance improvements, improved UI, and better handling of large-to-display images.
- UI inconsistency fixes.
- When reinstalling the loader, the current Minecraft version is automatically selected.
- Allow user to clean install modpacks on the modpack search page.
- Fixed 'Change platform' button leading to the wrong page on a vanilla server.`,
  },
  {
    date: `2024-12-11T22:18:45-08:00`,
    product: 'servers',
    version: `December Release`,
    body: `### Added
- Expanded loader support to include **Paper** and **Purpur** servers, offering fully native plugin compatibility.
- A live chat button has been added to the bottom right of all server pages, making it easier for customers to contact our support team.
- Automatic backups are now *rolling*. This means older backups will be deleted to make space for new backups when a new one is being created. You can also now **lock** specific backups so that they don't get deleted by the automatic backup process.
- Users can now easily create backups before reinstalling a server with a different loader.

### Improvements
- The Platform options page has been completely redesigned to streamline user interactions and improve overall clarity.
- Suspended servers now display a clear "Suspended" status instead of a confusing "Connection lost" message, allowing users to easily check their billing information.
- The console has been internally reworked to improve responsiveness and prevent freezing during high-volume spam.
- Resolved CPU usage readings that previously exceeded 100% during high-load scenarios. CPU usage is now accurately normalized to a 0–100% range across all cores.
- Corrected CPU limit settings for some servers, potentially improving performance by up to half a core.
- Fixed an issue preventing server reinstallation when at the maximum backup limit.
- Resolved installation and runtime problems with older Minecraft versions.
- Added missing dynamic system libraries to our images, ensuring compatibility with the vast majority of mods.
- Implemented several additional bug fixes and performance optimizations.
- Removed Herobrine.

### Known Issues
- Backups may occasionally take longer than expected or become stuck. If a backup is unresponsive, please submit a support inquiry, and we'll investigate further.`,
  },
].map((x) => ({ ...x, date: dayjs(x.date) }) as VersionEntry)

export function getChangelog() {
  return VERSIONS
}
