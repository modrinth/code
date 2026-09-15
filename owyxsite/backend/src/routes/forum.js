const express = require('express');
const { body, validationResult } = require('express-validator');
const db = require('../database/connection');
const { authenticateToken, requireRole } = require('./auth');

const router = express.Router();

function slugify(text) {
  return String(text)
    .toLowerCase()
    .trim()
    .replace(/[^\w\s-а-яё]/gi, '')
    .replace(/[\s_]+/g, '-')
    .replace(/-+/g, '-')
    .substring(0, 80) || `topic-${Date.now()}`;
}

// GET /api/forum/categories
router.get('/categories', async (_req, res) => {
  try {
    const result = await db.query(`
      SELECT id, title, slug, description, icon, color, parent_id, position,
             is_locked, topics_count, posts_count, created_at
      FROM forum_categories
      ORDER BY position ASC NULLS LAST, id ASC
    `);
    res.json({ categories: result.rows });
  } catch (error) {
    console.error('Forum categories error:', error);
    res.status(500).json({ error: 'Внутренняя ошибка сервера' });
  }
});

// GET /api/forum/categories/:slug/topics
router.get('/categories/:slug/topics', async (req, res) => {
  try {
    const page = Math.max(1, parseInt(req.query.page, 10) || 1);
    const limit = Math.min(50, parseInt(req.query.limit, 10) || 20);
    const offset = (page - 1) * limit;

    const cat = await db.query(
      `SELECT id, title, slug, description, is_locked FROM forum_categories WHERE slug = $1`,
      [req.params.slug]
    );
    if (cat.rows.length === 0) {
      return res.status(404).json({ error: 'Категория не найдена' });
    }

    const topics = await db.query(
      `
      SELECT t.id, t.title, t.slug, t.is_pinned, t.is_locked, t.is_solved,
             t.views_count, t.posts_count, t.created_at, t.last_post_at, t.tags,
             u.nickname AS author_nickname, u.avatar_url AS author_avatar
      FROM forum_topics t
      LEFT JOIN users u ON u.id = t.author_id
      WHERE t.category_id = $1
      ORDER BY t.is_pinned DESC, COALESCE(t.last_post_at, t.created_at) DESC
      LIMIT $2 OFFSET $3
      `,
      [cat.rows[0].id, limit, offset]
    );

    const count = await db.query(
      `SELECT COUNT(*)::int AS total FROM forum_topics WHERE category_id = $1`,
      [cat.rows[0].id]
    );

    res.json({
      category: cat.rows[0],
      topics: topics.rows,
      pagination: {
        page,
        limit,
        total: count.rows[0].total,
        totalPages: Math.ceil(count.rows[0].total / limit),
      },
    });
  } catch (error) {
    console.error('Forum topics error:', error);
    res.status(500).json({ error: 'Внутренняя ошибка сервера' });
  }
});

// GET /api/forum/topics/:id
router.get('/topics/:id', async (req, res) => {
  try {
    const topicId = parseInt(req.params.id, 10);
    const topic = await db.query(
      `
      SELECT t.*, c.title AS category_title, c.slug AS category_slug,
             u.nickname AS author_nickname, u.avatar_url AS author_avatar
      FROM forum_topics t
      JOIN forum_categories c ON c.id = t.category_id
      LEFT JOIN users u ON u.id = t.author_id
      WHERE t.id = $1
      `,
      [topicId]
    );

    if (topic.rows.length === 0) {
      return res.status(404).json({ error: 'Тема не найдена' });
    }

    await db.query(`UPDATE forum_topics SET views_count = views_count + 1 WHERE id = $1`, [topicId]);

    const posts = await db.query(
      `
      SELECT p.id, p.content, p.parent_post_id, p.created_at, p.updated_at,
             p.upvotes, p.downvotes, p.is_edited, p.is_deleted,
             u.id AS author_id, u.nickname AS author_nickname, u.avatar_url AS author_avatar,
             u.role AS author_role, u.trust_level AS author_trust_level
      FROM forum_posts p
      LEFT JOIN users u ON u.id = p.author_id
      WHERE p.topic_id = $1 AND p.is_deleted = false
      ORDER BY p.created_at ASC
      `,
      [topicId]
    );

    res.json({ topic: topic.rows[0], posts: posts.rows });
  } catch (error) {
    console.error('Forum topic detail error:', error);
    res.status(500).json({ error: 'Внутренняя ошибка сервера' });
  }
});

