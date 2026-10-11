# Amberite plan

The one plan document. It records goals and decisions, not implementation steps. When Ilai decides
something, it goes here in his terms, and whatever it replaces is removed. Open questions stay
listed as open; do not treat them as decided. Dev commands live in `scripts.md`.

Older planning files were merged into this one on 2026-10-10. They are in git history.

## The pieces

- **App** (`apps/app-frontend`, `apps/app`, `packages/app-lib`): the Modrinth App with Amberite
  added on top. Keep upstream behavior and UI. When something must differ, change the API behind
  the UI, not the UI.
- **Backend** (`apps/backend`, Cloudflare Workers + D1): a runnable clone of Modrinth's labrinth
  minus content management. It owns accounts, sessions, friends, presence, and sharing metadata:
  members, versions, where files are stored, and who may download them. Shared files live in R2 behind it. It runs locally with no Cloudflare account or billing.
- **Core** (`apps/core`, also called Copal): the self-hosted server manager. It is a product, not
  the general backend.
- **Modrinth** serves all public content. Shared versions reference Modrinth files by id.

Who talks to whom:

- App to backend and App to Core hosting: through `packages/api-client`, with Modrinth-shaped
  endpoints wherever that makes sense.
- App to Core is direct. The backend is never in that path. Core has two separate APIs:
  - **Hosting API** at `/hosting`, shaped like Modrinth's Archon. Only the Core's owner can use it.
    Nobody else can access a server for now; sharing a server with friends is a later overhaul.
  - **Sharing API**, later, for when Cores store shared files. It is separate from hosting and has
    its own access: the backend's metadata says who may store or fetch a file.
- Sharing uploads and downloads go through the existing native path in app-lib, kept as close to
  upstream as possible.

## Sharing and storage

There are no friend groups. You share an instance with friends, and sharing is not tied to a Core.
Someone an instance is shared with does not control it; for now they can only propose changes.

**Now: recreate Modrinth's sharing backend on R2.** The App keeps Modrinth's own sharing code and
UI. Our backend answers the same API, with our accounts, and stores uploaded files in R2. Nothing
custom on top.

- Shared exactly as Modrinth does: links for anything on Modrinth; real files for content that is
  not (mod jars, resource packs, shaders, datapacks); and the config bundle the owner picks when
  pushing.
- Limits, much tighter than Modrinth's and easy to change: 25 MB per file, 20 uploaded files per
  version, 5 MB config bundle, 100 MB stored per shared instance. An unchanged file is stored once.
- A file over a limit fails the push the way Modrinth's flow already reports a rejected upload.
- The five latest versions are kept. Files are deleted from R2 when their version is pruned.
- R2 runs locally through wrangler's emulation. No Cloudflare account, card, or hosting yet.
- The Amberite-only upload code is removed: the `sharing-storage` program, the two local storage
  processes, local copies of pushed versions, and resumable uploads.

Later, in rough order of intent:

- Show in the push UI which files are over a limit, before pushing.
- Pins, once a version history view exists. KubeJS scripts in the bundle.
- Stop uploading files at all where possible: reference where a file already lives and make sure
  it stays there, so a shared version is about half a megabyte.
- Large files not held in R2 are listed as needed and pulled from another App that has them.
- If you own a Core, share to it directly so friends fetch from your Core. Further out, files on
  other people's always-on Cores (never desktop Cores), contribution on by default with opt out
  in settings, a NAS as overflow, and encryption so a Core owner cannot read others' files.

## Privacy

Files in R2 are readable by whoever operates the backend. Once files live on other people's Cores,
a Core owner must not be able to read them; that needs encryption, which is deferred. Analytics
can be opted out of.

## Linked servers

A server is a hosted world of an instance. One instance can have many. A linked server takes its
loader, game version, and content from the instance and keeps its own world, files, backups, and
its own additions (mods and datapacks added on the server itself).

How it works: the App talks to the Core directly, and the backend is never involved. The App
exports the instance as a `.mrpack` and uploads it to the Core through the hosting API. Core's
engine installs it, and on a later upload applies only what changed. This works for instances
that are not shared with anyone and uses none of our storage.

- Create a server from Instance > Worlds, which opens server setup at its last step (world
  settings), or pick "From instance" in server setup. Only on a Core you own.
