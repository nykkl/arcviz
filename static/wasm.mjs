import init, { Arcviz } from './pkg/arcviz.js'; // default export should be the wasm initialization function

let arcviz = null;

export async function startWebAssembly() {
	await init(); // load wasm; leaving this out might sometimes work, but it's still wrong

	let io = window.io;
	if (!io) {
		console.log("info: no file io provided falling back on 'File System Access API'")
		io = {
			save: async (data, callback) => {
				function call_callback(success) {
					try {
						callback(success);
					} catch {
						console.log('save: discarded'); // callback might be invalid if user is no longer interested
					}
				}
				try {
					let fileHandle = await window.showSaveFilePicker();
					try {
						let stream = await fileHandle.createWritable();
						await stream.write(data.buffer);
						await stream.close();
						call_callback(true)
					} catch (err) {
						console.log('save: error: ' + err);
						call_callback(false);
					}
				} catch {
					console.log('save: canceled');
					call_callback(false);
				}
			},
			open: async (callback) => {
				function call_callback(error, data) {
					try {
						callback(error, data);
					} catch {
						console.log('open: discarded'); // callback might be invalid if user is no longer interested
					}
				}
				try {
					let fileHandles = await window.showOpenFilePicker();
					try {
						let fileHandle = fileHandles[0];
						try {
							let file = await fileHandle.getFile();
							let reader = new FileReader();
							reader.onloadend = () => call_callback(null, new Uint8Array(reader.result));
							reader.readAsArrayBuffer(file);
						} catch (err) {
							console.log('open: error: ' + err);
							call_callback(err, null);
						}
					} catch {
						console.log('open: empty');
						call_callback(null, null);
					}
				} catch {
					console.log('open: canceled');
					call_callback(null, null);
				}
			},
		};
	}

	arcviz = new Arcviz(io.open, io.save);
	arcviz.mount();
}
