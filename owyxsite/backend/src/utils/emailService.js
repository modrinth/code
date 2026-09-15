const nodemailer = require('nodemailer');
const db = require('../database/connection');
const { render: renderTemplate } = require('./emailTemplates');

const createTransporter = () => {
  const port = parseInt(process.env.SMTP_PORT || process.env.EMAIL_SMTP_PORT || '465', 10);
  const secure =
    process.env.SMTP_SECURE === 'true' ||
    port === 465;

  return nodemailer.createTransport({
    host: process.env.SMTP_HOST || process.env.EMAIL_SMTP_HOST || 'smtp.yandex.ru',
    port,
    secure,
    auth: {
      user: process.env.SMTP_USER || process.env.EMAIL_SMTP_USER || '',
      pass: process.env.SMTP_PASS || process.env.SMTP_PASSWORD || process.env.EMAIL_SMTP_PASSWORD || '',
    },
  });
};

/** Rough HTML → plain text for multipart/alternative (helps spam filters). */
const htmlToText = (html) =>
  String(html || '')
    .replace(/<style[\s\S]*?<\/style>/gi, '')
    .replace(/<script[\s\S]*?<\/script>/gi, '')
    .replace(/<br\s*\/?>/gi, '\n')
    .replace(/<\/p>/gi, '\n\n')
    .replace(/<\/tr>/gi, '\n')
    .replace(/<a\s+[^>]*href=["']([^"']+)["'][^>]*>([\s\S]*?)<\/a>/gi, '$2 ($1)')
    .replace(/<[^>]+>/g, '')
    .replace(/&nbsp;/g, ' ')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&#39;/g, "'")
    .replace(/&quot;/g, '"')
    .replace(/[ \t]+\n/g, '\n')
    .replace(/\n{3,}/g, '\n\n')
    .trim();

const getEmailTemplate = async (id) => {
  const result = await db.query(
    `SELECT template_subject, template_html, template_variables FROM email_templates WHERE id = $1`,
    [id]
  );

  if (result.rows.length === 0) {
    throw new Error(`Email template with ID '${id}' not found`);
  }

  const row = result.rows[0];
  let variables = [];
  if (row.template_variables) {
    try {
      if (String(row.template_variables).trim().startsWith('[')) {
        variables = JSON.parse(row.template_variables);
      } else {
        variables = String(row.template_variables)
          .split(',')
          .map((v) => v.trim());
      }
    } catch {
      variables = [];
    }
  }

  return {
    subject: row.template_subject,
    html: row.template_html,
    variables,
  };
};

const replaceTemplateVariables = (template, variables) => {
  if (!template) return '';
  let result = template;
  for (const [key, value] of Object.entries(variables)) {
    const regex = new RegExp(`{{${key}}}`, 'g');
    result = result.replace(regex, value || '');
  }
  return result;
};

const getServerSettings = async () => {
  try {
    const result = await db.query(`SELECT setting_key, setting_value FROM server_settings`);
    const settings = {};
    for (const row of result.rows) {
      settings[row.setting_key] = row.setting_value;
    }

    return {
      serverName: settings['server-name'] || 'Owyx',
      serverIp: settings['server-ip'] || '45.131.186.146:1488',
      discordInvite: settings['discord-invite'] || 'https://discord.gg/owyx',
      telegramInvite: settings['telegram-invite'] || 'https://t.me/owyx',
      currentDate: new Date().toLocaleDateString('ru-RU', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
      }),
    };
  } catch (error) {
    console.error('Error loading server settings for email:', error.message);
    return {
      serverName: 'Owyx',
      serverIp: '45.131.186.146:1488',
      discordInvite: 'https://discord.gg/owyx',
      telegramInvite: 'https://t.me/owyx',
      currentDate: new Date().toLocaleDateString('ru-RU'),
    };
  }
};

