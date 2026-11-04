# Service Utils Example 🔧

This example demonstrates how to use the `service_utils` crate to test service binaries in your integration tests.

## Features

- Service binary management
- Wait strategy implementation
- Environment variable configuration
- Integration test setup

## Running the Example

Run the example with Cargo:

```bash
# Copy the binary to run into the test folder.  
cp target/debug/service_example examples/service_utils_example/tests/service 
    
# Run the example     
cargo test -p service_utils_example
```

Note, the binary must be copied manually into the test folder otherwise the test run will fail
due to a panic when checking for a non existing binary.

This is expected since the Cargo configuration does not copy the binary into the tests directory automatically.

In a Cargo only setup, you can either set the path to target/debug/ to prevent this error
or copy all binaries into a separate folder before running any tests with the service util.

The service stops when the `ServiceUtil` is dropped, under Cargo as under Bazel, so nothing is left holding its port.  

Run the example with Bazel:

```bash
bazel  test //... --test_tag_filters=service_utils_example --test_env=ENV=LOCAL
```

Bazel copies the binary automatically into the test folder. However, the test_env flag must be set
so that the test runner selects the correct root path. This is necessary because Bazel always works from the project
root folder whereas Cargo works from the crate root folder, hence the two different root paths. 
In a Bazel only environment, you can safely assume the project root folder as root path and remove the test_env flag.
Also, when Bazel ends the test, it also terminates all sandboxes and all processes
meaning there is no need to kill any processes manually.

The example demonstrates:

- Starting a service binary
- Waiting for the service to be ready
- Testing service endpoints

## Dependencies

- service_utils = "0.4"
