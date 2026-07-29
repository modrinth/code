> [!success]
> API v2 is Modrinth's current, official API. It is the recommended API for integrations and applications and will receive long-term support.

## Authentication
This API has two options for authentication: personal access tokens and [OAuth2](https://en.wikipedia.org/wiki/OAuth).
All tokens are tied to a Modrinth user and use the `Authorization` header of the request.

Example:
```
Authorization: mrp_RNtLRSPmGj2pd1v1ubi52nX7TJJM9sznrmwhAuj511oe4t1jAqAQ3D6Wc8Ic
```

> [!tip] You do not need a token for most requests.
>
> Generally speaking, only the following types of requests require a token:
> - those which create data (such as version creation)
> - those which modify data (such as editing a project)
> - those which access private data (such as draft projects, notifications, emails, and payout data)

Each request requiring authentication has a certain scope. For example, to view the email of the user being requested, the token must have the `USER_READ_EMAIL` scope.
You can find the list of available scopes [on GitHub](https://github.com/modrinth/code/blob/main/apps/labrinth/src/models/v3/pats.rs#L9). Making a request with an invalid scope will return a 401 error.

Please note that certain scopes and requests cannot be completed with a personal access token or using OAuth.
For example, deleting a user account can only be done through Modrinth's frontend.

### Personal access tokens
Personal access tokens (PATs) can be generated from [the user settings](https://modrinth.com/settings/account).

### GitHub tokens
For backwards compatibility purposes, some types of GitHub tokens also work for authenticating a user with Modrinth's API, granting all scopes.
**We urge any application still using GitHub tokens to start using personal access tokens for security and reliability purposes.**

## Cross-Origin Resource Sharing
This API features Cross-Origin Resource Sharing (CORS) implemented in compliance with the [W3C spec](https://www.w3.org/TR/cors/).
This allows for cross-domain communication from the browser.
All responses have a wildcard same-origin which makes them completely public and accessible to everyone, including any code on any site.

## Identifiers
The majority of items you can interact with in the API have a unique eight-digit base62 ID.
Projects, versions, users, threads, teams, and reports all use this same way of identifying themselves.
Version files use the sha1 or sha512 file hashes as identifiers.

Each project and user has a friendlier way of identifying them; slugs and usernames, respectively.
While unique IDs are constant, slugs and usernames can change at any moment.
If you want to store something in the long term, it is recommended to use the unique ID.

## Ratelimits
The API has a ratelimit defined per IP. Limits and remaining amounts are given in the response headers.
- `X-Ratelimit-Limit`: the maximum number of requests that can be made in a minute
- `X-Ratelimit-Remaining`: the number of requests remaining in the current ratelimit window
- `X-Ratelimit-Reset`: the time in seconds until the ratelimit window resets

**Ratelimits are the same no matter whether you use a token or not.**

The ratelimit is currently 300 requests per minute per IP. If your application is hitting rate limits, try caching responses, using batch requests, or spreading out your requests over time.

Higher rate limits are granted only in very rare cases. If you believe your use case requires an exception, please [contact us](mailto:support@modrinth.com).

## User Agents
To access the Modrinth API, you **must** use provide a uniquely-identifying `User-Agent` header.
Providing a user agent that only identifies your HTTP client library (such as "okhttp/4.9.3") increases the likelihood that we will block your traffic.
It is recommended, but not required, to include contact information in your user agent.
This allows us to contact you if we would like a change in your application's behavior without having to block your traffic.
- Bad: `User-Agent: okhttp/4.9.3`
- Good: `User-Agent: project_name`
- Better: `User-Agent: github_username/project_name/1.56.0`
- Best: `User-Agent: github_username/project_name/1.56.0 (launcher.com)` or `User-Agent: github_username/project_name/1.56.0 (contact@launcher.com)`

## Versioning
Modrinth follows a simple pattern for its API versioning.
In the event of a breaking API change, the API version in the URL path is bumped, and migration steps will be published below.

### Migrations
Inside the following spoiler, you will be able to find all changes between versions of the Modrinth API, accompanied by tips and a guide to migrate applications to newer versions.

<details><summary>API v1 to API v2</summary>

These bullet points cover most changes in the v2 API, but please note that fields containing `mod` in most contexts have been shifted to `project`. For example, in the search route, the field `mod_id` was renamed to `project_id`.

- The search route has been moved from `/api/v1/mod` to `/v2/search`
- New project fields: `project_type` (may be `mod` or `modpack`), `moderation_message` (which has a `message` and `body`), `gallery`
- New search facet: `project_type`
- Alphabetical sort removed (it didn't work and is not possible due to limits in MeiliSearch)
- New search fields: `project_type`, `gallery`
	- The gallery field is an array of URLs to images that are part of the project's gallery
- The gallery is a new feature which allows the user to upload images showcasing their mod to the CDN which will be displayed on their mod page
- Internal change: Any project file uploaded to Modrinth is now validated to make sure it's a valid Minecraft mod, Modpack, etc.
	- For example, a Forge 1.17 mod with a JAR not containing a mods.toml will not be allowed to be uploaded to Modrinth
- In project creation, projects may not upload a mod with no versions to review, however they can be saved as a draft
	- Similarly, for version creation, a version may not be uploaded without any files
- Donation URLs have been enabled
- New project status: `archived`. Projects with this status do not appear in search
- Tags (such as categories, loaders) now have icons (SVGs) and specific project types attached
- Dependencies have been wiped and replaced with a new system
- Notifications now have a `type` field, such as `project_update`

Along with this, project subroutes (such as `/v2/project/{id}/version`) now allow the slug to be used as the ID. This is also the case with user routes.

<br />

The above snippet about User Agents was adapted from https://crates.io/policies, copyright (c) 2014 The Rust Project Developers under MIT license.
