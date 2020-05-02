FROM python:3-slim as model-trainer

COPY requirements.txt evaluator.py ./
RUN pip install -r requirements.txt && rm requirements.txt

FROM model-trainer

ENTRYPOINT ["python", "evaluator.py"]
