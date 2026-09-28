#!/bin/sh
set -eu

image="mahfel-ci-integration:$CI_JOB_ID"
container="mahfel-ci-integration-$CI_JOB_ID"
mkdir -p ci-artifacts

trap 'docker rm --force "$container" >/dev/null 2>&1 || true
      docker image rm "$image" >/dev/null 2>&1 || true' 0
trap 'exit 130' INT
trap 'exit 143' HUP TERM

docker buildx build --load --target ci-integration \
    --build-arg "CARGO_NEXTEST_VERSION=$CARGO_NEXTEST_VERSION" \
    --file backend/Dockerfile --tag "$image" .
docker create --name "$container" --network host \
    --volume /var/run/docker.sock:/var/run/docker.sock \
    --env MAHFEL_TESTCONTAINERS=1 \
    --env NO_PROXY=localhost,127.0.0.1 \
    --env no_proxy=localhost,127.0.0.1 \
    "$image" >/dev/null
docker start --attach "$container" || true
exit_code=$(docker inspect --format '{{.State.ExitCode}}' "$container")
docker cp "$container:/source/backend/target/nextest/ci/junit.xml" \
    ci-artifacts/integration-junit.xml
exit "$exit_code"
