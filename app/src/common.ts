import * as path from 'path';

import { Gateway } from 'oasis-std';

export const MOCK_HOSTING_SERVICE = path.resolve('demo/data');

export async function logDone<T>(action: string, task: Promise<T>): Promise<T> {
    process.stdout.write(`${action.padEnd(30)} `);
    const ret = await task;
    process.stdout.write('✔️\n');
    return ret;
}

export function createGateway(): Gateway {
    return new Gateway(
        'http://localhost:1234',
        'AAAAGYHZxhwjJXjnGEIiyDCyZJq+Prknbneb9gYe9teCKrGa',
    );
}
