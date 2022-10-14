import fs from 'fs';

fs.readFile('something.txt', 'utf-8', (err, contents) => {
	if (err !== null) {
		throw new Error(err.toString());
	} else {
		console.log(contents);
	}
});

