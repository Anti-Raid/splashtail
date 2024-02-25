export function utf8ToHex(str: string) {
	return Array.from(str)
		.map((c) =>
			c.charCodeAt(0) < 128
				? c.charCodeAt(0).toString(16)
				: encodeURIComponent(c).replace(/\%/g, '').toLowerCase()
		)
		.join('');
}

export function hexToUtf8(hex: string) {
	return decodeURIComponent('%' + hex.match(/.{1,2}/g)?.join('%'));
}

export function title(str: string) {
	return str.replace(/(^|\s)\S/g, function (t) {
		return t.toUpperCase();
	});
}
