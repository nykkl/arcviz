import { app, BrowserWindow, ipcMain, dialog } from 'electron/main';
import path from 'node:path';
import fsPromises from 'node:fs/promises';


app.on('window-all-closed', () => { app.quit() });  // setup handling for teardown
app.whenReady().then(() => { init() });             // setup initialization


function init () {
	const home = new BrowserWindow({
		width: 800,
		height: 600,

		webPreferences: {
			nodeIntegration: false,
			contextIsolation: true,
			sandbox: true,
			preload: path.join(app.getAppPath(), 'preload.js'), // electron does not support .mjs with sandboxing
		},
	});

	ipcMain.handle('save', async (event, data) => {
		const { canceled, filePath } = await dialog.showSaveDialog({});
		if (canceled) return { canceled: true };
		try {
			await fsPromises.writeFile(filePath, data);
			return {};
		} catch (err) {
			return { error: err };
		}
	});
	ipcMain.handle('open', async (event) => {
		const { canceled, filePaths } = await dialog.showOpenDialog({});
		if (canceled) return { canceled: true };
		if (!filePaths[0]) return { data: null };
		try {
			const file = await fsPromises.readFile(filePaths[0]);
			return { data: file };
		} catch (err) {
			return { error: err };
		}
	});

	home.loadFile('app/index.html'); // if you want to show an existing webpage instead: win.loadURL('https://github.com')
}
