# Custom resource schemas

`sealedsecret.json` is the `v1alpha1` OpenAPI schema extracted from the official Sealed Secrets
2.20.0 Helm chart (controller 0.40.0), downloaded from https://bitnami.github.io/sealed-secrets.
It lets `just kubernetes lint production` validate encrypted resources after sealing without
ignoring unknown CRDs. Update it together with a reviewed controller/chart upgrade.

Source: https://github.com/bitnami/sealed-secrets/tree/main/helm/sealed-secrets/crds
