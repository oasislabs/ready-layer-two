import { spawnSync } from 'child_process';
import { createDecipheriv } from 'crypto';
import * as fs from 'fs';
import * as path from 'path';

import { Address, decodeHex } from 'oasis-std';
import * as tmp from 'tmp';

import {
    AttestationReport,
    Competition,
    EncryptedData,
} from '../service-clients/competition';

import { MOCK_HOSTING_SERVICE, createGateway, logDone } from './common';

const SANDBOXED_EVALUATOR_IMAGE = 'ready-layer-2-evaluator:latest';
//^ this would include the hash in a not-demo

tmp.setGracefulCleanup();

async function main() {
    const competitionAddrHex = process.argv.pop()!;
    const mockMeasurement = process.argv.pop()!;

    const attestation = new AttestationReport({
        measurement: decodeHex(mockMeasurement),
        signature: Buffer.alloc(0), // mock
    });

    const gw = createGateway();

    const competition = await Competition.connect(
        new Address(competitionAddrHex),
        gw,
    );

    const { testDataset, submissions } = await logDone(
        'Beginning evaluation',
        competition.beginEvaluation({
            attestation,
        }),
    );

    const sandboxInputVol = tmp.dirSync().name;

    const testDatasetBasename = await downloadAndDecrypt(
        testDataset,
        sandboxInputVol,
    );

    const evaluations = [];
    for (const submission of submissions) {
        const [participant, model] = submission;
        evaluations.push(
            evaluate(
                model,
                sandboxInputVol,
                testDatasetBasename,
            ).then((score) => [participant, score]),
        );
    }
    const scores = await logDone(
        'Evaluating submissions',
        Promise.all(evaluations),
    );

    const winner = scores.reduce((winning, score) =>
        score[1] > winning[1] ? score : winning,
    )[0] as string;

    await logDone(
        'Announcing winner',
        competition.announceWinner({ attestation, winner }),
    );

    await gw.disconnect();
}

async function downloadAndDecrypt(
    data: EncryptedData,
    destDir: string,
): Promise<string> {
    const filename = path.basename(data.url, '.enc');
    const storagePath = path.join(destDir, filename);

    // fake download
    const downloadUrl = data.url.replace(/^file:\/\//, '');

    const { key, iv, tag } = data.cipher;
    const decipher = createDecipheriv('aes-256-gcm', key, iv);
    decipher.setAuthTag(tag);

    const writer = fs
        .createReadStream(downloadUrl)
        .pipe(decipher)
        .pipe(fs.createWriteStream(storagePath));

    return new Promise((resolve, reject) => {
        writer.on('finish', () => {
            resolve(filename);
        });

        writer.on('error', reject);
    });
}

async function evaluate(
    model: EncryptedData,
    sandboxInputVol: string,
    testDatasetBasename: string,
): Promise<number> {
    const modelBasename = await downloadAndDecrypt(model, sandboxInputVol);
    const cp = spawnSync('docker', [
        'run',
        '--rm',
        '--network=none',
        '-v',
        `${sandboxInputVol}:/data`,
        SANDBOXED_EVALUATOR_IMAGE,
        '--data-path',
        `/data/${testDatasetBasename}`,
        '--model-path',
        `/data/${modelBasename}`,
    ]);
    const stderr = Buffer.from(cp.stderr).toString();
    if (stderr) {
        throw new Error(stderr);
    }
    const score = Buffer.from(cp.stdout).toString().trim();
    return parseFloat(score);
}

main().catch(console.error);
