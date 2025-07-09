# Modrinth Blog Articles

This package contains the articles for the Modrinth blog. The articles are written in Markdown and are rendered on the Modrinth website.

## How to add a new article

Write your article in the `articles` directory. The filename should be the slug of the article, and the file should have a `.md` extension. The first line of the file should be the frontmatter, which contains metadata about the article such as the title, summary and date of writing.

### Example Frontmatter

```md
---
title: Quintupling Creator Revenue and Becoming Sustainable
short_title: Becoming Sustainable
summary: Announcing an update to our monetization program, creator split, and more!
short_summary: Announcing 5x creator revenue and updates to the monetization program.
date: 2024-09-13T12:00:00-08:00
---
```

You **can** link other articles in the frontmatter, but it's recommended you're explicit about it, for example: `https://modrinth.com/news/article/...` instead of `/news/article/...`. It's not a requirement though, you just have to be careful about it.

You can place images in the `public/{slug}/...` directory, the thumbnail must be a `.webp` file named `thumbnail.webp` in the same public directory.
