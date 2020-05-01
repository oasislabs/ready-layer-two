#!/usr/bin/env python3

import argparse

import joblib
import pandas as pd


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--model-path")
    parser.add_argument("--data-path")
    args = parser.parse_args()

    X, y = load_data(args.data_path)
    model = joblib.load(args.model_path)
    print(model.score(X, y))


def load_data(data_path):
    df = pd.read_csv(data_path, header=None)
    df.columns = ["sepal length", "sepal width", "petal length", "petal width", "label"]

    X = df.drop(["label"], axis=1)
    y = pd.factorize(df["label"], sort=True)[0]

    return X, y


if __name__ == "__main__":
    main()
