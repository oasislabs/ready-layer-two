#!/usr/bin/env python3

from contextlib import contextmanager
from os import path as osp

import joblib
import pandas as pd
from sklearn.ensemble import AdaBoostClassifier
from sklearn.svm import SVC


DEMO_DIR = osp.abspath(osp.dirname(__file__))
DATA_DIR = osp.join(DEMO_DIR, "data")
MODELS_DIR = osp.join(DEMO_DIR, "models")


@contextmanager
def load_data(train=True):
    df = pd.read_csv(osp.join(DATA_DIR, f'iris_{"train" if train else "test"}.csv'), header=None)
    df.columns = ["sepal length", "sepal width", "petal length", "petal width", "label"]

    X = df.drop(["label"], axis=1)
    y = pd.factorize(df["label"], sort=True)[0]

    yield X, y


def main():
    with load_data(train=True) as (X, y):
        model_a = SVC(gamma="scale")
        model_a.fit(X, y)

        model_b = AdaBoostClassifier()
        model_b.fit(X, y)

        print("train")
        print(f"├─ model A score: {model_a.score(X, y):.3f}")
        print(f"└─ model B score: {model_b.score(X, y):.3f}")

    with load_data(train=False) as (X, y):
        print("\ntest (debugging only. you wouldn't see these irl)")
        print(f"├─ model A score: {model_a.score(X, y):.3f}")
        print(f"└─ model B score: {model_b.score(X, y):.3f}")

    joblib.dump(model_a, osp.join(MODELS_DIR, "model_a.joblib"))
    joblib.dump(model_b, osp.join(MODELS_DIR, "model_b.joblib"))


if __name__ == "__main__":
    main()
