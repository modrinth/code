# Playable release polish — short plan

Branch: `cloud/playable-release-polish`

## A — Social / Friends / Account / Share
1. Friends: add `declineOwyxFriend`, clearer offline/error empty states; wire decline endpoint helper; presence pulse in list.
2. Presence: ensure heartbeat starts on Owyx sign-in; playing when instance launches; Social settings no longer claim “not in this release”.
3. Social settings: add site API `GET/PATCH /api/friends/settings` (`allowFriendRequests`); real toggle; if missing session → sign-in CTA.
4. Account: WelcomeScreen + account chooser — three clear paths; Errors → owyx.site / Discord, never support.modrinth.com in launcher UX.
5. Share: audit SharedInstanceInviteHandler — accept/decline/install copy; no Modrinth-only dead ends for Owyx users.
6. Contract bump notes if settings endpoint added.

## B — Skins in-world
- Prefer CustomSkinLoader / SkinRestorer-compatible URL from `api.owyx.site` cosmetics.
- Document + ship launcher helper: inject recommended skin mod + `CustomSkinLoader` config pointing at Owyx skin URL pattern when installing recommended packs / Owyx Admin shortcut.
- MVP: write skin URL into instance extras + docs for Fabric CustomSkinLoader JSON.

## C — UX debt
- Site: download page, NewsSection → locales; LegalDoc last-modified via locale; admin CatalogAdmin bilingual keys where feasible.
- Launcher: ErrorModal / MinecraftRequired / MinecraftAuthError → Owyx support URLs.

## D — Invented (ship 2–3)
1. Copy friend playing instance / “Join friend’s server” when presence has instanceName matching catalog.
2. Owyx Admin: one-click “open site admin / catalog” + client-key health check strip.
3. Skin preview sync: show site cosmetics skin in account card when signed in.

## E — Polish
- Presence online pulse + reduced-motion; Social empty skeletons; muteable UI click (CC0) in settings; focus rings on Social controls.

## Out
- No prod deploy, no MS OAuth, no marketplace redesign, no merge.
