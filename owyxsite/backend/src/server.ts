import http from 'http';
import path from 'path';
import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import morgan from 'morgan';
import compression from 'compression';
import dotenv from 'dotenv';

dotenv.config({ path: path.resolve(__dirname, '../../.env') });
dotenv.config();

const db = require('./database/connection');
const authRoutes = require('./routes/auth');
const applicationRoutes = require('./routes/applications');
const profileRoutes = require('./routes/profile');
const adminRoutes = require('./routes/admin');
const reputationRoutes = require('./routes/reputation');
const trustLevelRoutes = require('./routes/trust-level');
const settingsRoutes = require('./routes/settings');
const forumRoutes = require('./routes/forum');
const chatRoutes = require('./routes/chat');
const launcherRoutes = require('./routes/launcher');
const newsRoutes = require('./routes/news');
const catalogRoutes = require('./routes/catalog');
const { clientKeyGate } = require('./middleware/clientKey');
const { initSocket } = require('./socket');

const app = express();
const PORT = parseInt(process.env.API_PORT || process.env.PORT || '3001', 10);

const trustProxyHops = Number.parseInt(process.env.TRUST_PROXY_HOPS || '0', 10);
app.set('trust proxy', Number.isSafeInteger(trustProxyHops) && trustProxyHops > 0 ? trustProxyHops : false);

app.use((req: any, _res, next) => {
  // Express applies the configured trusted-proxy hop count to req.ip. Never
  // parse X-Forwarded-For directly: a public client could forge rate-limit IPs.
  req.clientIp = req.ip || req.socket?.remoteAddress;

  if (req.clientIp && req.clientIp.startsWith('::ffff:')) {
    req.clientIp = req.clientIp.substring(7);
  }
  next();
});

app.use(
  helmet({
    crossOriginResourcePolicy: { policy: 'cross-origin' },
    contentSecurityPolicy: false,
  })
);

/** Tauri 2 webview Origins — browser fetch to api.owyx.site needs these or login is "Failed to fetch". */
const TAURI_CORS_ORIGINS = [
  'https://tauri.localhost',
  'http://tauri.localhost',
  'tauri://localhost',
  'http://localhost:1420',
  'http://127.0.0.1:1420',
];

const envCors = process.env.CORS_ORIGIN
  ? process.env.CORS_ORIGIN.split(',').map((s) => s.trim()).filter(Boolean)
  : [];

const corsOrigin = [
  ...new Set([
    ...(envCors.length
      ? envCors
      : process.env.NODE_ENV === 'production'
        ? [process.env.FRONTEND_URL || 'https://owyx.site', 'https://www.owyx.site']
        : [
            process.env.FRONTEND_URL || 'http://localhost:3000',
            'http://127.0.0.1:3000',
          ]),
    ...TAURI_CORS_ORIGINS,
  ]),
];

app.use(
  cors({
    origin: corsOrigin,
    credentials: true,
  })
);

app.use(compression());
app.use(morgan(process.env.NODE_ENV === 'production' ? 'combined' : 'dev'));
app.use(express.json({ limit: '1mb' }));
app.use(express.urlencoded({ extended: true, limit: '1mb' }));

// api.owyx.site requires X-Owyx-Client-Key; owyx.site /api proxy does not.
app.use(clientKeyGate);

app.use(
  '/uploads',
  express.static(path.join(__dirname, '../uploads'), {
    maxAge: process.env.NODE_ENV === 'production' ? '7d' : 0,
  })
);
app.use(
  '/fixtures',
  express.static(path.join(__dirname, '../fixtures'), {
    maxAge: process.env.NODE_ENV === 'production' ? '1h' : 0,
  })
);

app.use('/api/auth', authRoutes);
app.use('/api/applications', applicationRoutes);
app.use('/api/profile', profileRoutes);
app.use('/api/friends', require('./routes/friends'));
app.use('/api/admin', adminRoutes);
app.use('/api', settingsRoutes);
app.use('/api/reputation', reputationRoutes);
app.use('/api/trust-level', trustLevelRoutes);
app.use('/api/forum', forumRoutes);
app.use('/api/chat', chatRoutes);
app.use('/api/launcher', launcherRoutes);
app.use('/api/news', newsRoutes);
app.use('/api/admin/news', newsRoutes.adminRouter);
app.use('/api/admin/packs', catalogRoutes.packsAdmin);
app.use('/api/admin/servers', catalogRoutes.serversAdmin);

app.get('/health', async (_req, res) => {
  try {
    await db.query('SELECT 1');
    res.json({
      status: 'healthy',
      timestamp: new Date().toISOString(),
      version: '1.0.0',
      services: {
        database: 'connected',
        server: 'running',
        socketio: 'enabled',
      },
    });
  } catch (error) {
    console.error('Health check failed:', error);
    res.status(503).json({
      status: 'unhealthy',
      timestamp: new Date().toISOString(),
      error: 'Database connection failed',
    });
  }
});

