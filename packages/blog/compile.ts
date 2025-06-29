import { promises as fs } from 'fs'
import * as path from 'path'
import fg from 'fast-glob'
import matter from 'gray-matter'
import { md } from '@modrinth/utils/parse'
import { minify } from 'html-minifier-terser'
import { repoPath, copyDir, toVarName } from './utils'

const ARTICLES_GLOB = repoPath('packages/blog/articles/**/*.md')
const COMPILED_DIR = repoPath('packages/blog/compiled')
const ROOT_FILE = path.join(COMPILED_DIR, 'index.ts')
const PUBLIC_SRC = repoPath('packages/blog/public')
const PUBLIC_LOCATIONS = [repoPath('apps/frontend/src/public/news/article')]

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

async function compileArticles() {
  await ensureCompiledDir()

  const files = await fg([ARTICLES_GLOB])
  console.log(`🔎  Found ${files.length} markdown articles!`)
  const articleExports: string[] = []
  const articlesArray: string[] = []

  for (const file of files) {
    const src = await fs.readFile(file, 'utf8')
    const { content, data } = matter(src)

    const { title, summary, date, ...rest } = data
    if (!title || !summary || !date) {
      console.error(`❌  Missing required frontmatter in ${file}. Required: title, summary, date`)
      process.exit(1)
    }

    const html = md().render(content)
    const minifiedHtml = await minify(html, {
      collapseWhitespace: true,
      removeComments: true,
      minifyCSS: true,
      minifyJS: true,
    })

    const slug = path.basename(file, '.md')
    const varName = toVarName(slug)
    const exportFile = path.join(COMPILED_DIR, `${varName}.ts`)
    const thumbnailPresent = await hasThumbnail(slug)

    const ts = `
// AUTO-GENERATED FILE - DO NOT EDIT
export const article = {
  html: ${JSON.stringify(minifiedHtml)},
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
    // Delete existing contents first
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
