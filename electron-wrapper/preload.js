const { contextBridge, ipcRenderer } = require('electron/renderer');

contextBridge.exposeInMainWorld( // provide file io in main application
	'io',
	{
		save: (data, callback) => {
			ipcRenderer.invoke('save', data).then(result => {
				let success = false;
				if (result.canceled) {
					console.log('save: canceled');
				} else if (result.error) {
					console.log('save: error: ' + result.error);
				} else {
					success = true;
					console.log('save: ok');
				}
				try {
					callback(success);
				} catch {
					console.log('save: discarded'); // callback might be invalid if user is no longer interested
				}
			});
		},
		open: (callback) => {
			ipcRenderer.invoke('open').then(result => {
				let error = null;
				let data = null;
				if (result.canceled) {
					console.log('open: canceled');
				} else if (result.error) {
					error = result.error;
					console.log('open: error: ' + result.error);
				} else if (result.data) {
					data = result.data;
					console.log('open: ok');
				} else {
					console.log('open: empty');
				}
				try {
					callback(error, data);
				} catch {
					console.log('open: discarded'); // callback might be invalid if user is no longer interested
				}
			});
		},
	}
);
