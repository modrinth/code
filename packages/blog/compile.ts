import { promises as fs } from 'fs'
import * as path from 'path'
import fg from 'fast-glob'
import matter from 'gray-matter'
import { md } from '@modrinth/utils'
import { minify } from 'html-minifier-terser'
import { copyDir, toVarName } from './utils'
import RSS from 'rss'
import { parseStringPromise } from 'xml2js'

import {
  ARTICLES_GLOB,
  COMPILED_DIR,
  ROOT_FILE,
  PUBLIC_SRC,
  PUBLIC_LOCATIONS,
  RSS_PATH,
  JSON_PATH,
  SITE_URL,
} from './blog.config'

async function ensureCompiledDir() {
  await fs.mkdir(COMPILED_DIR, { recursive: true })
}

async function hasThumbnail(slug: string): Promise<boolean> {
  const thumbnailPath = path.join(PUBLIC_SRC, slug, 'thumbnail.webp')
  try {
    await fs.access(thumbnailPath)
    return true
  } catch {
    return false
  }
}

function getArticleLink(slug: string): string {
  return `${SITE_URL}/news/article/${slug}`
}

function getThumbnailUrl(slug: string, hasThumb: boolean): string {
  if (hasThumb) {
    return `${SITE_URL}/news/article/${slug}/thumbnail.webp`
  } else {
    return `${SITE_URL}/news/default.jpg`
  }
}

async function compileArticles() {
  await ensureCompiledDir()

  const files = await fg([ARTICLES_GLOB])
  console.log(`🔎  Found ${files.length} markdown articles!`)
  const articleExports: string[] = []
  const articlesArray: string[] = []
  const articlesForRss = []
  const articlesForJson = []

  for (const file of files) {
    const src = await fs.readFile(file, 'utf8')
    const { content, data } = matter(src)

    const { title, summary, date, slug: frontSlug, ...rest } = data
    if (!title || !summary || !date) {
      console.error(`❌  Missing required frontmatter in ${file}. Required: title, summary, date`)
      process.exit(1)
    }

    const html = md().render(content)
    const minifiedHtml = await minify(html, {
      collapseWhitespace: true,
      removeComments: true,
    })

    const slug = frontSlug || path.basename(file, '.md')
    const varName = toVarName(slug)
    const exportFile = path.join(COMPILED_DIR, `${varName}.ts`)
    const contentFile = path.join(COMPILED_DIR, `${varName}.content.ts`)
    const thumbnailPresent = await hasThumbnail(slug)

    const contentTs = `
// AUTO-GENERATED FILE - DO NOT EDIT
export const html = ${JSON.stringify(minifiedHtml)};
`.trimStart()
    await fs.writeFile(contentFile, contentTs, 'utf8')

    const ts = `
// AUTO-GENERATED FILE - DO NOT EDIT
export const article = {
  html: () => import("./${varName}.content").then(m => m.html),
  title: ${JSON.stringify(title)},
  summary: ${JSON.stringify(summary)},
  date: ${JSON.stringify(date)},
  slug: ${JSON.stringify(slug)},
  thumbnail: ${thumbnailPresent},
  ${Object.keys(rest)
    .map((k) => `${k}: ${JSON.stringify(rest[k])},`)
    .join('\n  ')}
};
`.trimStart()

    await fs.writeFile(exportFile, ts, 'utf8')
    articleExports.push(`import { article as ${varName} } from "./${varName}";`)
    articlesArray.push(varName)

    articlesForRss.push({
      title,
      summary,
      date,
      slug,
      html: minifiedHtml,
    } as never)

    articlesForJson.push({
      title,
      summary,
      thumbnail: getThumbnailUrl(slug, thumbnailPresent),
      date: new Date(date).toISOString(),
      link: getArticleLink(slug),
    } as never)
  }

  console.log(`📂  Compiled ${files.length} articles.`)

  const rootExport = `
// AUTO-GENERATED FILE - DO NOT EDIT
${articleExports.join('\n')}

export const articles = [
  ${articlesArray.join(',\n  ')}
];
`.trimStart()

  await fs.writeFile(ROOT_FILE, rootExport, 'utf8')
  console.log(`🌟  Done! Wrote root articles export.`)

  await generateRssFeed(articlesForRss)
  await generateJsonFile(articlesForJson)
}

