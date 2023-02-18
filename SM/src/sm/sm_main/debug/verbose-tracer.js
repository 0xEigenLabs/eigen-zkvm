const chalk = require("chalk");


class VerboseTracer {

    constructor(enable) {
        this.enable = enable == true;
    }

    printOpcode(message) {
        if (this.enable !== true) return;

        let info = `${chalk.magenta("OPCODE".padEnd(7))} | `;
        info += `${message}`;
        console.log(info);
    }

    printTx(message) {
        if (this.enable !== true) return;

        let info = `${chalk.yellowBright("TX".padEnd(7))} | `;
        info += `${message}`;
        console.log(info);
    }

    printBatch(message) {
        if (this.enable !== true) return;

        let info = `${chalk.blue("BATCH".padEnd(7))} | `;
        info += `${message}`;
        console.log(info);
    }

    printError(message) {
        if (this.enable !== true) return;

        let info = `${chalk.red("ERROR".padEnd(7))} | `;
        info += `${message}`;
        console.log(info);
    }
}

module.exports = VerboseTracer;