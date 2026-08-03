import {defineMessage, defineMessages, type MessageDescriptor} from "@modrinth/ui"
import {computed, reactive, type Ref, watch} from "vue"

export type LinkField =
  | "source"
  | "issues"
  | "wiki"
  | "discord"
  | "site"
  | "store"
  | "patreon"
  | "bmac"
  | "paypal"
  | "github"
  | "ko-fi"
  | "other"
  | "license"

const donationFields = new Set<LinkField>(["patreon", "bmac", "paypal", "github", "ko-fi", "other"])

interface LinkFieldInfo {
  name: LinkField
  isDonation: boolean
}

function fieldInfo(name: LinkField): LinkFieldInfo {
  return {name, isDonation: donationFields.has(name)}
}

export type LinkCheckResult =
  | { severity: "valid" }
  | { severity: "warn"; message: MessageDescriptor; values?: Record<string, unknown> }
  | { severity: "error"; message: MessageDescriptor; values?: Record<string, unknown> }

const valid: LinkCheckResult = {severity: "valid"}

function warn(message: MessageDescriptor, values?: Record<string, unknown>): LinkCheckResult {
  return {severity: "warn", message, values}
}

function error(message: MessageDescriptor, values?: Record<string, unknown>): LinkCheckResult {
  return {severity: "error", message, values}
}

type FieldMatcher = LinkField | LinkField[] | ((field: LinkFieldInfo) => boolean)

function matchesField(matcher: FieldMatcher, field: LinkFieldInfo): boolean {
  if (typeof matcher === "function") return matcher(field)
  if (Array.isArray(matcher)) return matcher.includes(field.name)
  return matcher === field.name
}

type LinkCheckVerify = (match: RegExpMatchArray) => Promise<LinkCheckResult>
type LinkCheckMatcher = RegExp | ((remaining: string) => number | null)

interface LinkCheckNode {
  when: LinkCheckMatcher
  label?: string
  unrecognizedSeverity?: "error" | "warn"
  unrecognizedMessage?: MessageDescriptor
  forMatchers?: FieldMatcher[]
  verify?: LinkCheckVerify
  childNodes?: LinkCheckNode[]
  isTransparent?: boolean
}

interface LinkCheckBuilder {
  when: LinkCheckMatcher
  label?: string

  for(fields: FieldMatcher): LinkCheckBuilder

  verify(fn: LinkCheckVerify): LinkCheckBuilder

  children(...shapes: LinkCheckChildShape[]): LinkCheckBuilder

  severity(value: "error" | "warn"): LinkCheckBuilder

  message(descriptor: MessageDescriptor): LinkCheckBuilder

  transparent(): LinkCheckBuilder
}

type LinkCheckChildShape = LinkCheckNode | LinkCheckBuilder | RegExp | string | ((remaining: string) => number | null)

function anchored(source: string): RegExp {
  return new RegExp(`^${source}`, "i")
}

function buildNode(when: LinkCheckMatcher, label?: string): LinkCheckBuilder {
  const childNodes: LinkCheckNode[] = []
  const forMatchers: FieldMatcher[] = []
  const node: Record<string, unknown> = {when, label, childNodes, forMatchers}

  node.for = (fields: FieldMatcher) => {
    forMatchers.push(fields)
    return node
  }
  node.verify = (fn: LinkCheckVerify) => {
    node.verify = fn
    return node
  }
  node.severity = (value: "error" | "warn") => {
    node.unrecognizedSeverity = value
    return node
  }
  node.message = (descriptor: MessageDescriptor) => {
    node.unrecognizedMessage = descriptor
    return node
  }
  node.transparent = () => {
    node.isTransparent = true
    return node
  }
  node.children = (...shapes: LinkCheckChildShape[]) => {
    const parentLabel = node.label as string | undefined
    const parentForMatchers = node.forMatchers as FieldMatcher[] | undefined
    for (const shape of shapes) {
      const child = normalizeChild(shape)
      const label = child.label ? [parentLabel, child.label].filter(Boolean).join(" ") : parentLabel
      const inheritedFor = [...(parentForMatchers ?? []), ...(child.forMatchers ?? [])]
      childNodes.push({...child, label, forMatchers: inheritedFor})
    }
    return node
  }

  return node as unknown as LinkCheckBuilder
}

function check(when: RegExp | string | ((remaining: string) => number | null), label?: string): LinkCheckBuilder {
  const matcher = typeof when === "function" ? when : typeof when === "string" ? new RegExp(when) : when
  return buildNode(matcher, label)
}

