// Applications / whitelist gate retired — registered accounts use the launcher.
const express = require('express');
const router = express.Router();

router.all('*', (_req, res) => {
    res.status(410).json({
        error: 'gone',
        message: 'Applications API is retired. Create an Owyx account and use the launcher.'
    });
});

module.exports = router;
