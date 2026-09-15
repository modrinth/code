const { Pool } = require('pg');
require('dotenv').config();

const getDbHost = () => process.env.DB_HOST || '127.0.0.1';

const pool = new Pool({
  host: getDbHost(),
  port: parseInt(process.env.DB_PORT || '5432', 10),
  database: process.env.DB_NAME || 'owyx_db',
  user: process.env.DB_USER || 'owyx_user',
  password: process.env.DB_PASSWORD || process.env.DB_PASS || '',
  max: 20,
  idleTimeoutMillis: 60000,
  connectionTimeoutMillis: 30000,
  keepAlive: true,
  keepAliveInitialDelayMillis: 0,
  ssl: process.env.DB_SSL === 'true' ? { rejectUnauthorized: false } : false,
});

pool.on('error', (err) => {
  console.error('Unexpected PostgreSQL pool error:', err.message);
});

async function query(text, params, retries = 3) {
  if (process.env.NO_DATABASE === 'true') {
    console.log('NO_DATABASE mode — query skipped:', String(text).substring(0, 60));
    return { rows: [], rowCount: 0 };
  }

  const start = Date.now();

  for (let attempt = 1; attempt <= retries; attempt++) {
    try {
      const res = await pool.query(text, params);
      const duration = Date.now() - start;
      if (process.env.LOG_QUERIES === 'true') {
        console.log('Query OK', { attempt, duration: `${duration}ms`, rows: res.rowCount });
      }
      return res;
    } catch (error) {
      console.error(`Query attempt ${attempt}/${retries} failed:`, {
        error: error.message,
        code: error.code,
        host: getDbHost(),
      });

      if (attempt === retries) throw error;

      const delay = Math.min(1000 * Math.pow(2, attempt - 1), 10000);
      await new Promise((resolve) => setTimeout(resolve, delay));
    }
  }
}

async function getClient() {
  return pool.connect();
}

async function testConnection() {
  let client;
  try {
    console.log(`Connecting to PostgreSQL ${getDbHost()}:${process.env.DB_PORT || 5432}/${process.env.DB_NAME || 'owyx_db'}`);
    client = await pool.connect();
    const result = await client.query('SELECT NOW() as current_time, version() as version');
    console.log('Database connected:', result.rows[0].current_time);
    return true;
  } catch (error) {
    console.error('Database connection failed:', error.message);
    return false;
  } finally {
    if (client) client.release();
  }
}

async function end() {
  await pool.end();
}

module.exports = {
  query,
  getClient,
  testConnection,
  end,
  pool,
};
