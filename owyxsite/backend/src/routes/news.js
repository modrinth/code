const express = require('express');
const db = require('../database/connection');
const { authenticateToken, requireRole } = require('./auth');

// News/updates. Public read of published items; admin CRUD.
// Mounted: app.use('/api/news', router)  and  app.use('/api/admin/news', adminRouter)

const router = express.Router();
const adminRouter = express.Router();

function mapRow(r) {
  return {
    id: r.id,
    title: r.title,
    tag: r.tag,
    summary: r.summary,
    published: r.published,
    created_at: r.created_at,
    updated_at: r.updated_at,
  };
}

// GET /api/news — public list of published items (newest first).
router.get('/', async (req, res) => {
  try {
    const limit = Math.min(parseInt(req.query.limit, 10) || 12, 50);
    const result = await db.query(
      `SELECT id, title, tag, summary, published, created_at, updated_at
       FROM news WHERE published = true
       ORDER BY created_at DESC LIMIT $1`,
      [limit]
    );
    res.json({ news: result.rows.map(mapRow) });
  } catch (error) {
    console.error('news list error:', error);
    res.status(500).json({ error: 'Не удалось загрузить новости' });
  }
});

// --- Admin CRUD (/api/admin/news) ---
adminRouter.use(authenticateToken, requireRole(['admin', 'moderator']));
adminRouter.use((req, res, next) => {
  if (req.method === 'GET' || req.method === 'HEAD' || req.method === 'OPTIONS') return next();
  return requireRole(['admin'])(req, res, next);
});

// GET /api/admin/news — all items incl. unpublished.
adminRouter.get('/', async (_req, res) => {
  try {
    const result = await db.query(
      `SELECT id, title, tag, summary, published, created_at, updated_at
       FROM news ORDER BY created_at DESC LIMIT 100`
    );
    res.json({ news: result.rows.map(mapRow) });
  } catch (error) {
    console.error('admin news list error:', error);
    res.status(500).json({ error: 'Не удалось загрузить новости' });
  }
});

// POST /api/admin/news — create.
adminRouter.post('/', async (req, res) => {
  try {
    const title = String(req.body.title || '').trim();
    const tag = String(req.body.tag || 'Новость').trim().slice(0, 40) || 'Новость';
    const summary = String(req.body.summary || '').trim();
    const published = req.body.published !== false;
    if (title.length < 2 || title.length > 200) {
      return res.status(400).json({ error: 'Заголовок 2–200 символов' });
    }
    const result = await db.query(
      `INSERT INTO news (title, tag, summary, published, author_id)
       VALUES ($1,$2,$3,$4,$5) RETURNING *`,
      [title, tag, summary, published, req.user.id]
    );
    res.status(201).json({ success: true, news: mapRow(result.rows[0]) });
  } catch (error) {
    console.error('admin news create error:', error);
    res.status(500).json({ error: 'Не удалось создать новость' });
  }
});

// PUT /api/admin/news/:id — update.
adminRouter.put('/:id', async (req, res) => {
  try {
    const fields = [];
    const values = [];
    let i = 1;
    if (req.body.title !== undefined) { fields.push(`title = $${i++}`); values.push(String(req.body.title).trim().slice(0, 200)); }
    if (req.body.tag !== undefined) { fields.push(`tag = $${i++}`); values.push(String(req.body.tag).trim().slice(0, 40) || 'Новость'); }
    if (req.body.summary !== undefined) { fields.push(`summary = $${i++}`); values.push(String(req.body.summary)); }
    if (req.body.published !== undefined) { fields.push(`published = $${i++}`); values.push(Boolean(req.body.published)); }
    if (fields.length === 0) return res.status(400).json({ error: 'Нечего обновлять' });
    fields.push(`updated_at = NOW()`);
    values.push(req.params.id);
    const result = await db.query(
      `UPDATE news SET ${fields.join(', ')} WHERE id = $${i} RETURNING *`,
      values
    );
    if (result.rows.length === 0) return res.status(404).json({ error: 'Новость не найдена' });
    res.json({ success: true, news: mapRow(result.rows[0]) });
  } catch (error) {
    console.error('admin news update error:', error);
    res.status(500).json({ error: 'Не удалось обновить новость' });
  }
});

// DELETE /api/admin/news/:id
adminRouter.delete('/:id', async (req, res) => {
  try {
    const result = await db.query('DELETE FROM news WHERE id = $1 RETURNING id', [req.params.id]);
    if (result.rows.length === 0) return res.status(404).json({ error: 'Новость не найдена' });
    res.json({ success: true });
  } catch (error) {
    console.error('admin news delete error:', error);
    res.status(500).json({ error: 'Не удалось удалить новость' });
  }
});

module.exports = router;
module.exports.adminRouter = adminRouter;