app.get('/api', (_req, res) => {
  res.json({
    name: 'Owyx API',
    version: '1.0.0',
    description: 'API for Owyx Minecraft site (migrated from chiwawasite)',
    endpoints: {
      auth: '/api/auth',
      applications: '/api/applications',
      profile: '/api/profile',
      admin: '/api/admin',
      forum: '/api/forum',
      chat: '/api/chat',
      settings: '/api/settings',
      launcher: '/api/launcher',
      adminServers: '/api/admin/servers',
      adminPacks: '/api/admin/packs',
    },
  });
});

app.use('/api', (req, res) => {
  res.status(404).json({
    error: 'API endpoint не найден',
    path: req.path,
    method: req.method,
  });
});

app.use((error: any, _req: express.Request, res: express.Response, _next: express.NextFunction) => {
  console.error('Server error:', error);
  const status = Number.isInteger(error.status) ? error.status : 500;
  res.status(status).json({
    error: status >= 500 ? 'Внутренняя ошибка сервера' : error.message,
  });
});

const httpServer = http.createServer(app);
initSocket(httpServer);

setInterval(async () => {
  try {
    const webResult = await db.query('DELETE FROM user_sessions WHERE expires_at < NOW()');
    const deactivateResult = await db.query(`
      UPDATE user_sessions
      SET is_active = false
      WHERE is_active = true
      AND last_activity < NOW() - INTERVAL '1 day'
    `);
    if (webResult.rowCount + deactivateResult.rowCount > 0) {
      console.log(
        `Cleaned sessions: web=${webResult.rowCount}, deactivated=${deactivateResult.rowCount}`
      );
    }
  } catch (error) {
    console.error('Session cleanup error:', error);
  }
}, 60 * 60 * 1000);

async function ensureChatSchema() {
  try {
    await db.query(`
      CREATE TABLE IF NOT EXISTS chat_rooms (
        id SERIAL PRIMARY KEY,
        name VARCHAR(100) NOT NULL,
        slug VARCHAR(100) NOT NULL UNIQUE,
        description TEXT,
        is_private BOOLEAN DEFAULT false,
        created_by INTEGER,
        created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP
      )
    `);
    await db.query(`
      CREATE TABLE IF NOT EXISTS chat_room_members (
        room_id INTEGER NOT NULL REFERENCES chat_rooms(id) ON DELETE CASCADE,
        user_id INTEGER NOT NULL,
        joined_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (room_id, user_id)
      )
    `);
    await db.query(`
      CREATE TABLE IF NOT EXISTS chat_messages (
        id SERIAL PRIMARY KEY,
        room_id INTEGER NOT NULL REFERENCES chat_rooms(id) ON DELETE CASCADE,
        user_id INTEGER NOT NULL,
        content TEXT NOT NULL,
        is_deleted BOOLEAN DEFAULT false,
        created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP
      )
    `);
    await db.query(`
      INSERT INTO chat_rooms (name, slug, description, is_private)
      VALUES ('Общий чат', 'general', 'Публичный чат сервера Owyx', false)
      ON CONFLICT (slug) DO NOTHING
    `);
    console.log('Chat schema ready');
  } catch (error: any) {
    console.warn('Chat schema ensure skipped:', error.message);
  }
}

const shutdown = async (signal: string) => {
  console.log(`\nReceived ${signal}, shutting down...`);
  try {
    await db.end();
  } catch (error) {
    console.error('Error closing DB pool:', error);
  }
  process.exit(0);
};

process.on('SIGINT', () => shutdown('SIGINT'));
process.on('SIGTERM', () => shutdown('SIGTERM'));

httpServer.listen(PORT, async () => {
  console.log(`Owyx API listening on :${PORT}`);
  console.log(`Health: http://127.0.0.1:${PORT}/health`);
  console.log(`Socket.io path: /socket.io`);

  // Ensure upload dirs exist (volume may be empty). Permission fix is in entrypoint.
  try {
    const fs = await import('fs/promises');
    const path = await import('path');
    const uploads = path.join(__dirname, '../uploads');
    for (const sub of ['avatars', 'skins', 'packs']) {
      await fs.mkdir(path.join(uploads, sub), { recursive: true });
    }
  } catch (e: any) {
    console.warn('Upload dirs ensure skipped:', e?.message || e);
  }

  const ok = await db.testConnection();
  if (ok) {
    await ensureChatSchema();
    try {
      await catalogRoutes.ensureCatalogSchema();
      console.log('Catalog schema ready');
    } catch (error: any) {
      console.warn('Catalog schema ensure skipped:', error.message);
    }
    try {
      const friendsRoutes = require('./routes/friends');
      if (typeof friendsRoutes.ensureFriendsSchema === 'function') {
        await friendsRoutes.ensureFriendsSchema();
        console.log('Friends schema ready');
      }
    } catch (error: any) {
      console.warn('Friends schema ensure skipped:', error.message);
    }
    try {
      await db.query('DELETE FROM user_sessions WHERE expires_at < NOW()');
    } catch {
      /* ignore */
    }
  } else {
    console.warn('Started without a working database connection');
  }
});

export default app;
