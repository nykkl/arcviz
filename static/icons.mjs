export async function loadIcons() {
	const file = await fetch('heroicons/icons.html');
	const text = await file.text();
	const iconCollection = document.createElement('div');
	iconCollection.id = 'icon-templates';
	iconCollection.innerHTML = text;
	iconCollection.hidden = true;
	document.body.append(iconCollection);
}
