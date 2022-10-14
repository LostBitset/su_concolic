// begin declarations for instrumentation blocksonly

let __hasblock = true;

function __claimblock(f, args) {
	let __hadblock = __hasblock;
	if (!__hadblock) {
		__sendNextCallback(f, args);
	}
	process.nextTick(() => {
		__hasblock = false;
	});
}

function __sendNextCallback(f, args) {
	console.log('GOT NEXT CALLBACK:', f, 'with arguments', args);
}

// end declarations for instrumentation blocksonly

// begin instrumented code

__claimblock('__top__', []);

import fs from 'fs';

fs.readFile('something.txt', 'utf8', (() => {
	let __func_0 = ((...__args) => {
		__claimblock(__func_0, __args);
		return ((err, contents) => {
			if (err !== null) {
				throw new Error(err.toString());
			} else {
				fs.readFile('something.txt', 'utf8', (() => {
					let __func_1 = ((...__args) => {
						__claimblock(__func_1, __args);
						return ((err, contents2) => {
							if (err !== null) {
								throw new Error(err.toString());
							} else {
								console.log(contents === contents2);
							}
						})(...__args);
					});
					return __func_1;
				})());
			}
		})(...__args);
	});
	return __func_0;
})());

