// Built-in Owyx email templates (inline CSS, dark + cyan, mobile-friendly).
// Version-controlled and renderable without a database or SMTP, so they can be
// previewed/tested any time. render(key, vars) -> { subject, html }.

const BRAND = {
  name: "Owyx",
  accent: "#00e5ff",
  ink: "#041018",
  bg: "#050508",
  panel: "#131319",
  line: "#2a2b30",
  text: "#f3f4f6",
  muted: "#9aa0a8",
  site: "https://owyx.site",
  discord: "https://discord.gg/owyx",
};

function esc(v) {
  return String(v ?? "").replace(/[&<>"']/g, (c) => (
    { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]
  ));
}

// Shared shell. `button` is optional { href, label }.
function layout({ title, intro, bodyHtml, button, footerNote }) {
  const btn = button
    ? `<tr><td style="padding:8px 0 4px;">
         <a href="${esc(button.href)}" target="_blank"
            style="display:inline-block;background:${BRAND.accent};color:${BRAND.ink};
                   text-decoration:none;font-weight:700;font-size:15px;
                   padding:12px 22px;border-radius:10px;">${esc(button.label)}</a>
       </td></tr>
       <tr><td style="padding:6px 0 0;color:${BRAND.muted};font-size:12px;line-height:1.5;">
         Если кнопка не работает, скопируйте ссылку:<br/>
         <span style="color:${BRAND.accent};word-break:break-all;">${esc(button.href)}</span>
       </td></tr>`
    : "";

  return `<!doctype html>
<html lang="ru"><head><meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>${esc(title)}</title></head>
<body style="margin:0;padding:0;background:${BRAND.bg};">
  <table role="presentation" width="100%" cellpadding="0" cellspacing="0"
         style="background:${BRAND.bg};padding:28px 12px;">
    <tr><td align="center">
      <table role="presentation" width="100%" cellpadding="0" cellspacing="0"
             style="max-width:560px;width:100%;background:${BRAND.panel};
                    border:1px solid ${BRAND.line};border-radius:16px;overflow:hidden;">
        <tr><td style="padding:24px 28px 8px;">
          <span style="font-size:22px;font-weight:800;color:${BRAND.accent};
                       letter-spacing:-0.01em;">Owyx</span>
        </td></tr>
        <tr><td style="padding:4px 28px 0;">
          <h1 style="margin:0 0 6px;color:${BRAND.text};font-size:20px;
                     font-family:Arial,Helvetica,sans-serif;">${esc(title)}</h1>
          <p style="margin:0 0 16px;color:${BRAND.muted};font-size:14px;line-height:1.6;
                    font-family:Arial,Helvetica,sans-serif;">${intro}</p>
        </td></tr>
        <tr><td style="padding:0 28px 8px;font-family:Arial,Helvetica,sans-serif;
                       color:${BRAND.text};font-size:14px;line-height:1.6;">
          <table role="presentation" width="100%" cellpadding="0" cellspacing="0">
            ${bodyHtml || ""}
            ${btn}
          </table>
        </td></tr>
        <tr><td style="padding:18px 28px 24px;">
          <div style="border-top:1px solid ${BRAND.line};margin-bottom:14px;"></div>
          <p style="margin:0;color:${BRAND.muted};font-size:12px;line-height:1.6;
                    font-family:Arial,Helvetica,sans-serif;">
            ${footerNote || `Это письмо отправлено сервисом ${BRAND.name}.`}<br/>
            <a href="${BRAND.site}" style="color:${BRAND.accent};text-decoration:none;">${BRAND.site.replace("https://", "")}</a>
            &nbsp;·&nbsp;
            <a href="${BRAND.discord}" style="color:${BRAND.accent};text-decoration:none;">Discord</a>
          </p>
        </td></tr>
      </table>
      <p style="max-width:560px;margin:14px auto 0;color:${BRAND.muted};font-size:11px;
                font-family:Arial,Helvetica,sans-serif;">© ${new Date().getFullYear()} Owyx</p>
    </td></tr>
  </table>
</body></html>`;
}

