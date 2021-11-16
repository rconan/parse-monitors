phony: build update test

build:
	cargo build  --release --target x86_64-unknown-linux-gnu

update:
	cp ./target/x86_64-unknown-linux-gnu/release/main ./bootstrap && zip lambda.zip bootstrap && rm bootstrap
	aws lambda update-function-code --function-name rustTest  --zip-file fileb://./lambda.zip

create:
	aws lambda create-function --function-name rustTest --handler doesnt.matter --zip-file fileb://./lambda.zip  --runtime provided.al2  --role arn:aws:iam::378722409401:role/lambda_basic_execution --timeout 60 --environment Variables={RUST_BACKTRACE=1} --tracing-config Mode=Active

test:
	aws lambda invoke --function-name rustTest --payload '{"key": "Baseline2021/Baseline2021/Baseline2021/CASES/zen30az000_OS7/pressures/M1p_M1p_7.000000e+02.csv.bz2"}' output.json
	cat output.json

all: build update test
