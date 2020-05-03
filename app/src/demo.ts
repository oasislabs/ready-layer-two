import { spawnSync } from 'child_process';
import { createCipheriv, createHash, randomBytes } from 'crypto';
import * as fs from 'fs';
import * as path from 'path';

import { encodeHex } from 'oasis-std';

import {
    Aes256GcmParams,
    AuthenticatedData,
    Competition,
    CompetitionCompleted,
    EncryptedData,
} from '../service-clients/competition';
import { UserRegistry } from '../service-clients/user-registry';

import {
    MOCK_HOSTING_SERVICE,
    STATUS_MSG_WIDTH,
    createGateway,
    TaskLogger,
} from './common';

const taskLogger = new TaskLogger();
const logDone = taskLogger.logDone.bind(taskLogger);

async function main() {
    const gw = createGateway();

    const userRegistry = await logDone(
        'Create UserRegistry',
        UserRegistry.deploy(gw),
    );

    const participants = [
        ['PEGASOS', 'password'],
        ['AdaBooster', 'password'],
    ];

    await logDone(
        'Register participants',
        Promise.all(
            participants.map(([name, loginCredential]) => {
                return userRegistry.register({
                    name,
                    loginCredential,
                });
            }),
        ),
    );

    const [
        trainDataset,
        testDataset,
        evaluationProgram,
        submissions,
    ] = await logDone(
        'Upload data',
        Promise.all([
            uploadFile('demo/data/iris_train.csv'),
            encryptAndUploadFile('demo/data/iris_test.csv'),
            uploadFile('app/src/evaluator.ts'),
            Promise.all([
                // These would be uploaded after the public training data is posted
                // but these demo models are pre-trained, so we can upload them now.
                encryptAndUploadFile('demo/models/model_a.joblib'),
                encryptAndUploadFile('demo/models/model_b.joblib'),
            ]),
        ]),
    );

    const endTimestampMillis = Date.now() + 7 * 1000;
    const competition = await logDone(
        'Create Competition',
        Competition.deploy(gw, {
            userRegistry: userRegistry.address,
            trainDataset,
            testDataset,
            evaluationProgram,
            endTimestamp: BigInt(Math.floor(endTimestampMillis / 1000)),
        }),
    );

    await logDone(
        'Make submissions',
        Promise.all(
            participants.map(async ([name, loginCredential], i) => {
                const participantAuthToken = await userRegistry.signIn({
                    name,
                    loginCredential,
                    audience: competition.address,
                });
                return competition.submit({
                    participantAuthToken,
                    model: submissions[i],
                });
            }),
        ),
    );

    await logDone(
        'Start evaluation program "enclave"',
        new Promise((resolve) =>
            setTimeout(resolve, endTimestampMillis - Date.now()),
        ),
    );

    const winnerNotification = await CompetitionCompleted.subscribe(
        gw,
        competition.address,
    );

    spawnSync(
        process.argv[0],
        [
            'dist/evaluator',
            encodeHex(evaluationProgram.hash),
            competition.address.hex,
        ],
        {
            stdio: [null, 'inherit', 'inherit'],
        },
    );

    const { winner } = await winnerNotification.first();
    console.log(`\n🎉 ${winner} has won the competition! 🎉`);

    await gw.disconnect();
}

async function uploadFile(filePath: string): Promise<AuthenticatedData> {
    const reader = fs.createReadStream(filePath);
    const hasher = createHash('sha256');
    return new Promise((resolve, reject) => {
        reader.on('readable', () => {
            const data = reader.read();
            if (data) {
                hasher.update(data);
            } else {
                resolve(
                    new AuthenticatedData({
                        url: `file://${path.resolve(filePath)}`,
                        hash: hasher.digest(),
                    }),
                );
            }
        });

        reader.on('error', reject);
    });
}

async function encryptAndUploadFile(filename: string): Promise<EncryptedData> {
    const { name, ext } = path.parse(filename);
    const disambiguator = randomBytes(3).toString('hex');
    const encName = `${name}-${disambiguator}${ext}.enc`;
    const storagePath = path.join(MOCK_HOSTING_SERVICE, encName);

    const key = randomBytes(256 / 8);
    const iv = randomBytes(256 / 2 / 8);
    const cipher = createCipheriv('aes-256-gcm', key, iv);

    const writer = fs
        .createReadStream(filename)
        .pipe(cipher)
        .pipe(fs.createWriteStream(storagePath));

    return new Promise((resolve, reject) => {
        writer.on('finish', () => {
            resolve(
                new EncryptedData({
                    url: `file://${storagePath}`,
                    cipher: new Aes256GcmParams({
                        key,
                        iv,
                        tag: cipher.getAuthTag(),
                    }),
                }),
            );
        });

        writer.on('error', reject);
    });
}

main().catch(console.error);
