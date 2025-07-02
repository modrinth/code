import * as path from 'path'
import { repoPath } from './utils'

/**
 * The glob pattern to find all markdown articles which should be compiled.
 */
export const ARTICLES_GLOB = repoPath('packages/blog/articles/**/*.md')

/**
 * The directory where compiled articles are stored.
 */
export const COMPILED_DIR = repoPath('packages/blog/compiled')
export const ROOT_FILE = path.join(COMPILED_DIR, 'index.ts')

/**
 * The source directory for public assets used in articles.
 */
export const PUBLIC_SRC = repoPath('packages/blog/public')

/**
 * An array of git-repository-root-relative paths where public assets should be copied to.
 */
export const PUBLIC_LOCATIONS = [repoPath('apps/frontend/src/public/news/article')]

/**
 * The git-repository-root-relative path to the frontend RSS feed file.
 */
export const RSS_PATH = repoPath('apps/frontend/src/public/news/feed/rss.xml')
export const JSON_PATH = repoPath('apps/frontend/src/public/news/feed/articles.json')

/**
 * The base URL of the Modrinth site, used for the RSS feed.
 */
export const SITE_URL = 'https://modrinth.com'
