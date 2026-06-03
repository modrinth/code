---
title: Manage servers together
summary: Add other users to your server, assign roles, and track what’s changed.
date: 2026-06-03T20:10:28.823Z
authors: ['bOHH0P9Z', 'AJfd8YH6']
---

Hey everyone,

With this release, you can now give other users access to your server! This has been one of the most requested features for Modrinth Hosting and we’re excited to finally get it out.

![The new Access tab in the Modrinth Hosting panel, featuring a list of invited users and their permissions, invite new users, and an activity log to see what changes are being made to your server and by whom.](./server-access.webp)

## TL;DR

- Add users to your server
- Set permission roles
- View activity log

## Invite your friends

You can now give other users access to your server so they can help manage content, start the server, and more. To invite someone, just enter their Modrinth username and they’ll receive an invite by email or as a notification in the app if they’re signed in.

Alongside this release, we’ve also improved state syncing between the website panel, app, and other users, so everything should stay up to date in real time.

![A pop-up modal for adding a user to your server. Search by Modrinth username, select their role (editor or limited), and an option to also send them a friend request.](./add-user-modal.webp)

## Permission roles

When adding someone to your server, you can choose what level of access they have. There are three roles, with each role inheriting the permissions of the previous one:

- **Owner:** Full access to the server including billing (you)
- **Editor:** Manage content, files, backups, settings, and more
- **Limited:** Start, stop, and view the server without making changes

You can find a full permission breakdown below:

| Permission               | Owner | Editor | Limited |
| ------------------------ | ----- | ------ | ------- |
| Start / stop server      | ✅    | ✅     | ✅      |
| Execute commands         | ✅    | ✅     | ❌      |
| Edit settings            | ✅    | ✅     | ❌      |
| Edit installation        | ✅    | ✅     | ❌      |
| Manage content           | ✅    | ✅     | ❌      |
| Manage files             | ✅    | ✅     | ❌      |
| Create & restore backups | ✅    | ✅     | ❌      |
| Invite users             | ✅    | ❌     | ❌      |
| Reset server             | ✅    | ❌     | ❌      |
| Manage billing           | ✅    | ❌     | ❌      |

## See what changed

Along with adding users, we’ve introduced an activity log. This is a chronological history of actions related to your server so you can see what changed, who changed it, and when it happened. Some actions are grouped together, like updating multiple projects at once, to keep things easier to read.

You can select a time timeframe and filter by user or action type if you’re looking for something specific.

![The activity log section of the Access tab, where you can see the user that performed an action on the left column, the action that was performed in the center, and the time it happened on the right.](./activity-log.webp)

—

Thank you for your continued support! 💚
