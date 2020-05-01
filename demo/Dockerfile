FROM python:3-slim
COPY requirements.txt evaluator.py ./
RUN pip install -r requirements.txt && rm requirements.txt
ENTRYPOINT ["python", "evaluator.py"]