function normalizeChild(shape: LinkCheckChildShape): LinkCheckNode {
  if (shape instanceof RegExp || typeof shape === "function") return {when: shape}
  if (typeof shape === "string") return {when: new RegExp(shape)}
  return shape as unknown as LinkCheckNode
}

function named(label: string, shapes: LinkCheckChildShape[]): LinkCheckNode[] {
  return shapes.map((shape) => ({...normalizeChild(shape), label}))
}

interface MatchResult {
  node: LinkCheckNode
  match: RegExpMatchArray
  expectedChild?: LinkCheckNode
}

function matchNode(node: LinkCheckNode, remaining: string, field: LinkFieldInfo): MatchResult | null {
  let match: RegExpMatchArray | null
  if (node.when instanceof RegExp) {
    match = node.when.exec(remaining)
  } else {
    const consumed = node.when(remaining)
    match =
      consumed === null
        ? null
        : (Object.assign([remaining.slice(0, consumed)], {input: remaining, index: 0}) as RegExpMatchArray)
  }

  if (!match) {
    if (!node.unrecognizedMessage) return null
    return {
      node: {
        when: node.when,
        label: node.label,
        unrecognizedMessage: node.unrecognizedMessage,
        unrecognizedSeverity: node.unrecognizedSeverity,
      },
      match: Object.assign([remaining], {input: remaining, index: 0}) as RegExpMatchArray,
    }
  }

  if (node.childNodes?.length) {
    const rest = remaining.slice(match[0].length)
    let expectedChild: LinkCheckNode | undefined
    for (const child of node.childNodes) {
      const found = matchNode(child, rest, field)
      if (found) return found
      if (!expectedChild && child.forMatchers?.some((matcher) => matchesField(matcher, field))) expectedChild = child
    }
    if (node.isTransparent) return null
    return {node, match, expectedChild}
  }

  return {node, match}
}

const coreMessages = defineMessages({
  wrongField: {
    id: "nags.link.wrong-field",
    defaultMessage: "{label} links aren't valid for this field.",
  },
  neverValid: {
    id: "nags.link.never-valid",
    defaultMessage: "{label} links aren't allowed here.",
  },
  expectedType: {
    id: "nags.link.expected-type",
    defaultMessage: "This doesn't look like a {label} link.",
  },
})

//TODO: we should probably just let you not provide https but backend currently requires it
const invalidUrlMessage = defineMessage({
  id: "nags.link.invalid-url",
  defaultMessage: "Links must be a valid https:// URL, not a raw IP address or localhost.",
})

function validUrlPrefix(remaining: string): number | null {
  let url: URL
  try {
    url = new URL(remaining)
    const hostname = url.hostname

    // https pls
    if (url.protocol !== "https:") return null;

    // ensure domain is present
    if (!hostname.includes(".")) return null;

    // reserved TLDs
    if (/(^|\.)(local|localhost|test|example|invalid|onion|arpa|home)$/i.test(hostname)) return null;

    // example.com/net/org is reserved (and probably quite likely to be set by AI)
    if (/^example\.(com|net|org)$/i.test(hostname)) return null

    // No IP addresses
    const strippedHost = hostname.replace(/^\[|]$/g, "")
    if (/^\d{1,3}(?:\.\d{1,3}){3}$/.test(strippedHost) || strippedHost.includes(":")) return null;

    return remaining.indexOf(hostname)
  } catch {
    return null
  }
}

const checks = check(validUrlPrefix).message(invalidUrlMessage).transparent()

const rootNode = checks as unknown as LinkCheckNode

const cache = reactive(new Map<string, "pending" | LinkCheckResult>())

function cacheKey(field: LinkField, url: string): string {
  return `${field}:${url}`
}

