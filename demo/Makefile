.PHONY: demo docker clean

venv_dir = .venv

demo: \
	data/iris_train.csv data/iris_test.csv \
	$(venv_dir)/installed \
	models \
	docker

data/iris.csv: entropy.bin | data
	curl -s 'https://archive.ics.uci.edu/ml/machine-learning-databases/iris/iris.data' | shuf --random-source $< > $@

data/iris_train.csv: data/iris.csv
	head -n 100 $< > $@

data/iris_test.csv: data/iris.csv
	tail -n +101 $< > $@

data:
	@mkdir -p $@

$(venv_dir)/installed: $(venv_dir)
	@touch $@

$(venv_dir): requirements.txt
	python3 -m virtualenv $@
	. $(venv_dir)/bin/activate; pip install -r requirements.txt

models: train_models.py data/iris_train.csv
	@mkdir -p models
	. $(venv_dir)/bin/activate; python3 train_models.py

docker: Dockerfile requirements.txt evaluator.py
	docker build . -t ready-layer-2-evaluator:latest

clean:
	rm -rf data models $(venv_dir)
