import fs from 'fs';

fs.readFile('something.txt', 'utf8', (err, contents) => {
	if (err !== null) {
		throw new Error(err.toString());
	} else {
		fs.readFile('something.txt', 'utf8', (err, contents2) => {
			if (err !== null) {
				throw new Error(err.toString());
			} else {
				console.log(contents === contents2);
			}
		});
	}
});

