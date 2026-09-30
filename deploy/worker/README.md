# Chromium sandbox profile

`chromium-seccomp.json` is the unmodified profile from
[Playwright v1.63.0](https://github.com/microsoft/playwright/blob/v1.63.0/utils/docker/seccomp_profile.json),
licensed under Apache-2.0. It extends the default Docker seccomp profile with the namespace
operations required by Chromium's sandbox. The worker remains a non-root process.

Compose loads this profile directly. The Minikube application workflow installs it under
`/var/lib/kubelet/seccomp/mahfel/chromium.json`; other clusters must provision it on worker nodes
before deploying the worker's `Localhost` seccomp profile.
