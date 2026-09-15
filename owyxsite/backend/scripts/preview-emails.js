#!/usr/bin/env node
// Render every built-in Owyx email template with sample data to HTML files, so
// they can be opened in a browser and eyeballed. No SMTP or DB needed.
//
//   node scripts/preview-emails.js [outDir]   (default: ./email-previews)

const fs = require("fs");
const path = require("path");
const { render, TEMPLATE_KEYS } = require("../src/utils/emailTemplates");

const outDir = path.resolve(process.argv[2] || path.join(__dirname, "..", "email-previews"));
fs.mkdirSync(outDir, { recursive: true });

const sample = {
  nickname: "Steve",
  verificationLink: "https://owyx.site/verify?token=SAMPLE_TOKEN",
  resetLink: "https://owyx.site/reset-password?token=SAMPLE_TOKEN",
  oldNick: "OldNick",
  newNick: "Steve",
  serverName: "Owyx",
  serverIp: "play.owyx.site",
};

const index = [];
for (const key of TEMPLATE_KEYS) {
  const { subject, html } = render(key, sample);
  const file = path.join(outDir, `${key}.html`);
  fs.writeFileSync(file, html, "utf8");
  index.push({ key, subject, file });
  console.log(`✓ ${key.padEnd(18)} — ${subject}`);
}

fs.writeFileSync(
  path.join(outDir, "index.html"),
  `<!doctype html><meta charset="utf-8"><title>Owyx email previews</title>
   <body style="font-family:sans-serif;background:#050508;color:#f3f4f6;padding:24px">
   <h1 style="color:#00e5ff">Owyx email previews</h1><ul>` +
    index.map((i) => `<li><a style="color:#00e5ff" href="./${i.key}.html">${i.key}</a> — ${i.subject}</li>`).join("") +
    `</ul></body>`,
  "utf8"
);

console.log(`\nWrote ${index.length} templates to ${outDir}`);