const sendEmail = async (to, id, customVariables = {}) => {
  const template = await getEmailTemplate(id);
  const serverSettings = await getServerSettings();
  const variables = { ...serverSettings, ...customVariables };

  if (!template.subject || !template.html) {
    throw new Error(`Template '${id}' is incomplete`);
  }

  const subject = replaceTemplateVariables(template.subject, variables);
  const html = replaceTemplateVariables(template.html, variables);

  const smtpUser = process.env.SMTP_USER || process.env.EMAIL_SMTP_USER;
  const smtpPass = process.env.SMTP_PASS || process.env.SMTP_PASSWORD || process.env.EMAIL_SMTP_PASSWORD;

  if (!smtpUser || !smtpPass) {
    console.log('SMTP not configured — email simulated');
    console.log(`To: ${to}`);
    console.log(`Subject: ${subject}`);
    return {
      success: true,
      messageId: 'simulated-' + Date.now(),
      simulated: true,
    };
  }

  const transporter = createTransporter();
  const fromAddress = process.env.EMAIL_FROM || smtpUser;
  const siteUrl = process.env.FRONTEND_URL || 'https://owyx.site';

  const info = await transporter.sendMail({
    from: `"${serverSettings.serverName}" <${fromAddress}>`,
    replyTo: process.env.EMAIL_REPLY_TO || fromAddress,
    to,
    subject,
    html,
    text: htmlToText(html),
    headers: {
      'List-Unsubscribe': `<${siteUrl}/login>`,
      'List-Unsubscribe-Post': 'List-Unsubscribe=One-Click',
      'X-Auto-Response-Suppress': 'OOF, AutoReply',
    },
  });

  return {
    success: true,
    messageId: info.messageId,
    simulated: false,
  };
};

// Send one of the built-in Owyx templates (no DB dependency). Simulates in logs
// when SMTP is not configured, exactly like sendEmail().
const sendTemplate = async (to, key, vars = {}) => {
  const serverSettings = await getServerSettings();
  const { subject, html } = renderTemplate(key, { ...serverSettings, ...vars });

  const smtpUser = process.env.SMTP_USER || process.env.EMAIL_SMTP_USER;
  const smtpPass = process.env.SMTP_PASS || process.env.SMTP_PASSWORD || process.env.EMAIL_SMTP_PASSWORD;

  if (!smtpUser || !smtpPass) {
    console.log(`SMTP not configured — email simulated (template: ${key})`);
    console.log(`To: ${to}`);
    console.log(`Subject: ${subject}`);
    return { success: true, messageId: 'simulated-' + Date.now(), simulated: true };
  }

  const transporter = createTransporter();
  const fromAddress = process.env.EMAIL_FROM || smtpUser;
  const siteUrl = process.env.FRONTEND_URL || 'https://owyx.site';
  const info = await transporter.sendMail({
    from: `"${serverSettings.serverName}" <${fromAddress}>`,
    replyTo: process.env.EMAIL_REPLY_TO || fromAddress,
    to,
    subject,
    html,
    text: htmlToText(html),
    headers: {
      'List-Unsubscribe': `<${siteUrl}/login>`,
      'List-Unsubscribe-Post': 'List-Unsubscribe=One-Click',
      'X-Auto-Response-Suppress': 'OOF, AutoReply',
    },
  });
  return { success: true, messageId: info.messageId, simulated: false };
};

const sendVerificationEmail = async (email, nickname, verificationLink) =>
  sendTemplate(email, 'verify', { nickname, verificationLink });

const sendWelcomeEmail = async (email, nickname) =>
  sendTemplate(email, 'welcome', { nickname });

const sendPasswordResetEmail = async (email, nickname, resetLink) =>
  sendTemplate(email, 'reset', { nickname, resetLink });

const sendPasswordChangedEmail = async (email, nickname) =>
  sendTemplate(email, 'passwordChanged', { nickname });

const sendNicknameChangedEmail = async (email, oldNick, newNick) =>
  sendTemplate(email, 'nicknameChanged', { oldNick, newNick });

const sendEmailChangeCode = async (email, code) =>
  sendTemplate(email, 'emailChange', { code });

// Legacy DB-template senders (kept for back-compat with existing admin routes).
const sendApplicationApprovedEmail = async (email, nickname) =>
  sendEmail(email, 9, { nickname });

const sendApplicationRejectedEmail = async (email, nickname, reason) =>
  sendEmail(email, 10, { nickname, rejectionReason: reason || 'Не указана' });

module.exports = {
  sendEmail,
  sendTemplate,
  sendVerificationEmail,
  sendWelcomeEmail,
  sendPasswordResetEmail,
  sendPasswordChangedEmail,
  sendNicknameChangedEmail,
  sendEmailChangeCode,
  sendApplicationApprovedEmail,
  sendApplicationRejectedEmail,
};
