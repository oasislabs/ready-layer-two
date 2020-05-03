import * as path from 'path';
import * as readline from 'readline';

import { Gateway } from 'oasis-std';

export const STATUS_MSG_WIDTH = 30;

export const MOCK_HOSTING_SERVICE = path.resolve('demo/data');

export class TaskLogger {
    public constructor(private prefix = '   ') {}

    public async logDone<T>(action: string, task: Promise<T>): Promise<T> {
        const msg = `${this.prefix}${action.padEnd(
            STATUS_MSG_WIDTH - this.prefix.length,
        )}`;
        process.stdout.write(`🔜 ${msg}`);
        const ret = await task;
        readline.clearLine(process.stdout, 0);
        readline.cursorTo(process.stdout, 0);
        process.stdout.write(`✔️  ${msg}\n`);
        return ret;
    }
}

export function createGateway(): Gateway {
    return new Gateway(
        'http://localhost:1234',
        'AAAAGYHZxhwjJXjnGEIiyDCyZJq+Prknbneb9gYe9teCKrGa',
    );
}