async function generateRssFeed(articles): Promise<void> {
  const sorted = [...articles].sort(
    (a, b) => new Date(b.date).getTime() - new Date(a.date).getTime(),
  )

  let currentRssArticles: { title: string; html: string }[] = []
  try {
    const xml = await fs.readFile(RSS_PATH, 'utf8')
    const parsed = await parseStringPromise(xml)
    const items = parsed.rss?.channel?.[0]?.item || []
    currentRssArticles = items.map((item) => ({
      title: (item.title?.[0] ?? '').trim(),
      html: (item['content:encoded']?.[0] ?? '').replace(/^<!\[CDATA\[|\]\]>$/g, '').trim(),
    }))
  } catch {
    currentRssArticles = []
  }

  const newArr = sorted.map((a) => ({
    title: (a.title ?? '').trim(),
    html: (a.html ?? '').trim(),
  }))

  let isEqual = currentRssArticles.length === newArr.length
  if (isEqual) {
    for (let i = 0; i < newArr.length; ++i) {
      if (
        currentRssArticles[i].title !== newArr[i].title ||
        currentRssArticles[i].html !== newArr[i].html
      ) {
        isEqual = false
        break
      }
    }
  }

  if (isEqual) {
    console.log(`⏭️  RSS feed not regenerated (articles unchanged)`)
    return
  }

  const feed = new RSS({
    title: 'Modrinth News',
    description: 'Keep up-to-date on the latest news from Modrinth.',
    feed_url: `${SITE_URL}/news/feed/rss.xml`,
    site_url: `${SITE_URL}/news/`,
    language: 'en',
    generator: '@modrinth/blog',
  })

  for (const article of sorted) {
    feed.item({
      title: article.title,
      description: article.summary,
      url: `${SITE_URL}/news/article/${article.slug}/`,
      guid: `${SITE_URL}/news/article/${article.slug}/`,
      date: article.date,
      custom_elements: [{ 'content:encoded': `<![CDATA[${article.html}]]>` }],
    })
  }

  await fs.mkdir(path.dirname(RSS_PATH), { recursive: true })
  await fs.writeFile(RSS_PATH, feed.xml({ indent: true }), 'utf8')
  console.log(`📂  RSS feed written to ${RSS_PATH}`)
}

async function generateJsonFile(articles): Promise<void> {
  const sorted = [...articles].sort(
    (a, b) => new Date(b.date).getTime() - new Date(a.date).getTime(),
  )
  const json = { articles: sorted }
  await fs.mkdir(path.dirname(JSON_PATH), { recursive: true })
  await fs.writeFile(JSON_PATH, JSON.stringify(json, null, 2), 'utf8')
  console.log(`📝  Wrote JSON articles to ${JSON_PATH}`)
}

async function deleteDirContents(dir: string) {
  try {
    const entries = await fs.readdir(dir, { withFileTypes: true })
    await Promise.all(
      entries.map(async (entry) => {
        const fullPath = path.join(dir, entry.name)
        if (entry.isDirectory()) {
          await fs.rm(fullPath, { recursive: true, force: true })
        } else {
          await fs.unlink(fullPath)
        }
      }),
    )
  } catch (error) {
    console.error(`❌  Error deleting contents of ${dir}:`, error)
    throw error
  }
}

async function copyPublicAssets() {
  console.log('🚚  Copying ./public to all PUBLIC_LOCATIONS...')
  for (const loc of PUBLIC_LOCATIONS) {
    await deleteDirContents(loc)
    await copyDir(PUBLIC_SRC, loc)
    console.log(`📂  Copied ./public to ${loc}`)
  }
  console.log('🎉  All public assets copied!')
}

async function main() {
  await compileArticles()
  await copyPublicAssets()
}

main().catch((e) => {
  console.error('❌  Error in compile.ts:', e)
  process.exit(1)
})
