---
title: Sync settings across instances
summary: Keep game options, servers, resource packs, and more the same across your instances.
date: 2026-09-07T20:00:00+01:00
authors: [bOHH0P9Z, AJfd8YH6]
---

Hey everyone,

With this update, we’ve introduced syncing across your instances, along with a new screenshots page for viewing all your screenshots in one place.

### TL;DR

- New Screenshots page with a built-in viewer and editor
- Sync game options, servers, resource packs, command history, and creative hotbars across instances
- Override synced content on individual instances when needed

## Screenshot page

To start, we’ve added a new screenshots page to the sidebar, bringing screenshots from all your instances together.

By default, screenshots are grouped by date, but you can also group them by instance or create your own groups to organize them however you want. You can drag and drop screenshots between groups too, just like you can with instances.

![The screenshots page with screenshots grouped across instances.](./screenshots-page.webp)

In your app settings, you can also enable a screenshots tab on individual instances if you’d prefer to view them there. We’ve also added more control over instance tabs, letting you enable or disable worlds and files.

### Screenshot viewer

Screenshots can be opened in a larger view so you can take a closer look. From there, you can edit, copy, show in folder, or delete them.

![The screenshot viewer with actions to edit, copy, show in folder, or delete a screenshot.](./screenshot-viewer.webp)

### Screenshot editor

Screenshots can also be marked up and saved as a new copy. The editor includes tools for cropping, drawing, highlighting, adding shapes, and more.

<div class="video-wrapper mb-8">
	<video autoplay loop muted playsinline>
		<source src="/news/article/sync-settings/screenshot-editor.webm" type="video/webm" />
	</video>
</div>

## Synced settings

One thing that has always sucked about playing modded is that every instance can have completely different game settings, servers, and resource packs. These are often things you want to keep the same every time you play, but until now you’ve had to manage them manually.

Syncing keeps these the same across your instances. Syncing is bi-directional, so changes made from any instance will propagate across all your other instances.

![The app settings for syncing content across instances.](./synced-settings.webp)

Syncing will be disabled by default for existing users and enabled for new users. When enabling it, we’ll ask which instance you want to use as the starting point.

<div class="video-wrapper mb-8">
	<video autoplay loop muted playsinline>
		<source src="/news/article/sync-settings/sync-source.webm" type="video/webm" />
	</video>
</div>

You can also override syncing for individual instances in their settings. For example, you can start with your default synced game options and then customize them for just that instance, or try a new resource pack without affecting the rest.

### Game options

Synced game options work at the individual setting level within the `options.txt` file. We’ve chosen a default set of settings to sync, but you can fine-tune these however you want.

When a setting is synced, it can be edited either in-game or from within the app. If you create a linked modpack instance that comes with its own `options.txt`, any fields you’ve chosen to sync will override those values while leaving the rest untouched.

![Individual game options that can be configured and synced across instances.](./game-options.webp)

Any settings added to `options.txt` by mods will appear under custom settings and be synced by default. These will only apply to instances where those mods are installed.

### Multiplayer servers

Your multiplayer server list can also stay synced across your instances. You can add or edit servers in-game or from within the app. Servers will not be synced to linked server instances, since these projects often ship with a server specific to that project.

### Resource packs

Resource pack syncing is the only option disabled by default for all users.

When enabled packs will only sync to compatible instances, meaning they must support the game version and loader used by that instance. Enabling, disabling, and deleting actions will sync across instances, while version changes will not.

Resource packs included with a modpack will not be synced to your other instances. Only additional resource packs you install yourself will be synced.

<div class="video-wrapper mb-8">
	<video autoplay loop muted playsinline>
		<source src="/news/article/sync-settings/resource-packs-sync.webm" type="video/webm" />
	</video>
</div>

### Misc

We’ve also added syncing for command history and creative hotbars. Command history can also be viewed and edited from your app settings.

Enjoy!
