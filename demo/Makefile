.PHONY: demo docker clean

demo: \
	data/iris_train.csv data/iris_test.csv \
	models \
	docker

data/iris.csv: entropy.bin | data
	@curl -s 'https://archive.ics.uci.edu/ml/machine-learning-databases/iris/iris.data' | shuf --random-source $< > $@

data/iris_train.csv: data/iris.csv
	@head -n 100 $< > $@

data/iris_test.csv: data/iris.csv
	@tail -n +101 $< > $@

data:
	@mkdir -p $@

models: train_models.py data/iris_train.csv | .docker_built
	@mkdir -p models
	@docker run --rm -v "$(shell pwd):/demo" ready-layer-two-trainer:latest python /demo/train_models.py

.docker_built: Dockerfile .dockerignore requirements.txt evaluator.py
	@docker build -q . -t ready-layer-two-trainer:latest --target model-trainer
	@docker build -q . -t ready-layer-two-evaluator:latest
	@touch .docker_built

clean:
	@rm -rf data models
