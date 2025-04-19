---
title: The hitchhiker's guide to OAuth
description: Guide for using Modrinth OAuth to interact with the API on users' behalf.
---

Modrinth allows developers to create applications which, once authorized by a Modrinth user, let the developer interact with the API on their behalf. The flow used to get an API token is based on the OAuth 2 protocol. It is recommended that most people use an existing OAuth library to handle the authentication. If you want to implement it from scratch, you will need to look into [RFC 6749]. If the only user of the application is yourself, a personal access token (PAT) may be a better fit.

If you're familiar with OAuth 2, these are the URLs you will need:

| Name               | URL                                              |
|--------------------|--------------------------------------------------|
| Authorization page | `https://modrinth.com/auth/authorize`            |
| Token exchange     | `https://api.modrinth.com/_internal/oauth/token` |

The flow will generally look like this:

1. User is redirected to Modrinth to authorize your application
2. User is redirected back to your site after authorizing, with an authorization code
3. Your backend exchanges this code for an access token

## Register your application

To start off, you need to [register an application] in Modrinth's systems. The settings chosen here can always be changed later. You need to select what permissions you need, called scopes. For security reasons you will want to select only the scopes you need. See the [principle of least privilege].

In addition to name and scopes, you will also need to add one or more redirect URIs. These are the URIs that the user can be redirected to after they authorize your application.

After you've registered your application, it is important that you take note of the client secret somewhere safe. If the client secret is to ever leak, it is important that you regenerate it to ensure the security of your authorized users. If your client secret or access tokens are found exposed in the wild, your application may be disabled without prior notice.

## Getting authorization

Once the user is ready to authorize your application, you need to construct a URL to redirect them to. The authorization URL for Modrinth is `https://api.modrinth.com/_internal/oauth/token`. Supply the following query parameters:

| Query parameter | Description                                                                               |
|-----------------|-------------------------------------------------------------------------------------------|
| `response_type` | In Modrinth this always needs to be `code`, since only code grants are supported          |
| `client_id`     | The application identifier found in the settings                                          |
| `scope`         | The permissions you need access to                                                        |
| `state`         | A mechanism to prevent certain attacks. Explained further below. Recommended but optional |
| `redirect_uri`  | The URI the user is redirect to after finishing authorization                             |

You might have noticed the `state` parameter. [CSRF] (Cross-site request forgery), and [clickjacking] are security vulnerabilities that you're recommended to protect against. In OAuth2 this is usually done with the `state` parameter. When the user initiates a request to start authorization, you include a `state` which is unique to this request. This can, for example, be saved in localStorge or a cookie. When the redirect URI is called, you verify that the `state` parameter is the same. Using `state` is optional, but recommended.

The scope identifiers are currently best found in the backend source code located at [`apps/labrinth/src/models/v3/pats.rs`]. The scope parameter is an array of scope identifiers, seperated by a plus sign (`+`).

The redirect URI is the endpoint on your server that will receive the code which can eventually be used to act on the user's behalf. For security reasons the redirect URI used has to be allowlisted in your application settings. The redirect will contain the following query parameters:

| Query parameter | Description                                        |
|-----------------|----------------------------------------------------|
| `code`          | The code that can be exchanged for an access token |
| `client_id`     | Your client id                                     |
| `redirect_uri`  | The redirect URI which was used                    |
| `grant_type`    | Always `authorization_code` in Modrinth            |

## Exchanging tokens

If you've followed the previous section on getting authorization, you should now have an authorization code. Before you can access the API, you need to exchange this code for an access token. This is done by sending a POST request to the exchange token endpoint, `https://api.modrinth.com/_internal/oauth/token`. This request has to be of type urlencoded form. Make sure the `Content-Type` header is set to `application/x-www-form-urlencoded`. To authenticate this request you need to place your client secret in the `Authorization` header.

In the body use these fields:

| Field          | Description                                                  |
|----------------|--------------------------------------------------------------|
| `code`         | The authorization code                                       |
| `client_id`    | Your client id, the same as in the authorization request     |
| `redirect_uri` | The redirect URI which was redirected to after authorization |
| `grant_type`   | Always `authorization_code` in Modrinth                      |

If the request succeeds, you should receive a JSON payload with these fields:

| Field          | Description                                          |
|----------------|------------------------------------------------------|
| `access_token` | The access token you can use to access the API       |
| `token_type`   | Currently only `Bearer`                              |
| `expires_in`   | The amount of seconds until the access token expires |

To use this access token, you attach it to API requests in the `Authorization` header. To get basic information about the authorizer, you can use the [`/user` endpoint], which automatically gets the user from the header.

If you have any questions, you're welcome to ask in #api-development in the [Discord guild], or create a ticket on the [support portal].

[RFC 6749]: https://datatracker.ietf.org/doc/html/rfc6749

[register an application]: https://modrinth.com/settings/applications

[principle of least privilege]: https://en.wikipedia.org/wiki/Principle_of_least_privilege

[`apps/labrinth/src/models/v3/pats.rs`]: https://github.com/modrinth/code/blob/main/apps/labrinth/src/models/v3/pats.rs

[CSRF]: https://en.wikipedia.org/wiki/Cross-site_request_forgery

[Clickjacking]: https://en.wikipedia.org/wiki/Clickjacking

[`/user` endpoint]: https://docs.modrinth.com/api/operations/getuserfromauth/

[Discord guild]: https://discord.modrinth.com

[support portal]: https://support.modrinth.com/en/
