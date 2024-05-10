const { action } = require("./action");

const logMessage = (title, message) => {
    const colors = {
        info: '\x1b[34m', // blue
        warning: '\x1b[33m', // yellow
        error: '\x1b[31m', // red
        empty: '\x1b[35m', // magenta
        db: '\x1b[36m', // cyan
        unexpected: '\x1b[90m', // gray
        success: '\x1b[32m' // green
    };

    const color = colors[title.toLowerCase()] || '\x1b[37m'; // white
    const time = new Date().toLocaleTimeString();

    console.log(`\x1b[37m[\x1b[32m${time}\x1b[37m] ${color}${title}: \x1b[37m${message}\x1b[0m`);
}

module.exports = {
    logMessage
}