# K3s configuration

This directory contains the production K3s configuration example and Traefik HelmChartConfig.

- `config.yaml.example`: review and replace its documentation addresses before manual installation.
- `traefik.yaml`: configure HTTP-to-HTTPS redirects for the bundled ingress controller.

For automated Ubuntu 24.04 host and cluster setup, use the
[VPS bootstrap](../vps/README.md). Continue deployment with the
[production runbook](../kubernetes/RUNBOOK.md).
