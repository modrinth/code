/**
 * One-shot generator for owyxsite EN/RU locale dictionaries (UI + legal).
 * Run: node owyxsite/frontend/scripts/generate-locales.mjs
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const outDir = path.join(__dirname, "..", "locales");

const lastModified = "September 16, 2026";
const lastModifiedRu = "16 сентября 2026";

function section(id, title, paragraphs) {
  return { id, title, paragraphs };
}

function buildEn() {
  const terms = {
    title: "Terms of Use",
    lastModified,
    intro:
      'These Terms of Use are entered into by and between You and Owyx Team ("Owyx", "we", "us", or "our"). Owyx Team is an independent individual maintainer / project team (not a registered company unless later stated on this site). These Terms, together with our Privacy Policy and Launcher EULA (collectively, the "Agreements"), govern your access to and use of owyx.site, www.owyx.site, api.owyx.site, related subdomains, and the Owyx desktop launcher and related client software (collectively, the "Service"), whether as a guest or a registered user.',
    sections: [
      section("acceptance", "Acceptance of the Terms of Use", [
        "Please read these Terms carefully before you start to use the Service. By accessing or using the Service, creating an account, downloading the launcher, or otherwise interacting with Owyx domains or clients, you accept and agree to be bound by these Terms and the Privacy Policy and Launcher EULA. If you do not want to agree, you must not access or use the Service.",
        "The Service is offered and available to users who are 13 years of age or older. By using the Service, you represent that you meet this requirement and have legal capacity to form a binding agreement. If you do not meet these requirements, you must not use the Service.",
        "If you use the Service on behalf of an organization, you represent that you have authority to bind that organization to these Terms.",
      ]),
      section("parties", "Parties and nature of Owyx Team", [
        'The contracting party providing access to the Service is Owyx Team (the sole maintainer / operator of the Owyx project as presented on owyx.site). References to "Company" in similar industry terms of use should be read as Owyx Team for this Service.',
        "Owyx Team is not affiliated with, endorsed by, or sponsored by Mojang Studios, Microsoft Corporation, Rinth, Inc., or Modrinth. Minecraft® is a trademark of Mojang Synergies AB / Microsoft.",
        "We may provide contact channels (for example Discord or an email address published on the site). We do not publish personal passport, tax, or bank requisites on these pages.",
      ]),
      section("changes", "Changes to the Terms of Use", [
        "We may revise and update these Terms from time to time in our sole discretion. Changes are effective when posted on this page and apply to all subsequent access and use of the Service.",
        "Your continued use of the Service after revised Terms are posted means you accept the changes. You are expected to check this page periodically.",
      ]),
      section("service", "About the Service", [
        "The Service includes: (a) the website and account system on Owyx domains; (b) APIs on api.owyx.site; (c) the Owyx launcher and related desktop client features; (d) optional cosmetics, friends, catalog, news, and community features we enable from time to time.",
        "Depending on configuration, you may be able to play using an Owyx site nickname (offline-mode style profile) and/or a Microsoft-linked Minecraft account. Availability of features may change without notice.",
        "Owyx does not sell you a license to Minecraft itself. Any play that requires a valid Minecraft / Microsoft license remains your responsibility under Mojang/Microsoft terms. Offline-mode or nickname play does not replace, waive, or grant rights under the Minecraft EULA or Microsoft services agreements.",
        "We reserve the right to withdraw, restrict, or amend any part of the Service without notice. We are not liable if the Service is unavailable for any period.",
      ]),
      section("accounts", "Accessing the Service and account security", [
        "You are responsible for arranging your own access to the Service and for ensuring that anyone who accesses the Service through your connection complies with these Terms.",
        "Registration information you provide must be correct, current, and complete. Account information is governed by our Privacy Policy.",
        "You must treat email, password, session tokens, and API client keys as confidential. Your account is personal to you. You must not share credentials. You agree to notify us promptly of any unauthorized access.",
        "We may disable any username, nickname, password, or account at any time, for any or no reason, including suspected Terms violations, abuse, fraud, or legal risk.",
        "You are responsible for all activity under your account, including actions by anyone you allow to use it.",
      ]),
      section("minecraft", "Minecraft, third-party rights, and compliance", [
        "Minecraft game files, assets, trademarks, and online services are owned by Mojang/Microsoft. Owyx is a third-party launcher / community ecosystem and is not an official Minecraft product.",
        "You agree to comply with applicable Mojang/Microsoft terms (including the Minecraft EULA and related policies) whenever they apply to your use of Minecraft. Nothing in these Terms authorizes infringement, piracy, circumvention of technical protection measures where prohibited, or unauthorized redistribution of Minecraft.",
        "Servers, packs, mods, resource packs, and other third-party content you obtain through or outside Owyx remain subject to their own licenses. You are solely responsible for verifying that you have rights to use such content.",
        "Owyx may integrate Microsoft OAuth / Minecraft authentication flows for licensed accounts. Those flows are subject to Microsoft terms. Owyx is not responsible for Microsoft outages, account bans, or authentication changes.",
      ]),
      section("launcher", "Launcher software and open source", [
        "The Owyx launcher is based on open-source software derived from the Modrinth App / Theseus lineage and is generally distributed under GNU GPL v3 (see the public repository LICENSE and COPYING.md). GPL covers source code rights — not the Owyx brand.",
        "You may use the launcher subject to GPL-3 (for GPL-covered components) and this Launcher EULA / Terms for Service access. Forks must not use Owyx branding (see COPYING.md and brand/).",
        "Auto-update, Discord presence, cosmetics sync, friends, and catalog features may phone home to Owyx APIs. By using those features you consent to the related network calls described in the Privacy Policy.",
      ]),
      section("api", "API usage", [
        "We may provide HTTP APIs (including api.owyx.site). We grant a limited, non-exclusive, non-sublicensable, revocable license to use the API for personal or project use that complies with these Terms and applicable law.",
        "You must not abuse rate limits, scrape aggressively, attempt unauthorized access, share secret client keys publicly, or use the API to harm the Service or other users.",
        "We may revoke API access at any time. You indemnify Owyx Team against claims arising from your API use.",
      ]),
      section("ugc", "User content and contributions", [
        'You may upload or submit nicknames, avatars, skins, messages, catalog-related materials, or other content ("User Content"). You retain ownership of your User Content, but grant Owyx Team a worldwide, non-exclusive, royalty-free license to host, display, transmit, and process it as needed to operate the Service.',
        "You represent that you have all rights needed to submit User Content and that it does not infringe others’ rights or violate law.",
        "We may remove or refuse User Content at our discretion. We do not pre-screen all content and are not responsible for User Content.",
      ]),
      section("conduct", "Prohibited uses", [
        "You may use the Service only for lawful purposes. You agree not to:",
        "(a) violate any applicable law or regulation; (b) exploit or harm minors; (c) transmit malware, spam, or unauthorized advertising; (d) impersonate others or Owyx Team; (e) harass, threaten, or doxx users; (f) attempt unauthorized access to systems, accounts, or data; (g) interfere with Service integrity, including DDoS, scraping that degrades service, or reverse engineering where prohibited; (h) bypass bans, rate limits, or access controls; (i) use the Service to distribute illegal content; (j) misrepresent Owyx as an official Mojang/Microsoft/Modrinth product; (k) use Owyx trademarks in a misleading way.",
      ]),
      section("ip", "Intellectual property and trademarks", [
        "Except for User Content and third-party open-source components under their licenses, the Service’s branding, design, text, and proprietary materials are owned by Owyx Team or licensors and protected by intellectual property laws.",
        "The name Owyx, Owyx Team, related logos, crystal marks, and design system assets under brand/ are trademarks / trade dress of Owyx Team. You must not use them without prior written permission.",
        "No rights are transferred to you except the limited license to use the Service as expressly permitted.",
      ]),
      section("disclaimer", "Disclaimer of warranties", [
        'THE SERVICE IS PROVIDED ON AN "AS IS" AND "AS AVAILABLE" BASIS WITHOUT WARRANTIES OF ANY KIND, WHETHER EXPRESS, IMPLIED, OR STATUTORY, INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, TITLE, AND NON-INFRINGEMENT.',
        "We do not warrant that the Service will be uninterrupted, secure, error-free, or free of harmful components, or that defects will be corrected.",
        "You use Minecraft, mods, packs, and third-party servers at your own risk.",
      ]),
      section("liability", "Limitation of liability", [
        "TO THE FULLEST EXTENT PERMITTED BY LAW, OWYX TEAM AND ITS CONTRIBUTORS SHALL NOT BE LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL, CONSEQUENTIAL, EXEMPLARY, OR PUNITIVE DAMAGES, OR ANY LOSS OF PROFITS, DATA, GOODWILL, OR BUSINESS OPPORTUNITY, ARISING FROM YOUR USE OF THE SERVICE.",
        "OUR TOTAL LIABILITY FOR ANY CLAIM RELATING TO THE SERVICE SHALL NOT EXCEED THE GREATER OF (A) THE AMOUNTS YOU PAID TO OWYX TEAM FOR THE SERVICE IN THE 12 MONTHS BEFORE THE CLAIM (IF ANY) OR (B) EUR 50 (OR LOCAL CURRENCY EQUIVALENT).",
        "Some jurisdictions do not allow certain limitations; in those cases our liability is limited to the maximum extent permitted.",
      ]),
      section("indemnity", "Indemnification", [
        "You agree to defend, indemnify, and hold harmless Owyx Team and contributors from any claims, damages, losses, and expenses (including reasonable legal fees) arising out of your use of the Service, your User Content, your violation of these Terms, or your violation of any third-party rights or law (including Mojang/Microsoft terms).",
      ]),
      section("termination", "Termination", [
        "You may stop using the Service at any time. We may suspend or terminate access immediately, without notice, for any reason including Terms violations.",
        "Upon termination, your right to use the Service ceases. Provisions that by nature should survive (including IP, disclaimers, liability limits, indemnity) will survive.",
      ]),
      section("law", "Governing law and disputes", [
        "These Terms are governed by the laws applicable at the place of the Owyx Team maintainer’s habitual residence, without regard to conflict-of-law rules, unless mandatory consumer law of your country provides otherwise.",
        "You agree to first contact us and attempt informal resolution. Where permitted, disputes shall be resolved in the courts competent for the maintainer’s habitual residence, subject to mandatory consumer venue rights.",
        "If any provision is held unenforceable, the remaining provisions remain in effect.",
      ]),
      section("entire", "Entire agreement", [
        "These Terms, the Privacy Policy, and the Launcher EULA constitute the entire agreement between you and Owyx Team regarding the Service and supersede prior understandings on the same subject.",
        "Our failure to enforce a provision is not a waiver. You may not assign these Terms without our consent; we may assign them in connection with a project transfer.",
      ]),
      section("contact", "Contact", [
        "Questions about these Terms may be sent through the contact channels published on owyx.site (for example Discord community links). We do not list personal legal requisites on this page.",
      ]),
    ],
  };

  const privacy = {
    title: "Privacy Policy",
    lastModified,
    intro:
      'This Privacy Policy explains how Owyx Team ("Owyx", "we", "us") collects, uses, stores, and shares information when you use owyx.site, api.owyx.site, and the Owyx launcher (the "Service"). Owyx Team is an independent project maintainer / team. This policy is designed to be transparent; it is not legal advice.',
    sections: [
      section("scope", "Scope", [
        "This policy applies to information we collect on the website, via email or other messages you send us, through the desktop launcher, and through APIs that support the Service.",
        "It does not apply to third-party sites, Minecraft servers, Discord, Microsoft, Mojang, Modrinth, payment processors, or other services you use outside Owyx — those have their own policies.",
      ]),
      section("foreword", "Legal foreword", [
        "This document is intended to support transparency expectations under frameworks such as the EU GDPR and similar privacy laws, to the extent they apply. Because Owyx Team may be a small / solo operator, some corporate formalities may not apply in the same way as for large companies; we still aim to respect your rights.",
      ]),
      section("controller", "Data controller", [
        "The data controller for personal data processed through the Service is Owyx Team as operator of owyx.site / api.owyx.site. Contact channels are published on the site. We do not publish home address or national ID requisites here.",
      ]),
      section("collect", "Information we collect", [
        "Account data: email address, password hash (not plaintext password), nickname, role/trust flags, optional Discord link, avatar/skin uploads, email verification status, registration timestamps.",
        "Session data: authentication tokens / session hashes, device or client metadata reasonably needed for security, IP addresses in logs, approximate timestamps of access.",
        "Launcher / API usage: requests to catalog, cosmetics, friends, presence heartbeats (online/playing status and optional instance name), admin actions if you are staff.",
        "Technical logs: server logs, error reports, rate-limit and abuse-prevention signals, CDN/WAF metadata if used.",
        "Communications: messages you send to support channels.",
        "We do not intentionally collect special-category data. Please do not submit sensitive personal data in nicknames or skins.",
      ]),
      section("use", "How we use information", [
        "To create and secure accounts; authenticate launcher and site sessions; provide cosmetics, friends, catalog, and related features; prevent fraud and abuse; debug and improve the Service; communicate about security or account issues; comply with law.",
        "We do not sell your personal data. We do not use your data for third-party advertising networks as a core business model.",
      ]),
      section("legal_bases", "Legal bases (where GDPR-like rules apply)", [
        "Contract / steps prior to contract: providing the account and launcher features you request.",
        "Legitimate interests: securing the Service, preventing abuse, understanding aggregate reliability.",
        "Consent: where we ask for it (for example optional marketing, if ever enabled).",
        "Legal obligation: when we must retain or disclose data to comply with law.",
      ]),
      section("sharing", "Sharing and processors", [
        "We may use infrastructure providers (hosting, database, object storage, email delivery, CDN/WAF, captcha, Discord for community). They process data on our instructions or as independent controllers for their own services.",
        "Microsoft / Mojang receive data only when you choose Microsoft login for Minecraft — according to their terms.",
        "We may disclose information if required by law, to protect rights and safety, or in connection with a good-faith transfer of the project.",
      ]),
      section("retention", "Retention", [
        "Account data is kept while your account remains open and for a reasonable period afterward for security, backups, and legal claims.",
        "Logs are retained for a limited operational period unless needed longer for investigations.",
        "You may request deletion subject to legal exceptions (see Rights).",
      ]),
      section("security", "Security", [
        "We use reasonable technical and organizational measures (hashed passwords, HTTPS, session controls, access-limited admin tools). No method of transmission or storage is 100% secure.",
      ]),
      section("transfers", "International transfers", [
        "Servers and processors may be located in different countries. Where required, we rely on appropriate safeguards or your necessity for the Service.",
      ]),
      section("cookies", "Cookies and local storage", [
        "We use cookies / local storage for login sessions, language preference, and similar functional needs. You can clear site data in your browser; some features may stop working.",
      ]),
      section("children", "Children", [
        "The Service is not directed to children under 13. We do not knowingly collect personal data from children under 13. If you believe we have, contact us and we will delete it promptly.",
      ]),
      section("rights", "Your rights", [
        "Depending on your jurisdiction, you may have rights to access, rectify, erase, restrict, port, or object to certain processing, and to withdraw consent.",
        "To exercise rights, contact us via published channels and verify your identity. We will respond within a reasonable period (typically within 30 days where required).",
        "You may lodge a complaint with a supervisory authority if applicable.",
      ]),
      section("changes_p", "Changes", [
        'We may update this Privacy Policy by posting a new version on this page and updating the "Last modified" date. Continued use after changes means you acknowledge the updated policy.',
      ]),
      section("contact_p", "Contact", [
        "Privacy questions: use the contact channels on owyx.site. We do not list personal postal requisites on this page.",
      ]),
    ],
  };

  const eula = {
    title: "Launcher End-User License Agreement (EULA)",
    lastModified,
    intro:
      'This Launcher EULA is an agreement between you and Owyx Team regarding the Owyx desktop launcher and related client components (the "Software"). It supplements the Terms of Use and Privacy Policy. If there is a conflict about Software licensing, GPL-3 terms for GPL-covered code control for source distribution rights; this EULA governs branded Service access and acceptable use.',
    sections: [
      section("license_grant", "License grant", [
        "Subject to these terms, Owyx Team grants you a personal, non-exclusive, non-transferable, revocable license to install and use the Software to access the Service and play Minecraft in configurations the Software supports.",
        "Open-source components are also licensed under their stated licenses (including GNU GPL v3 for applicable launcher packages). You may exercise GPL rights for those components as provided by GPL-3.",
      ]),
      section("brand", "Brand and trademark restrictions", [
        "This license does not grant rights to the Owyx name, logos, icons, or design system. Forks and redistributed builds must not use Owyx branding. See repository COPYING.md and the brand/ directory inventory.",
      ]),
      section("restrictions", "Restrictions", [
        "Except as allowed by GPL-3 for GPL-covered code or applicable law, you must not: misrepresent the Software as an official Mojang/Microsoft client; use the Software to violate Minecraft/Microsoft terms; bypass Owyx access controls; introduce malware; or remove attribution / copyright notices where required.",
      ]),
      section("updates", "Updates and network calls", [
        "The Software may automatically check for updates, authenticate sessions, sync cosmetics, send friends presence, and fetch catalog/news from Owyx servers. You can limit some features by not signing in or by network configuration, but core update checks may still occur in release builds.",
      ]),
      section("no_game_license", "No Minecraft license", [
        "The Software does not include Minecraft game ownership. Obtaining and using Minecraft remains subject to Mojang/Microsoft terms. Nickname / offline-mode profiles are a technical feature of certain server configurations and do not equal a purchased Minecraft license.",
      ]),
      section("warranty_eula", "Disclaimer and liability", [
        'THE SOFTWARE IS PROVIDED "AS IS" WITHOUT WARRANTY. LIABILITY LIMITATIONS IN THE TERMS OF USE APPLY TO THE SOFTWARE.',
      ]),
      section("termination_eula", "Termination", [
        "This EULA terminates if you breach it or if we discontinue the Software. GPL rights to corresponding source (where applicable) survive according to GPL-3.",
      ]),
      section("contact_eula", "Contact", [
        "Questions: contact channels published on owyx.site.",
      ]),
    ],
  };

  return {
    meta: { locale: "en_US", languageName: "English" },
    header: {
      home: "Home",
      download: "Download",
      login: "Sign in",
      register: "Register",
      cabinet: "Account",
      admin: "Admin",
      logout: "Sign out",
      openMenu: "Open menu",
      closeMenu: "Close menu",
      languageAria: "Switch language",
      languageTitle: "Language (EN / RU)",
      logoHome: "Owyx — home",
    },
    footer: {
      madeBy: "made by ebluffy",
      terms: "Terms",
      privacy: "Privacy",
      eula: "EULA",
      discord: "Discord",
      attribution:
        "The launcher uses open-source code from the Modrinth (Theseus) client — a fork under our own brand, not affiliated with Modrinth.",
    },
    legal: {
      breadcrumbHome: "Home",
      breadcrumbDocs: "Legal",
      draftBadge: "",
      notLegalAdvice: "This page is provided for transparency and is not legal advice.",
      terms,
      privacy,
      eula,
    },
    common: { skipToContent: "Skip to content" },
  };
}

function buildRu() {
  // Import structure from a compact Russian mirror — keep keys identical to EN.
  const en = buildEn();
  const mapDoc = (enDoc, title, lastMod, intro, sectionTitles, paragraphsById) => ({
    title,
    lastModified: lastMod,
    intro,
    sections: enDoc.sections.map((s) => ({
      id: s.id,
      title: sectionTitles[s.id] || s.title,
      paragraphs: paragraphsById[s.id] || s.paragraphs,
    })),
  });

  // Full RU texts (parallel to EN)
  const terms = {
    title: "Пользовательское соглашение",
    lastModified: lastModifiedRu,
    intro:
      "Настоящее Пользовательское соглашение заключается между Вами и Owyx Team («Owyx», «мы», «нас»). Owyx Team — независимый мейнтейнер / команда проекта (как правило, один разработчик-владелец; не зарегистрированная компания, если иное прямо не указано на сайте). Настоящие условия вместе с Политикой конфиденциальности и EULA лаунчера (совместно — «Соглашения») регулируют доступ к сайту owyx.site, www.owyx.site, api.owyx.site, связанным поддоменам, а также к десктоп-лаунчеру Owyx и связанному клиентскому ПО (совместно — «Сервис»), как для гостей, так и для зарегистрированных пользователей.",
    sections: en.legal.terms.sections.map((s) => {
      const ru = RU_TERMS[s.id];
      return { id: s.id, title: ru?.title || s.title, paragraphs: ru?.paragraphs || s.paragraphs };
    }),
  };
  const privacy = {
    title: "Политика конфиденциальности",
    lastModified: lastModifiedRu,
    intro:
      "Эта Политика объясняет, как Owyx Team («Owyx», «мы») собирает, использует, хранит и передаёт информацию при использовании owyx.site, api.owyx.site и лаунчера Owyx («Сервис»). Owyx Team — независимый мейнтейнер / команда проекта. Документ носит информационный характер и не является юридической консультацией.",
    sections: en.legal.privacy.sections.map((s) => {
      const ru = RU_PRIVACY[s.id];
      return { id: s.id, title: ru?.title || s.title, paragraphs: ru?.paragraphs || s.paragraphs };
    }),
  };
  const eula = {
    title: "Лицензионное соглашение лаунчера (EULA)",
    lastModified: lastModifiedRu,
    intro:
      "Настоящее EULA — соглашение между Вами и Owyx Team относительно десктоп-лаунчера Owyx и связанных компонентов («ПО»). Оно дополняет Пользовательское соглашение и Политику конфиденциальности. В части прав на исходный код GPL-покрытых компонентов действует GNU GPL v3; настоящее EULA регулирует брендированный доступ к Сервису и допустимое использование.",
    sections: en.legal.eula.sections.map((s) => {
      const ru = RU_EULA[s.id];
      return { id: s.id, title: ru?.title || s.title, paragraphs: ru?.paragraphs || s.paragraphs };
    }),
  };

  void mapDoc;
  return {
    meta: { locale: "ru_RU", languageName: "Русский" },
    header: {
      home: "Главная",
      download: "Скачать",
      login: "Войти",
      register: "Регистрация",
      cabinet: "Личный кабинет",
      admin: "Админ-панель",
      logout: "Выйти",
      openMenu: "Открыть меню",
      closeMenu: "Закрыть меню",
      languageAria: "Сменить язык",
      languageTitle: "Язык (RU / EN)",
      logoHome: "Owyx — на главную",
    },
    footer: {
      madeBy: "сделано ebluffy",
      terms: "Соглашение",
      privacy: "Конфиденциальность",
      eula: "EULA",
      discord: "Discord",
      attribution:
        "Лаунчер использует код open-source клиента Modrinth (Theseus) — форк под себя, не аффилирован с Modrinth.",
    },
    legal: {
      breadcrumbHome: "Главная",
      breadcrumbDocs: "Документы",
      draftBadge: "",
      notLegalAdvice:
        "Страница носит информационный характер и не является юридической консультацией.",
      terms,
      privacy,
      eula,
    },
    common: { skipToContent: "К содержимому" },
  };
}

const RU_TERMS = {
  acceptance: {
    title: "Принятие условий",
    paragraphs: [
      "Внимательно прочитайте условия до начала использования Сервиса. Получая доступ к Сервису, создавая аккаунт, скачивая лаунчер или иным образом взаимодействуя с доменами и клиентами Owyx, Вы принимаете эти условия, Политику конфиденциальности и EULA лаунчера. Если Вы не согласны — не используйте Сервис.",
      "Сервис предназначен для пользователей старше 13 лет. Используя Сервис, Вы подтверждаете соответствие этому требованию и наличие правоспособности. Если требования не выполнены — не используйте Сервис.",
      "Если Вы действуете от имени организации, Вы подтверждаете полномочия связать её этими условиями.",
    ],
  },
  parties: {
    title: "Стороны и статус Owyx Team",
    paragraphs: [
      "Стороной, предоставляющей доступ к Сервису, является Owyx Team (мейнтейнер / оператор проекта Owyx на доменах owyx.site).",
      "Owyx Team не аффилирован с Mojang Studios, Microsoft Corporation, Rinth, Inc. или Modrinth и не является их официальным продуктом. Minecraft® — товарный знак Mojang Synergies AB / Microsoft.",
      "Контактные каналы (например Discord или email на сайте) могут публиковаться отдельно. Персональные паспортные, налоговые и банковские реквизиты на этих страницах не указываются.",
    ],
  },
  changes: {
    title: "Изменение условий",
    paragraphs: [
      "Мы можем обновлять условия по своему усмотрению. Изменения вступают в силу с момента публикации на этой странице и применяются к последующему использованию Сервиса.",
      "Продолжение использования после публикации изменений означает согласие с ними. Рекомендуем периодически проверять эту страницу.",
    ],
  },
  service: {
    title: "О Сервисе",
    paragraphs: [
      "Сервис включает: (a) сайт и аккаунты на доменах Owyx; (b) API на api.owyx.site; (c) лаунчер Owyx и связанные функции клиента; (d) опциональные косметику, друзей, каталог, новости и иные функции, которые мы включаем.",
      "В зависимости от настроек Вы можете играть по нику сайта Owyx (профиль offline-mode) и/или через Microsoft-аккаунт Minecraft. Набор функций может меняться без уведомления.",
      "Owyx не продаёт Вам лицензию на саму игру Minecraft. Если для режима игры нужна лицензия Minecraft/Microsoft — это Ваша ответственность по условиям Mojang/Microsoft. Игра по нику / offline-mode не заменяет и не выдаёт права по EULA Minecraft.",
      "Мы можем ограничивать или изменять Сервис без уведомления и не несём ответственности за недоступность Сервиса.",
    ],
  },
  accounts: {
    title: "Доступ и безопасность аккаунта",
    paragraphs: [
      "Вы отвечаете за организацию доступа к Сервису и за соблюдение условий всеми, кто пользуется Вашим подключением.",
      "Регистрационные данные должны быть актуальными и достоверными. Обработка данных — по Политике конфиденциальности.",
      "Email, пароль, сессионные токены и клиентские ключи API конфиденциальны. Аккаунт персонален; передавать доступ нельзя. О несанкционированном доступе нужно сообщить нам.",
      "Мы можем отключить ник, пароль или аккаунт в любое время, в том числе при подозрении на нарушение условий, злоупотребления или правовой риск.",
      "Вы отвечаете за все действия в рамках аккаунта.",
    ],
  },
  minecraft: {
    title: "Minecraft, права третьих лиц и соблюдение правил",
    paragraphs: [
      "Игра Minecraft, её ассеты, товарные знаки и онлайн-сервисы принадлежат Mojang/Microsoft. Owyx — сторонний лаунчер / экосистема сообщества, а не официальный продукт Minecraft.",
      "Вы обязуетесь соблюдать применимые условия Mojang/Microsoft (включая EULA Minecraft), когда они относятся к Вашему использованию игры. Ничто в этих условиях не разрешает нарушение прав, пиратство, обход технических средств защиты там, где это запрещено законом, или несанкционированное распространение Minecraft.",
      "Серверы, сборки, моды и иной сторонний контент подчиняются своим лицензиям. Вы сами проверяете наличие прав на использование.",
      "Owyx может использовать Microsoft OAuth / авторизацию Minecraft для лицензионных аккаунтов — по условиям Microsoft. Мы не отвечаем за сбои Microsoft, блокировки аккаунтов или изменения их API.",
    ],
  },
  launcher: {
    title: "ПО лаунчера и open source",
    paragraphs: [
      "Лаунчер Owyx основан на open-source коде линии Modrinth App / Theseus и в целом распространяется по GNU GPL v3 (см. LICENSE и COPYING.md в репозитории). GPL относится к коду, а не к бренду Owyx.",
      "Использование лаунчера — по GPL-3 (для покрытых компонентов) и по настоящим условиям / EULA в части доступа к Сервису. Форки не должны использовать брендинг Owyx (см. COPYING.md и brand/).",
      "Автообновления, Discord presence, синхронизация косметики, друзья и каталог могут обращаться к API Owyx. Используя эти функции, Вы соглашаетесь на соответствующие сетевые запросы, описанные в Политике конфиденциальности.",
    ],
  },
  api: {
    title: "Использование API",
    paragraphs: [
      "Мы можем предоставлять HTTP API (включая api.owyx.site). Вам даётся ограниченная, неисключительная, непередаваемая, отзывная лицензия на использование API в личных / проектных целях при соблюдении условий и закона.",
      "Запрещены злоупотребление лимитами, агрессивный scraping, несанкционированный доступ, публичная утечка секретных ключей и использование API во вред Сервису или пользователям.",
      "Мы можем отозвать доступ к API в любое время. Вы возмещаете убытки Owyx Team, связанные с Вашим использованием API.",
    ],
  },
  ugc: {
    title: "Пользовательский контент",
    paragraphs: [
      "Вы можете загружать ники, аватары, скины, сообщения и иной контент («Пользовательский контент»). Права на него остаются у Вас, но Вы предоставляете Owyx Team всемирную неисключительную безвозмездную лицензию на размещение, отображение и обработку контента для работы Сервиса.",
      "Вы гарантируете наличие прав на публикацию контента и отсутствие нарушений закона и прав третьих лиц.",
      "Мы можем удалять контент по своему усмотрению и не обязаны предварительно модерировать всё содержимое.",
    ],
  },
  conduct: {
    title: "Запрещённое использование",
    paragraphs: [
      "Сервис можно использовать только в законных целях. Запрещается, в частности:",
      "(a) нарушать законы; (b) эксплуатировать или причинять вред несовершеннолетним; (c) распространять вредоносный код, спам и нежелательную рекламу; (d) выдавать себя за других или за Owyx Team; (e) преследовать пользователей; (f) пытаться получить несанкционированный доступ; (g) нарушать целостность Сервиса (DDoS, вредоносный scraping и т.п.); (h) обходить блокировки и лимиты; (i) распространять незаконный контент; (j) выдавать Owyx за официальный продукт Mojang/Microsoft/Modrinth; (k) вводить в заблуждение использованием товарных знаков Owyx.",
    ],
  },
  ip: {
    title: "Интеллектуальная собственность",
    paragraphs: [
      "За исключением Пользовательского контента и open-source компонентов по их лицензиям, брендинг, дизайн, тексты и иные материалы Сервиса принадлежат Owyx Team или лицензиарам.",
      "Имя Owyx, Owyx Team, логотипы, кристалл и ассеты из brand/ — средства индивидуализации Owyx Team. Их использование без письменного разрешения запрещено.",
      "Вам не передаются права, кроме явно предоставленной ограниченной лицензии на использование Сервиса.",
    ],
  },
  disclaimer: {
    title: "Отказ от гарантий",
    paragraphs: [
      "СЕРВИС ПРЕДОСТАВЛЯЕТСЯ «КАК ЕСТЬ» И «ПО МЕРЕ ДОСТУПНОСТИ» БЕЗ ЛЮБЫХ ГАРАНТИЙ, ЯВНЫХ ИЛИ ПОДРАЗУМЕВАЕМЫХ, ВКЛЮЧАЯ КОММЕРЧЕСКУЮ ПРИГОДНОСТЬ, ПРИГОДНОСТЬ ДЛЯ ОПРЕДЕЛЁННОЙ ЦЕЛИ И НЕНАРУШЕНИЕ ПРАВ.",
      "Мы не гарантируем бесперебойность, безопасность, отсутствие ошибок или вредоносных компонентов.",
      "Вы используете Minecraft, моды, сборки и сторонние серверы на свой риск.",
    ],
  },
  liability: {
    title: "Ограничение ответственности",
    paragraphs: [
      "В МАКСИМАЛЬНОЙ СТЕПЕНИ, ДОПУЩЕННОЙ ЗАКОНОМ, OWYX TEAM И УЧАСТНИКИ ПРОЕКТА НЕ ОТВЕЧАЮТ ЗА КОСВЕННЫЕ, СЛУЧАЙНЫЕ, СПЕЦИАЛЬНЫЕ, ПОСЛЕДУЮЩИЕ ИЛИ ШТРАФНЫЕ УБЫТКИ, УПУЩЕННУЮ ВЫГОДУ, ПОТЕРЮ ДАННЫХ ИЛИ ДЕЛОВОЙ РЕПУТАЦИИ, СВЯЗАННЫЕ С ИСПОЛЬЗОВАНИЕМ СЕРВИСА.",
      "СОВОКУПНАЯ ОТВЕТСТВЕННОСТЬ ПО ЛЮБОМУ ТРЕБОВАНИЮ НЕ ПРЕВЫШАЕТ БОЛЬШУЮ ИЗ СУММ: (A) ОПЛАТЫ, УПЛАЧЕННЫЕ ВАМИ OWYX TEAM ЗА СЕРВИС ЗА 12 МЕСЯЦЕВ ДО ТРЕБОВАНИЯ (ЕСЛИ БЫЛИ), ЛИБО (B) 50 ЕВРО (ИЛИ ЭКВИВАЛЕНТ).",
      "Если закон не допускает такие ограничения, ответственность ограничивается максимально допустимым пределом.",
    ],
  },
  indemnity: {
    title: "Возмещение убытков",
    paragraphs: [
      "Вы обязуетесь защищать и возмещать Owyx Team убытки и расходы (включая разумные юридические издержки), возникшие из-за Вашего использования Сервиса, контента, нарушения условий или прав третьих лиц / закона (включая условия Mojang/Microsoft).",
    ],
  },
  termination: {
    title: "Прекращение",
    paragraphs: [
      "Вы можете прекратить использование в любое время. Мы можем приостановить или прекратить доступ немедленно без уведомления, в том числе при нарушении условий.",
      "После прекращения право использования Сервиса заканчивается. Положения об ИП, отказах от гарантий, ограничении ответственности и возмещении продолжают действовать.",
    ],
  },
  law: {
    title: "Применимое право и споры",
    paragraphs: [
      "Условия регулируются правом страны обычного места жительства мейнтейнера Owyx Team, если иное не предусмотрено императивными нормами защиты потребителей Вашей страны.",
      "Сначала постарайтесь решить спор через указанные контакты. При допустимости споры рассматриваются судом по месту жительства мейнтейнера с учётом обязательной подсудности для потребителей.",
      "Недействительность отдельного положения не влияет на остальные.",
    ],
  },
  entire: {
    title: "Полнота соглашения",
    paragraphs: [
      "Настоящие условия, Политика конфиденциальности и EULA лаунчера составляют полное соглашение между Вами и Owyx Team относительно Сервиса.",
      "Неприменение нами какого-либо положения не означает отказ от права. Уступка Вами прав без нашего согласия недействительна; мы можем уступить права при передаче проекта.",
    ],
  },
  contact: {
    title: "Контакты",
    paragraphs: [
      "Вопросы по условиям — через каналы, указанные на owyx.site (например Discord). Персональные юридические реквизиты на этой странице не публикуются.",
    ],
  },
};

const RU_PRIVACY = {
  scope: {
    title: "Область применения",
    paragraphs: [
      "Политика применяется к данным на сайте, в переписке с нами, в десктоп-лаунчере и через API Сервиса.",
      "Она не применяется к сторонним сайтам, серверам Minecraft, Discord, Microsoft, Mojang, Modrinth и иным сервисам вне Owyx.",
    ],
  },
  foreword: {
    title: "Правовой контекст",
    paragraphs: [
      "Документ учитывает ожидания прозрачности, в том числе близкие к GDPR и аналогичным законам, в той мере, в какой они применимы. У небольшого / соло-оператора формальности крупной компании могут отличаться; мы всё равно стремимся уважать Ваши права.",
    ],
  },
  controller: {
    title: "Оператор данных",
    paragraphs: [
      "Оператор персональных данных — Owyx Team как оператор owyx.site / api.owyx.site. Контакты публикуются на сайте. Домашний адрес и паспортные данные здесь не указываются.",
    ],
  },
  collect: {
    title: "Какие данные мы собираем",
    paragraphs: [
      "Данные аккаунта: email, хеш пароля, ник, роли/флаги доверия, опциональная привязка Discord, аватар/скин, статус подтверждения email, даты регистрации.",
      "Сессии: токены / хеши сессий, технические метаданные клиента, IP в логах, метки времени.",
      "Лаунчер / API: запросы каталога, косметики, друзей, presence (онлайн/в игре и имя инстанса), действия админов.",
      "Технические логи: журналы сервера, ошибки, антифрод/рейтлимиты, метаданные CDN/WAF при использовании.",
      "Обращения в поддержку.",
      "Мы намеренно не собираем особые категории данных. Не указывайте чувствительные сведения в никах и скинах.",
    ],
  },
  use: {
    title: "Цели обработки",
    paragraphs: [
      "Создание и защита аккаунтов; аутентификация; косметика, друзья, каталог; предотвращение злоупотреблений; отладка и улучшение Сервиса; сообщения о безопасности; исполнение закона.",
      "Мы не продаём персональные данные и не строим бизнес на рекламных сетях третьих лиц.",
    ],
  },
  legal_bases: {
    title: "Правовые основания (где применимо)",
    paragraphs: [
      "Исполнение договора / преддоговорные действия.",
      "Законный интерес: безопасность, антифрод, надёжность.",
      "Согласие — где мы его запрашиваем.",
      "Юридическая обязанность — когда закон требует хранения или раскрытия.",
    ],
  },
  sharing: {
    title: "Передача и обработчики",
    paragraphs: [
      "Инфраструктура (хостинг, БД, хранилище, почта, CDN/WAF, капча, Discord). Microsoft/Mojang — только если Вы выбираете вход Microsoft.",
      "Раскрытие возможно по закону, для защиты прав и безопасности или при добросовестной передаче проекта.",
    ],
  },
  retention: {
    title: "Хранение",
    paragraphs: [
      "Данные аккаунта — пока аккаунт существует и разумный срок после для безопасности и споров.",
      "Логи — ограниченный операционный срок, дольше — при расследованиях.",
      "Удаление — по запросу с учётом законных исключений.",
    ],
  },
  security: {
    title: "Безопасность",
    paragraphs: [
      "Разумные меры: хеширование паролей, HTTPS, контроль сессий, ограниченный админ-доступ. Абсолютной безопасности не существует.",
    ],
  },
  transfers: {
    title: "Трансграничная передача",
    paragraphs: [
      "Серверы и подрядчики могут находиться в разных странах. При необходимости используются применимые механизмы защиты.",
    ],
  },
  cookies: {
    title: "Cookies и local storage",
    paragraphs: [
      "Используются для сессий, языка и функциональных нужд. Очистка данных сайта может отключить часть функций.",
    ],
  },
  children: {
    title: "Дети",
    paragraphs: [
      "Сервис не предназначен для детей младше 13 лет. Если данные ребёнка попали к нам — сообщите, и мы удалим их.",
    ],
  },
  rights: {
    title: "Ваши права",
    paragraphs: [
      "В зависимости от юрисдикции: доступ, исправление, удаление, ограничение, переносимость, возражение, отзыв согласия.",
      "Для реализации прав свяжитесь с нами через опубликованные каналы и подтвердите личность. Ответ — в разумный срок (обычно до 30 дней, где требуется).",
      "Вы можете обратиться в надзорный орган, если это применимо.",
    ],
  },
  changes_p: {
    title: "Изменения",
    paragraphs: [
      "Мы можем обновить Политику, опубликовав новую версию и дату. Продолжение использования означает ознакомление с обновлением.",
    ],
  },
  contact_p: {
    title: "Контакты",
    paragraphs: [
      "Вопросы о данных — через каналы на owyx.site. Почтовые реквизиты физлица здесь не публикуются.",
    ],
  },
};

const RU_EULA = {
  license_grant: {
    title: "Предоставление лицензии",
    paragraphs: [
      "При соблюдении условий Owyx Team предоставляет Вам личную, неисключительную, непередаваемую, отзывную лицензию на установку и использование ПО для доступа к Сервису и игры в поддерживаемых конфигурациях.",
      "Open-source компоненты также лицензируются по указанным лицензиям (включая GNU GPL v3). Права GPL сохраняются.",
    ],
  },
  brand: {
    title: "Ограничения по бренду",
    paragraphs: [
      "Лицензия не даёт прав на имя Owyx, логотипы и дизайн-систему. Форки и сборки не должны использовать брендинг Owyx. См. COPYING.md и каталог brand/.",
    ],
  },
  restrictions: {
    title: "Ограничения",
    paragraphs: [
      "Кроме случаев, разрешённых GPL-3 или законом, запрещается: выдавать ПО за официальный клиент Mojang/Microsoft; нарушать условия Minecraft/Microsoft; обходить контроль доступа Owyx; внедрять вредоносный код; удалять обязательные уведомления об авторских правах.",
    ],
  },
  updates: {
    title: "Обновления и сетевые обращения",
    paragraphs: [
      "ПО может проверять обновления, аутентифицировать сессии, синхронизировать косметику, отправлять presence друзей и загружать каталог/новости с серверов Owyx.",
    ],
  },
  no_game_license: {
    title: "Нет лицензии на Minecraft",
    paragraphs: [
      "ПО не включает право собственности на Minecraft. Получение и использование игры — по условиям Mojang/Microsoft. Ник / offline-mode не равны купленной лицензии Minecraft.",
    ],
  },
  warranty_eula: {
    title: "Отказ от гарантий и ответственность",
    paragraphs: [
      "ПО ПРЕДОСТАВЛЯЕТСЯ «КАК ЕСТЬ». ОГРАНИЧЕНИЯ ОТВЕТСТВЕННОСТИ ИЗ ПОЛЬЗОВАТЕЛЬСКОГО СОГЛАШЕНИЯ ПРИМЕНЯЮТСЯ К ПО.",
    ],
  },
  termination_eula: {
    title: "Прекращение",
    paragraphs: [
      "EULA прекращается при нарушении или при прекращении распространения ПО. Права GPL на соответствующий исходный код сохраняются согласно GPL-3.",
    ],
  },
  contact_eula: {
    title: "Контакты",
    paragraphs: ["Вопросы — через каналы на owyx.site."],
  },
};

fs.mkdirSync(outDir, { recursive: true });
const en = buildEn();
const ru = buildRu();
fs.writeFileSync(path.join(outDir, "en_US.json"), JSON.stringify(en, null, 2) + "\n");
fs.writeFileSync(path.join(outDir, "ru_RU.json"), JSON.stringify(ru, null, 2) + "\n");
console.log("wrote", outDir);
