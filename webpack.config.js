const path = require('path');

const webpack = require('webpack');

const mode = process.env.NODE_ENV || 'development';

module.exports = {
    entry: {
        demo: './app/src/demo.ts',
        evaluator: './app/src/evaluator.ts',
    },
    output: {
        libraryTarget: 'commonjs2',
        filename: '[name]',
        path: path.resolve(__dirname, 'dist'),
    },
    target: 'node',
    module: {
        rules: [
            {
                test: /\.ts$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
        ],
    },
    resolve: {
        extensions: ['.ts', '.js'],
    },
    mode,
    plugins: [
        new webpack.BannerPlugin({ banner: '#!/usr/bin/env node', raw: true }),
        new webpack.DefinePlugin({
            'process.env.NODE_ENV': JSON.stringify(mode),
        }),
    ],
};