function checkLink(field: LinkField, url: string | null | undefined) {
  if (!url) return
  const key = cacheKey(field, url)
  if (cache.has(key)) return

  const normalizedUrl = url.replace(/^(https:\/\/)www\./i, "$1")
  const info = fieldInfo(field)

  const found = matchNode(rootNode, normalizedUrl, info)
  if (!found) return
  const {node: matched, match, expectedChild} = found

  const isLeaf = !matched.childNodes?.length
  const applies = isLeaf && matched.forMatchers?.some((matcher) => matchesField(matcher, info))

  if (!applies) {
    const build = matched.unrecognizedSeverity === "warn" ? warn : error

    if (matched.unrecognizedMessage && isLeaf) {
      cache.set(key, build(matched.unrecognizedMessage, {label: matched.label}))
      return
    }

    if (expectedChild) {
      if (matched.unrecognizedMessage) {
        cache.set(key, build(matched.unrecognizedMessage, {label: matched.label}))
        return
      }

      cache.set(key, build(coreMessages.expectedType, {label: expectedChild.label}))
      return
    }

    const validElsewhere = matched.forMatchers && matched.forMatchers.length > 0
    const message = validElsewhere ? coreMessages.wrongField : coreMessages.neverValid
    cache.set(key, build(message, {label: matched.label}))
    return
  }

  if (!matched.verify) {
    cache.set(key, valid)
    return
  }

  cache.set(key, "pending")
  matched.verify(match).then(
    (result) => cache.set(key, result),
    () => cache.delete(key),
  )
}

function getLinkCheckState(field: LinkField, url: string | null | undefined) {
  if (!url) return undefined
  return cache.get(cacheKey(field, url))
}

function useLinkCheck(field: LinkField, url: Ref<string | null | undefined>) {
  let timeout: ReturnType<typeof setTimeout>
  watch(url, (value) => {
    clearTimeout(timeout)
    timeout = setTimeout(() => checkLink(field, value), 500)
  })

  return computed(() => {
    const state = getLinkCheckState(field, url.value)
    return typeof state === "object" ? state : null
  })
}

async function discordInviteVerify(match: RegExpMatchArray): Promise<LinkCheckResult> {
  const res = await fetch(`https://discord.com/api/v10/invites/${match[1]}?with_expiration=true`)

  if (!res.ok)
    return error(
      defineMessage({
        id: "nags.link.discord.invite.invalid",
        defaultMessage: "This Discord invite is invalid or has expired.",
      }),
    )

  const invite = await res.json()

  if (!invite.guild)
    return error(
      defineMessage({
        id: "nags.link.discord.invite.not-guild",
        defaultMessage: "This Discord invite does not lead to a server.",
      }),
    )

  if (invite.expires_at)
    return warn(
      defineMessage({
        id: "nags.link.discord.invite.expires",
        defaultMessage: "This Discord invite is set to expire",
      }),
    )

  //TODO Ideally we could also check if the invite has a max uses and if its temporary but
  // we can't without auth which we can't really do in frontend

  return valid
}

checks.children(
  ...named("Discord", [
    check(/^discord\.gg\/([\w-]+)/i).for("discord").verify(discordInviteVerify),
    check(/^(?:discord\.com|discordapp\.com)/i)
      .message(
        defineMessage({
          id: "nags.link.discord.unrecognized",
          defaultMessage: "This doesn't look like a Discord invite.",
        }),
      )
      .children(
        check(/^\/invite\/([\w-]+)/i).for("discord").verify(discordInviteVerify),
        check(/^\/channels\//i).message(
          defineMessage({
            id: "nags.link.discord.channel",
            defaultMessage: "This is a link to a Discord channel, not a server invite.",
          }),
        ),
        check(/^\/users\//i).message(
          defineMessage({
            id: "nags.link.discord.user",
            defaultMessage: "This is a link to a Discord user, not a server invite.",
          }),
        ),
      ),
  ]),
)

