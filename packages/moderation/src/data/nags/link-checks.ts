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

type LinkCheckVerify = (match: RegExpMatchArray) => Promise<LinkCheckResult>
type LinkCheckMatcher = RegExp | ((remaining: string) => boolean)

interface LinkCheckNode {
  when: LinkCheckMatcher
  label?: string
  unrecognizedSeverity?: "error" | "warn"
  unrecognizedMessage?: MessageDescriptor
  otherwiseMessage?: MessageDescriptor
  for?: Partial<Record<LinkField, LinkCheckVerify | true>>
  childNodes?: LinkCheckNode[]
  isTransparent?: boolean
}

interface LinkCheckBuilder {
  when: LinkCheckMatcher
  label?: string

  for(fields: Partial<Record<LinkField, LinkCheckVerify | true>>): LinkCheckBuilder

  children(...shapes: LinkCheckChildShape[]): LinkCheckBuilder

  severity(value: "error" | "warn"): LinkCheckBuilder

  message(descriptor: MessageDescriptor): LinkCheckBuilder

  transparent(): LinkCheckBuilder

  otherwise(descriptor: MessageDescriptor): LinkCheckBuilder
}

type LinkCheckChildShape = LinkCheckNode | LinkCheckBuilder | RegExp | string | ((remaining: string) => boolean)

function compileChild(source: string): RegExp {
  return new RegExp(`^${source}`, "i")
}

