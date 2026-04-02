# Run all unit tests and integration tests
[group('test')]
test_all: test integration_test

[group('test')]
test test_name="":
  cargo test -p directory {{test_name}}
  cargo test -p federation {{test_name}}
  cargo test -p user {{test_name}}
  cargo test -p activitypub {{test_name}}
  cargo test -p db-utils {{test_name}}
  cargo test -p ic-utils {{test_name}}

[group('test')]
integration_test test_name="":
  echo "Running integration tests..."
  cargo test {{test_name}} -p integration-tests