function gitHost(
  name: string,
  domain: string,
  fetchRepo: (path: string) => Promise<Record<string, boolean> | undefined>,
  options: {
    pathPattern?: string
    subPageSeparator?: string
    wikiPath?: string
  } = {},
): LinkCheckBuilder {
  const path = options.pathPattern ?? /[^/]+\/[^/]+/.source
  const sep = options.subPageSeparator ?? ""
  const wikiPath = options.wikiPath ?? "wiki"

  async function checkRepo(
    fetchRepo: (path: string) => Promise<Record<string, boolean> | undefined>,
    path: string,
    evaluate: (facts: Record<string, boolean>) => LinkCheckResult,
  ): Promise<LinkCheckResult> {
    const facts = await fetchRepo(path)
    if (!facts)
      return error(
        defineMessage({
          id: "nags.link.git.not-found",
          defaultMessage: "This repository could not be found (it may be private or deleted).",
        }),
      )

    return evaluate(facts)
  }

  return check(anchored(domain), name)
    .severity("warn")
    .children(
      check(anchored(`/(${path})/?$`), "repo").for("source").verify(async (match) =>
        checkRepo(fetchRepo, match[1], (facts) =>
          facts.empty
            ? error(
              defineMessage({
                id: "nags.link.git.empty",
                defaultMessage: "This repository appears to be empty.",
              }),
            )
            : valid,
        ),
      ),

      check(anchored(`/(${path})${sep}/issues`), "issues").for("issues").verify(async (match) =>
        checkRepo(fetchRepo, match[1], (facts) => {
          if (facts.archived)
            return error(
              defineMessage({
                id: "nags.link.git.archived",
                defaultMessage: "This repository is archived, which disables issues.",
              }),
            )
          if (facts.issues === false)
            return error(
              defineMessage({
                id: "nags.link.git.issues-disabled",
                defaultMessage: "Issues are disabled on this repository.",
              }),
            )
          return valid
        }),
      ),

      check(anchored(`/(${path})${sep}/${wikiPath}`), "wiki").for("wiki").verify(async (match) =>
        checkRepo(fetchRepo, match[1], (facts) =>
          facts.wiki === false
            ? error(
              defineMessage({
                id: "nags.link.git.wiki-disabled",
                defaultMessage: "The wiki is disabled on this repository.",
              }),
            )
            : valid,
        ),
      ),
    )
}


// Repo Platforms, includes most source/issues/wiki + Github Sponsor
checks.children(
  gitHost("GitHub", "github\\.com", async (path) => {
    const res = await fetch(`https://api.github.com/repos/${path}`)
    if (!res.ok) return undefined
    const data = await res.json()
    return {
      empty: data.size === 0,
      archived: data.archived,
      issues: data.has_issues,
      wiki: data.has_wiki,
    }
  })
    // Github sponsor is here
    .children(check(/^\/sponsors\/[^/]+/i, "sponsors").for("github")),

  gitHost("Codeberg", "codeberg\\.org", async (path) => {
    const res = await fetch(`https://codeberg.org/api/v1/repos/${path}`)
    if (!res.ok) return undefined
    const data = await res.json()
    return {
      empty: data.size === 0,
      archived: data.archived,
      issues: data.has_issues,
      wiki: data.has_wiki,
    }
  }),

  gitHost(
    "GitLab",
    "gitlab\\.com",
    async (path) => {
      const res = await fetch(`https://gitlab.com/api/v4/projects/${encodeURIComponent(path)}`)
      if (!res.ok) return undefined
      //TODO unauthed gitlab doesn't give us like any info, so... yeah that sucks I guess
      return {}
    },
    {
      pathPattern: /[^/]+(?:\/[^/]+)+/.source,
      subPageSeparator: "/-",
      wikiPath: "wikis",
    },
  ),

  gitHost("Bitbucket", "bitbucket\\.org", async (path) => {
    const res = await fetch(`https://api.bitbucket.org/2.0/repositories/${path}`)
    if (!res.ok) return undefined
    const data = await res.json()
    return {
      empty: data.size === 0,
      issues: data.has_issues,
      wiki: data.has_wiki,
    }
  }),

  gitHost("Gitee", "gitee\\.com", async (path) => {
    const res = await fetch(`https://gitee.com/api/v5/repos/${path}`)
    if (!res.ok) return undefined
    const data = await res.json()
    return {
      issues: data.has_issues,
      wiki: data.has_wiki,
    }
  }),
)

// Donation
checks.children(
  check(/^patreon\.com/i, "Patreon")
    .children(check(/^\/(?:user\?u=\d+|[\w.-]+)/i).for("patreon")),

  check(/^(?:buymeacoffee\.com|buymeacoff\.ee)/i, "Buy Me a Coffee")
    .children(check(/^\/([\w-]+)/i).for("bmac")),

  check(/^paypal\.[a-z.]{2,}/i, "PayPal").for("paypal").children(
    check(/^\/paypalme\/[\w.-]+/i),
    check(/^\/donate/i),
    check(/^\/cgi-bin\/webscr\?cmd=_donations/i),
  ),
  check(/^paypal\.me/i, "PayPal")
    .children(check(/^\/([\w.-]+)/i).for("paypal")),

  // Github sponsor is with the rest of github.

  check(/^ko-fi\.com/i, "Ko-fi")
    .children(check(/^\/([\w-]+)/i).for("ko-fi")),

  (() => {
    const YOUTUBE_CHANNEL = "(?:@[\\w.-]+|channel/[\\w-]+|c/[\\w-]+|user/[\\w-]+)"

    return check(/^(?:youtube\.com|youtu\.be)/i, "YouTube")
      .message(
        defineMessage({
          id: "nags.link.youtube.unrecognized",
          defaultMessage: "This doesn't look like a YouTube donation link.",
        }),
      )
      .for("other")
      .children(check(anchored(`${YOUTUBE_CHANNEL}/join`)), check(anchored(`${YOUTUBE_CHANNEL}/store`)))
  })(),
)