// POST /api/forum/topics
router.post(
  '/topics',
  authenticateToken,
  body('category_id').isInt(),
  body('title').isLength({ min: 3, max: 200 }),
  body('content').isLength({ min: 10, max: 20000 }),
  async (req, res) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        return res.status(400).json({ error: 'Ошибка валидации', details: errors.array() });
      }

      const { category_id, title, content, tags } = req.body;
      const cat = await db.query(
        `SELECT id, is_locked FROM forum_categories WHERE id = $1`,
        [category_id]
      );
      if (cat.rows.length === 0) {
        return res.status(404).json({ error: 'Категория не найдена' });
      }
      if (cat.rows[0].is_locked && !['admin', 'moderator'].includes(req.user.role)) {
        return res.status(403).json({ error: 'Категория закрыта для новых тем' });
      }

      let slug = slugify(title);
      const slugCheck = await db.query(`SELECT id FROM forum_topics WHERE slug = $1`, [slug]);
      if (slugCheck.rows.length > 0) slug = `${slug}-${Date.now()}`;

      const topic = await db.query(
        `
        INSERT INTO forum_topics (category_id, title, slug, content, author_id, tags)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        `,
        [category_id, title, slug, content, req.user.id, tags ? JSON.stringify(tags) : null]
      );

      await db.query(
        `
        INSERT INTO forum_posts (topic_id, author_id, content)
        VALUES ($1, $2, $3)
        `,
        [topic.rows[0].id, req.user.id, content]
      );

      res.status(201).json({ success: true, topic: topic.rows[0] });
    } catch (error) {
      console.error('Create topic error:', error);
      res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
  }
);

// POST /api/forum/topics/:id/posts
router.post(
  '/topics/:id/posts',
  authenticateToken,
  body('content').isLength({ min: 1, max: 20000 }),
  async (req, res) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        return res.status(400).json({ error: 'Ошибка валидации', details: errors.array() });
      }

      const topicId = parseInt(req.params.id, 10);
      const topic = await db.query(`SELECT id, is_locked FROM forum_topics WHERE id = $1`, [topicId]);
      if (topic.rows.length === 0) {
        return res.status(404).json({ error: 'Тема не найдена' });
      }
      if (topic.rows[0].is_locked && !['admin', 'moderator'].includes(req.user.role)) {
        return res.status(403).json({ error: 'Тема закрыта' });
      }

      const post = await db.query(
        `
        INSERT INTO forum_posts (topic_id, author_id, content, parent_post_id)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        `,
        [topicId, req.user.id, req.body.content, req.body.parent_post_id || null]
      );

      res.status(201).json({ success: true, post: post.rows[0] });
    } catch (error) {
      console.error('Create post error:', error);
      res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
  }
);

// DELETE /api/forum/topics/:id (moderation)
router.delete('/topics/:id', authenticateToken, requireRole(['admin', 'moderator']), async (req, res) => {
  try {
    await db.query(`DELETE FROM forum_topics WHERE id = $1`, [req.params.id]);
    await db.query(
      `INSERT INTO admin_logs (admin_id, action, details) VALUES ($1, $2, $3)`,
      [req.user.id, 'forum_moderate_delete', `Topic ID: ${req.params.id}`]
    );
    res.json({ success: true });
  } catch (error) {
    console.error('Delete topic error:', error);
    res.status(500).json({ error: 'Внутренняя ошибка сервера' });
  }
});

module.exports = router;