function buildNode(when: LinkCheckMatcher, label?: string): LinkCheckBuilder {
  const childNodes: LinkCheckNode[] = []
  const node: Record<string, unknown> = {when, label, childNodes}

  node.for = (fields: Partial<Record<LinkField, LinkCheckVerify | true>>) => {
    node.for = fields
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
  node.otherwise = (descriptor: MessageDescriptor) => {
    node.otherwiseMessage = descriptor
    return node
  }
  node.children = (...shapes: LinkCheckChildShape[]) => {
    const parentLabel = node.label as string | undefined
    for (const shape of shapes) {
      const child = normalizeChild(shape)
      const label = child.label ? [parentLabel, child.label].filter(Boolean).join(" ") : parentLabel
      childNodes.push({...child, label})
    }
    return node
  }

  return node as unknown as LinkCheckBuilder
}

function check(when: RegExp | string | ((remaining: string) => boolean), label?: string): LinkCheckBuilder {
  return buildNode(typeof when === "string" ? compileChild(when) : when, label)
}

function normalizeChild(shape: LinkCheckChildShape): LinkCheckNode {
  if (shape instanceof RegExp || typeof shape === "function") return {when: shape}
  if (typeof shape === "string") return {when: compileChild(shape)}
  return shape as unknown as LinkCheckNode
}

function matchNode(node: LinkCheckNode, remaining: string): { node: LinkCheckNode; match: RegExpMatchArray } | null {
  const match =
    node.when instanceof RegExp
      ? node.when.exec(remaining)
      : node.when(remaining)
        ? (Object.assign([remaining], {input: remaining, index: 0}) as RegExpMatchArray)
        : null

  if (!match) {
    if (!node.otherwiseMessage) return null
    return {
      node: {when: node.when, label: node.label, unrecognizedMessage: node.otherwiseMessage},
      match: Object.assign([remaining], {input: remaining, index: 0}) as RegExpMatchArray,
    }
  }

  if (node.childNodes?.length) {
    const rest = remaining.slice(match[0].length)
    for (const child of node.childNodes) {
      const found = matchNode(child, rest)
      if (found) return found
    }
    if (node.isTransparent) return null
  }

  return {node, match}
}

const engineMessages = defineMessages({
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
const httpsRequiredMessage = defineMessage({
  id: "nags.link.https-required",
  defaultMessage: "Links must start with https://",
})

const checks = check(/^https:\/\/(?:www\.)?/i, "Links").otherwise(httpsRequiredMessage).transparent()

const rootNode = checks as unknown as LinkCheckNode

const cache = reactive(new Map<string, "pending" | LinkCheckResult>())

function cacheKey(field: LinkField, url: string): string {
  return `${field}:${url}`
}

function checkLink(field: LinkField, url: string | null | undefined) {
  if (!url) return
  const key = cacheKey(field, url)
  if (cache.has(key)) return

  const found = matchNode(rootNode, url)
  if (!found) return
  const {node: matched, match} = found

  const entry = matched.for?.[field]

  if (entry === undefined) {
    const build = matched.unrecognizedSeverity === "warn" ? warn : error

    if (matched.unrecognizedMessage) {
      cache.set(key, build(matched.unrecognizedMessage, {label: matched.label}))
      return
    }

    const expectedChild = matched.childNodes?.find((child) => child.for && field in child.for)
    if (expectedChild) {
      cache.set(key, build(engineMessages.expectedType, {label: expectedChild.label}))
      return
    }

    const validElsewhere = matched.for && Object.keys(matched.for).length > 0
    const message = validElsewhere ? engineMessages.wrongField : engineMessages.neverValid
    cache.set(key, build(message, {label: matched.label}))
    return
  }

  if (entry === true) {
    cache.set(key, valid)
    return
  }

  cache.set(key, "pending")
  entry(match).then(
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

const discordMessages = defineMessages({
  inviteInvalid: {
    id: "nags.link.discord.invite.invalid",
    defaultMessage: "This Discord invite is invalid or has expired.",
  },
  inviteNotGuild: {
    id: "nags.link.discord.invite.not-guild",
    defaultMessage: "This Discord invite does not lead to a server.",
  },
  inviteExpires: {
    id: "nags.link.discord.invite.expires",
    defaultMessage: "This Discord invite is set to expire",
  },
  channel: {
    id: "nags.link.discord.channel",
    defaultMessage: "This is a link to a Discord channel, not a server invite.",
  },
  user: {
    id: "nags.link.discord.user",
    defaultMessage: "This is a link to a Discord user, not a server invite.",
  },
})

async function discordInviteVerify(match: RegExpMatchArray): Promise<LinkCheckResult> {
  const res = await fetch(`https://discord.com/api/v10/invites/${match[1]}?with_expiration=true`)

  if (!res.ok) return error(discordMessages.inviteInvalid)

  const invite = await res.json()

  if (!invite.guild) return error(discordMessages.inviteNotGuild)

  if (invite.expires_at) return warn(discordMessages.inviteExpires)

  //TODO Ideally we could also check if the invite has a max uses and if its temporary but
  // we can't without auth which we can't really do in frontend

  return valid
}

checks
  .children(check(`discord\\.gg/(${/[\w-]+/.source})`, "Discord").for({discord: discordInviteVerify}))
  .children(
    check(/discord\.com|discordapp\.com/.source, "Discord")
      .message(
        defineMessage({
          id: "nags.link.discord.unrecognized",
          defaultMessage: "This doesn't look like a Discord invite.",
        }),
      )
      .children(
        check(`/invite/(${/[\w-]+/.source})`).for({discord: discordInviteVerify}),
        check("/channels/").message(discordMessages.channel),
        check("/users/").message(discordMessages.user),
      ),
  )

const gitRepoMessages = defineMessages({
  notFound: {
    id: "nags.link.git.not-found",
    defaultMessage: "This repository could not be found (it may be private or deleted).",
  },
  empty: {
    id: "nags.link.git.empty",
    defaultMessage: "This repository appears to be empty.",
  },
  archived: {
    id: "nags.link.git.archived",
    defaultMessage: "This repository is archived, which disables issues.",
  },
  issues: {
    id: "nags.link.git.issues-disabled",
    defaultMessage: "Issues are disabled on this repository.",
  },
  wiki: {
    id: "nags.link.git.wiki-disabled",
    defaultMessage: "The wiki is disabled on this repository.",
  },
})

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

  async function validate(
    fetchRepo: (path: string) => Promise<Record<string, boolean> | undefined>,
    path: string,
    relevantFacts: Partial<Record<keyof typeof gitRepoMessages, boolean>>,
  ): Promise<LinkCheckResult> {
    const facts = await fetchRepo(path)
    if (!facts) return error(gitRepoMessages.notFound)

    for (const key of Object.keys(relevantFacts) as (keyof typeof gitRepoMessages)[]) {
      if (facts[key] === relevantFacts[key]) return error(gitRepoMessages[key])
    }

    return valid
  }

  return check(domain, name)
    .severity("warn")
    .children(
      check(`/(${path})/?$`, "repo").for({
        source: async (match) => validate(fetchRepo, match[1], {empty: true}),
      }),

      check(`/(${path})${sep}/issues`, "issues").for({
        issues: async (match) => validate(fetchRepo, match[1], {archived: true, issues: false}),
      }),

      check(`/(${path})${sep}/${wikiPath}`, "wiki").for({
        wiki: async (match) => validate(fetchRepo, match[1], {wiki: false}),
      }),
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
    .children(check("/sponsors/[^/]+", "sponsors").for({github: true})),

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
  check("patreon\\.com", "Patreon")
    .children(check("/(?:user\\?u=\\d+|[\\w.-]+)").for({patreon: true})),

  check("(?:buymeacoffee\\.com|buymeacoff\\.ee)", "Buy Me a Coffee")
    .children(check("/([\\w-]+)").for({bmac: true})),

  check("paypal\\.[a-z.]{2,}", "PayPal")
    .children(
      check("/paypalme/[\\w.-]+").for({paypal: true}),
      check("/donate").for({paypal: true}),
      check("/cgi-bin/webscr\\?cmd=_donations").for({paypal: true}),
    ),
  check("paypal\\.me", "PayPal")
    .children(check("/([\\w.-]+)").for({paypal: true})),

  // Github sponsor is with the rest of github.

  check("ko-fi\\.com", "Ko-fi")
    .children(check("/([\\w-]+)").for({"ko-fi": true})),

  (() => {
    const YOUTUBE_CHANNEL = "(?:@[\\w.-]+|channel/[\\w-]+|c/[\\w-]+|user/[\\w-]+)"

    return check("(?:youtube\\.com|youtu\\.be)", "YouTube")
      .message(
        defineMessage({
          id: "nags.link.youtube.unrecognized",
          defaultMessage: "This doesn't look like a YouTube donation link.",
        }),
      )
      .children(
        check(`${YOUTUBE_CHANNEL}/join`).for({other: true}),
        check(`${YOUTUBE_CHANNEL}/store`).for({other: true}),
      )
  })(),
)

// Kinda just everything else
checks.children(
  check("docs\\.google\\.com", "Google").children(
    check("/forms/", "Forms").for({issues: true}),
    check("/document/", "Documents").for({wiki: true}),
  ),
  check("(?:bit\\.ly|adf\\.ly|tinyurl\\.com|short\\.io|is\\.gd)", "Link shortener"),
  check("(?:twitter\\.com|x\\.com)", "Twitter"),
  check("instagram\\.com", "Instagram"),
  check("facebook\\.com", "Facebook"),
  check("tiktok\\.com", "TikTok"),
  check("(?:telegram\\.org|t\\.me)", "Telegram"),
  check("bilibili\\.com", "Bilibili"),
  check("curseforge\\.com", "CurseForge"),
  check("modrinth\\.com", "Modrinth"),
  check("reddit\\.com", "Reddit"),
  check("twitch\\.tv", "Twitch"),
  check("minecraft\\.net", "Minecraft"),
  check("bsky\\.app", "Bluesky"),
)

export {checkLink, getLinkCheckState, useLinkCheck}
