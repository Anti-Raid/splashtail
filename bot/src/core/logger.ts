// Packages
import "colors";

export class Logger {
	context: string
	constructor(context: string) {
		this.context = context;
	}

	info(name: string, ...message: any) {
		console.log(`${this.context.blue} ${"[INFO]".red} [${name.green}] =>`, ...message);
	}

	debug(name: string, ...message: any) {
		console.log(`${this.context.blue} ${"[DEBUG]".green} [${name.green}] =>`, ...message);
	}

	warn(name: string, ...message: any) {
		console.log(`${this.context.blue} ${"[WARN]".yellow} [${name.green}] =>`, ...message);
	}

	error(name: string, ...message: any) {
		console.log(`${this.context.blue} ${"[ERROR]".red} [${name.green}] =>`, ...message);
	}

	success(name: string, ...message: any) {
		console.log(`${"[SUCCESS]".green} [${name.green}] =>`, ...message);
	}
}

