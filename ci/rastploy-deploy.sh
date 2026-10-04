#!/bin/sh
set -eu
: "${RASTPLOY_SERVER:?Configure the HTTPS Rastploy endpoint}"
: "${RASTPLOY_TOKEN:?Configure a protected, masked Mahfel deployment token}"
: "${IMAGE_TARGET:?Release component is required}"
: "${CI_COMMIT_TAG:?Release tag is required}"
case "$RASTPLOY_SERVER" in
    https://*) ;;
    *) echo 'RASTPLOY_SERVER must use HTTPS.' >&2; exit 1 ;;
esac
case "$IMAGE_TARGET" in api|media|worker|web) ;; *) exit 1 ;; esac
version=${CI_COMMIT_TAG#"$IMAGE_TARGET"-}
printf '%s\n' "$CI_COMMIT_TAG" | grep -Eq "^${IMAGE_TARGET}-v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$" || exit 1
server=${RASTPLOY_SERVER%/}
payload=$(jq -cn --arg target "mahfel-$IMAGE_TARGET" --arg version "$version" \
    '{target:$target,environment:"production",version:$version}')
response=$(curl --fail --silent --show-error --connect-timeout 5 --max-time 30 \
    -X POST "$server/api/v1/deployments" \
    -H "Authorization: Bearer $RASTPLOY_TOKEN" \
    -H 'Content-Type: application/json' --data "$payload")
id=$(printf '%s' "$response" | jq -er '.id | strings | select(test("^[a-zA-Z0-9_-]+$"))')
echo "Deployment $id queued: mahfel-$IMAGE_TARGET production $version"
for _attempt in $(seq 1 90); do
    response=$(curl --fail --silent --show-error --retry 2 --connect-timeout 5 --max-time 15 \
        "$server/api/v1/deployments/$id" -H "Authorization: Bearer $RASTPLOY_TOKEN")
    status=$(printf '%s' "$response" | jq -er '.status | strings')
    case "$status" in
        succeeded) echo "Deployment $id succeeded."; exit 0 ;;
        failed|canceled)
            echo "Deployment $id $status. Inspect Rastploy deployment logs." >&2; exit 1 ;;
        queued|running) ;;
        *) echo "Unexpected deployment status: $status" >&2; exit 1 ;;
    esac
    sleep 10
done
echo "Timed out waiting for deployment $id. Check its status before retrying." >&2
exit 1
