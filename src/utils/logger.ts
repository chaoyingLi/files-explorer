// Frontend logging utility — captures errors and sends to Rust log file

const LOG_BUFFER: string[] = [];
const MAX_BUFFER = 500;

function ts(): string {
  return new Date().toISOString().slice(11, 23);
}

export const logger = {
  info(msg: string, ...args: unknown[]) {
    const line = `[${ts()}] INFO  ${msg}`;
    LOG_BUFFER.push(line);
    if (LOG_BUFFER.length > MAX_BUFFER) LOG_BUFFER.shift();
    console.log(line, ...args);
  },

  warn(msg: string, ...args: unknown[]) {
    const line = `[${ts()}] WARN  ${msg}`;
    LOG_BUFFER.push(line);
    if (LOG_BUFFER.length > MAX_BUFFER) LOG_BUFFER.shift();
    console.warn(line, ...args);
  },

  error(msg: string, ...args: unknown[]) {
    const line = `[${ts()}] ERROR ${msg}`;
    LOG_BUFFER.push(line);
    if (LOG_BUFFER.length > MAX_BUFFER) LOG_BUFFER.shift();
    console.error(line, ...args);
  },

  getRecent(count = 50): string[] {
    return LOG_BUFFER.slice(-count);
  },

  getAll(): string[] {
    return [...LOG_BUFFER];
  },

  clear() {
    LOG_BUFFER.length = 0;
  },
};
