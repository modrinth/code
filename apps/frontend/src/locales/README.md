# Locales Directory

This directory contains translation files for different languages supported by Modrinth.

## Directory Structure

Each language has its own subdirectory with the following files:
- `index.json` - Main translation strings (auto-generated from Crowdin)
- `languages.json` - Language name mappings (manual, en-US only)

## Adding New Languages

When adding a new language to Crowdin:

1. **Automatic**: `index.json` files will be automatically synced from Crowdin
2. **Manual**: Update `en-US/languages.json` with the new language's English name

### Updating languages.json

Add entries to `en-US/languages.json` following this format:
```json
{
  "locale-code": "English Language Name"
}
```

**Examples:**
- `"zh-CN": "Chinese Simplified"`
- `"zh-TW": "Chinese Traditional"`
- `"ar-SA": "Arabic, Saudi Arabia"`
- `"pt-BR": "Portuguese, Brazilian"`

**Reference:** Use [Crowdin's official language codes](https://support.crowdin.com/developer/language-codes/) for proper naming conventions.

## RTL Language Support

Languages marked as RTL in `nuxt.config.ts` will display an "Experimental" badge in the language selector:
- Arabic (`ar-SA`)
- Hebrew (`he-IL`)
- Persian (`fa-IR`)