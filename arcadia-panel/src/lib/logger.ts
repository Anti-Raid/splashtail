// From [PRIVATE REPO] https://github.com/InfinityBotList/Infinity-Next/blob/master/src/utils/ui/logger.ts
const log = (...args: any[]) => {
	// @ts-ignore
	console[args[0]](
		`%c[${Date.now()}]%c[${args[1]}]%c`,
		'color:red;font-weight:bold;',
		'color:purple;font-weight:bold;',
		'',
		...args.slice(2)
	);
};

// Custom logger
const debug = (...args: any[]) => log('debug', ...args);
const info = (...args: any[]) => log('info', ...args);
const warn = (...args: any[]) => log('warn', ...args);
const error = (...args: any[]) => log('error', ...args);

/**
 * PROPER METHOD FOR HANDLING CONST EXPORTS
 */
const exportedObject = {
	debug,
	info,
	warn,
	error
};

export default exportedObject;
