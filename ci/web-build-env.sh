#!/bin/sh

case "${CI_COMMIT_TAG:-}" in
    web-v*) NEXT_PUBLIC_MAHFEL_API_URL=${MAHFEL_API_URL_PRODUCTION:-} ;;
esac

if [ -z "${NEXT_PUBLIC_MAHFEL_API_URL:-}" ]; then
    echo 'Set the public Mahfel API URL for this branch in GitLab CI variables.' >&2
    exit 1
fi
export NEXT_PUBLIC_MAHFEL_API_URL