- Push: an instance with a linked server has the push buttons. "Push and restart" is the primary
  action. If players are online it asks: restart now, apply on next stop, or cancel. "Push update"
  is secondary and never restarts. If the instance is also shared, the same push also publishes to
  friends. An instance with no friends and no server has no push button.
- Core applies an update only while the server is stopped. A push to a running server waits and
  applies when it stops. Only the latest pushed state is applied.
- If the Core is unreachable when pushing, the server is not updated; the App says so, and the
  server's Content page has "Update from instance" to do it later.
- Client-only content is not installed on the server but is still listed. Unknown custom mods are
  installed. Inherited content cannot be disabled yet.
- An update removes only files the instance provided before. Additions made on the server are
  never touched.
- Core reports the instance through the hosting API's existing content and modpack routes
  (install content, unlink, update, the content list with its linked pack and from-pack flags), so
  an instance is one more kind of pack source. There are no separate linked-server endpoints.
- Worlds lists linked servers with a Linked tag, their state (running, offline, updating), and
  Manage. The server Content page shows the instance's content read-only in Modrinth's modpack
  card, titled with the instance's name, with the server's additions in a separate list. Unlink
  is in the card's menu. A server can be deleted.

Later:

- Skip publishing to friends when a change only matters to the server (only server-side mods
  changed), so server-only changes use no backend storage and prompt nobody.
- Friends see an instance's servers in Worlds, from a small list of name and address kept with
  the shared instance.
- Update progress in the Tasks panel. History views, overrides, branches, forced resource packs,
  and creating a server on someone else's Core.

## Accounts, Core auth, pairing

- Accounts work like Modrinth's, served by the backend.
- Core checks account tokens against that same account system. The hosting API is always on and
  requires login.
- A Core belongs to one account, its owner. Owner only: no members, roles, or invites.
- Pairing is how a Core gets its owner. An unowned Core shows an 8-character code in its terminal,
  valid 15 minutes, and makes a new one when it expires. You type the code into "Pair a Core" in
  the App and that Core belongs to your account. An unowned Core refuses everything else.
- The backend keeps each account's list of Cores and where they are, so the App finds them on any
  computer you sign in on. After pairing the App still talks to the Core directly.
- How a code is trusted: the App claims the code with the backend, then sends the code and the
  account token to the Core. The Core checks the code itself and asks the backend whether that
  account claimed it. Only then does it store the owner. In that answer the backend issues the
  Core its own key (only the hash is stored), which the Core uses for its later calls.
- A Core reports its own address, on every start. By default that is the address and port it
  listens on, or the machine's local network address when it listens on every interface.
  `AMBERITE_PUBLIC_URL` overrides it. Nobody types an address. It must be HTTPS unless it is
  loopback or a private network address.
- An account can have many Cores, treated as one: their servers form one list, and creating a
  server asks which Core only when there is more than one.
- Unpair is in the App under Settings, Cores. The Core leaves the account's list, forgets its
  owner and key, and shows a new code. Its servers and files stay. If the Core is off at the
  time, it finds out the next time it starts.
- Wrong codes are limited to 5 per account in 10 minutes, on the backend and on the Core. A live
  code cannot be registered by another Core.
- A Core that was owned before pairing existed keeps its owner but must be paired once to be
  listed. It shows a code that only that owner can use.

## Current state, 2026-10-10

- Accounts, friends, presence, and sharing on R2 between two Apps work locally.
- Core has login against our accounts, one API (hosting), and the engine that installs and updates
  a server from a pack. Convex, Core's second API, the legacy sync, and `apps/realtime` are gone.
  The last commit where everything worked on Convex is `762843d41`.
- The website is upstream Modrinth, used only as the local sign-in page.
- Not yet checked on screen by anyone: the merged result of the sharing and Core work together.
- Core pairing is built: backend list of Cores, pairing by code, the "Pair a Core" modal, and
  Settings, Cores. The dev setting that held the Core address is gone.
- Known gaps: no way to delete a server; an over-limit push shows a generic error; the App's
  network allowlist is opened to Core addresses only in the dev runner, not in a packaged build.

## Open questions

- Whether the free Workers plan is enough for R2, to check before hosting.
- Enforcing who can join a server, beyond hiding its address.