interface KnownLicenseSite {
  domain: string
  path: string
  label: string
  extract?: (match: RegExpMatchArray) => string
}

const knownLicenseSites: KnownLicenseSite[] = [
  {domain: "spdx\\.org", path: "/licenses/([\\w.-]+)\\.html", label: "SPDX", extract: (m) => m[1]},
  {domain: "opensource\\.org", path: "/licenses?/([\\w.-]+)", label: "OSI", extract: (m) => m[1]},
  {domain: "choosealicense\\.com", path: "/licenses/([\\w.-]+)", label: "choosealicense.com", extract: (m) => m[1]},
  {domain: "(?:www\\.)?gnu\\.org", path: "/licenses/[\\w.-]+", label: "GNU"},
  {domain: "(?:www\\.)?apache\\.org", path: "/licenses/[\\w.-]+", label: "Apache"},
  {domain: "creativecommons\\.org", path: "/(?:licenses/[\\w-]+|publicdomain/zero)/[\\d.]+/?", label: "Creative Commons"},
]

checks.children(
  ...knownLicenseSites.map(({domain, path, label}) =>
    check(anchored(domain), label).children(check(anchored(path)).for("license")),
  ),
)

function identifyLicenseFromUrl(url: string): string | null {
  const normalized = url.replace(/^https:\/\/(?:www\.)?/i, "")
  for (const site of knownLicenseSites) {
    if (!site.extract) continue
    const pattern = new RegExp(`^${site.domain}${site.path}`, "i")
    const match = pattern.exec(normalized)
    if (match) return site.extract(match)
  }
  return null
}

const licenseCheckMessages = defineMessages({
  urlMismatch: {
    id: "nags.link.license.url-mismatch",
    defaultMessage: "This link points to the {detected} license, but your project is set to {selected}.",
  },
  urlRedundant: {
    id: "nags.link.license.url-redundant",
    defaultMessage:
      "You don't need to link to a generic license page for a supported license — consider linking to your repository's own license file instead, or leaving this blank.",
  },
})

function useLicenseUrlCheck(
  url: Ref<string | null | undefined>,
  license: Ref<{ friendly: string; short: string }>,
) {
  const baseCheck = useLinkCheck("license", url)

  return computed<LinkCheckResult | null>(() => {
    const isCustom = license.value.friendly === "Custom"
    const detected = url.value ? identifyLicenseFromUrl(url.value) : null

    if (detected && !isCustom && detected.toLowerCase() !== license.value.short.toLowerCase()) {
      return {
        severity: "warn",
        message: licenseCheckMessages.urlMismatch,
        values: {detected, selected: license.value.short},
      }
    }

    if (baseCheck.value?.severity === "valid" && !isCustom) {
      return {severity: "warn", message: licenseCheckMessages.urlRedundant}
    }

    return baseCheck.value
  })
}

// Kinda just everything else
checks.children(
  check(/^docs\.google\.com/i, "Google").children(
    check(/^\/forms\//i, "Forms").for("issues"),
    check(/^\/document\//i, "Documents").for("wiki"),
  ),
  check(/^(?:bit\.ly|adf\.ly|tinyurl\.com|short\.io|is\.gd)/i, "Link shortener"),
  check(/^(?:twitter\.com|x\.com)/i, "Twitter"),
  check(/^instagram\.com/i, "Instagram"),
  check(/^facebook\.com/i, "Facebook"),
  check(/^tiktok\.com/i, "TikTok"),
  check(/^(?:telegram\.org|t\.me)/i, "Telegram"),
  check(/^bilibili\.com/i, "Bilibili"),
  check(/^curseforge\.com/i, "CurseForge"),
  check(/^modrinth\.com/i, "Modrinth"),
  check(/^reddit\.com/i, "Reddit"),
  check(/^twitch\.tv/i, "Twitch"),
  check(/^minecraft\.net/i, "Minecraft"),
  check(/^bsky\.app/i, "Bluesky"),
)

export {checkLink, getLinkCheckState, useLinkCheck, useLicenseUrlCheck}