const templates = {
  verify: (v) => ({
    subject: "Owyx — подтвердите email",
    html: layout({
      title: "Подтвердите email",
      intro: `Привет, <b style="color:${BRAND.text}">${esc(v.nickname || "игрок")}</b>! Осталось подтвердить адрес — и аккаунт Owyx готов.`,
      button: { href: v.verificationLink || "#", label: "Подтвердить email" },
      footerNote: "Если вы не создавали аккаунт Owyx — просто проигнорируйте это письмо.",
    }),
  }),

  reset: (v) => ({
    subject: "Owyx — сброс пароля",
    html: layout({
      title: "Сброс пароля",
      intro: `Мы получили запрос на смену пароля для аккаунта <b style="color:${BRAND.text}">${esc(v.nickname || "игрок")}</b>. Ссылка действует ограниченное время.`,
      button: { href: v.resetLink || "#", label: "Сменить пароль" },
      footerNote: "Если вы не запрашивали сброс — проигнорируйте письмо, пароль останется прежним.",
    }),
  }),

  passwordChanged: (v) => ({
    subject: "Owyx — пароль изменён",
    html: layout({
      title: "Пароль изменён",
      intro: `Пароль аккаунта <b style="color:${BRAND.text}">${esc(v.nickname || "игрок")}</b> только что был изменён.`,
      bodyHtml: `<tr><td style="padding:4px 0 8px;color:${BRAND.muted};">Если это были не вы — срочно восстановите доступ через «Забыли пароль» и напишите в Discord.</td></tr>`,
      button: { href: `${BRAND.site}/login`, label: "Войти" },
      footerNote: "Уведомление о безопасности аккаунта Owyx.",
    }),
  }),

  welcome: (v) => ({
    subject: "Owyx — добро пожаловать",
    html: layout({
      title: "Добро пожаловать в Owyx",
      intro: `Привет, <b style="color:${BRAND.text}">${esc(v.nickname || "игрок")}</b>! Аккаунт активен. Скачай лаунчер, войди этим же аккаунтом — и играй.`,
      bodyHtml: `<tr><td style="padding:4px 0 8px;color:${BRAND.muted};">Гостю хватит ника, а аккаунт даёт скин и плюшки.</td></tr>`,
      button: { href: `${BRAND.site}/download`, label: "Скачать лаунчер" },
      footerNote: "Рады видеть тебя в Owyx.",
    }),
  }),

  emailChange: (v) => ({
    subject: "Owyx — подтверждение новой почты",
    html: layout({
      title: "Смена email",
      intro: `Кто-то (надеемся, вы) указал этот адрес как новую почту аккаунта Owyx. Введите код на сайте, чтобы подтвердить смену.`,
      bodyHtml: `<tr><td style="padding:6px 0 10px;">
          <div style="font-size:26px;font-weight:800;letter-spacing:6px;color:${BRAND.accent};
                      background:#0e0e14;border:1px solid ${BRAND.line};border-radius:10px;
                      padding:14px 0;text-align:center;">${esc(v.code || "------")}</div>
        </td></tr>
        <tr><td style="padding:0 0 6px;color:${BRAND.muted};">Код действует 15 минут. Если это были не вы — просто проигнорируйте письмо, почта не изменится.</td></tr>`,
      footerNote: "Подтверждение смены email аккаунта Owyx.",
    }),
  }),

  nicknameChanged: (v) => ({
    subject: "Owyx — ник изменён",
    html: layout({
      title: "Ник изменён",
      intro: `Ник аккаунта изменён с <b style="color:${BRAND.text}">${esc(v.oldNick || "—")}</b> на <b style="color:${BRAND.accent}">${esc(v.newNick || "—")}</b>.`,
      bodyHtml: `<tr><td style="padding:4px 0 8px;color:${BRAND.muted};">Если это были не вы — напишите в Discord. Следующая смена ника доступна через 30 дней.</td></tr>`,
      footerNote: "Уведомление о безопасности аккаунта Owyx.",
    }),
  }),
};

function render(key, vars = {}) {
  const t = templates[key];
  if (!t) throw new Error(`Unknown email template: ${key}`);
  return t(vars);
}

const TEMPLATE_KEYS = Object.keys(templates);

module.exports = { render, TEMPLATE_KEYS, BRAND };
