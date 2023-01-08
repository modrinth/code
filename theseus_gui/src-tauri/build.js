// Adds a DMG background image to the finalized MacOS DMG file.
const path = require('path');
const spawn = require('child_process');

process.env.BACKGROUND_FILE = path.join(__dirname, './dmg-bg.png');
process.env.BACKGROUND_FILE_NAME = path.basename(process.env.BACKGROUND_FILE);
process.env.BACKGROUND_CLAUSE = `set background picture of opts to file ".background:${process.env.BACKGROUND_FILE_NAME}"`;

const child = spawn('pnpm', ['exec', 'tauri', 'build']);
child.stdout.on('data', (data) => console.log(data.toString()));
child.stderr.on('data', (data) => console.error(data.toString()));
child.on('exit', (code) => {
	if (code !== 0) console.log(`Child exited with code ${code}`);
});