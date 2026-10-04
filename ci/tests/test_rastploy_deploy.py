"""Verify release payloads and failure handling without contacting a deployment host."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "rastploy-deploy.sh"


class RastployDeploymentTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        curl = self.root / "curl"
        curl.write_text("""#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
args = sys.argv[1:]
if "-X" in args:
    Path(os.environ["REQUEST_FILE"]).write_text(args[args.index("--data") + 1])
    print(json.dumps({"id": os.environ.get("MOCK_ID", "fixture-deployment")}))
else:
    print(json.dumps({"status": os.environ.get("MOCK_STATUS", "succeeded")}))
""")
        curl.chmod(0o755)
        self.environment = {
            **os.environ,
            "PATH": f"{self.root}:{os.environ['PATH']}",
            "REQUEST_FILE": str(self.root / "request.json"),
            "RASTPLOY_SERVER": "https://deployment.example.test/",
            "RASTPLOY_TOKEN": "fixture-only-secret",
            "IMAGE_TARGET": "api",
            "CI_COMMIT_TAG": "api-v1.2.3",
        }

    def run_script(self, **changes):
        return subprocess.run(
            ["sh", str(SCRIPT)], env={**self.environment, **changes},
            text=True, capture_output=True, timeout=5,
        )

    def test_each_component_deploys_its_published_tag(self):
        for component in ("api", "media", "worker", "web"):
            with self.subTest(component=component):
                result = self.run_script(IMAGE_TARGET=component, CI_COMMIT_TAG=f"{component}-v1.2.3")
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(json.loads((self.root / "request.json").read_text()), {
                    "target": f"mahfel-{component}", "environment": "production", "version": "v1.2.3",
                })
                self.assertNotIn("fixture-only-secret", result.stdout + result.stderr)

    def test_failed_and_canceled_deployments_fail_the_job(self):
        for status in ("failed", "canceled", "unexpected"):
            with self.subTest(status=status):
                self.assertNotEqual(self.run_script(MOCK_STATUS=status).returncode, 0)

    def test_rejects_plain_http_before_sending_credentials(self):
        self.assertNotEqual(self.run_script(RASTPLOY_SERVER="http://deployment.example.test").returncode, 0)
        self.assertFalse((self.root / "request.json").exists())

    def test_rejects_wrong_component_and_nonrelease_tags(self):
        for tag in ("web-v1.2.3", "api-v01.2.3", "api-v1.2.3-rc1", "api-v1.2.3;echo injected"):
            with self.subTest(tag=tag):
                self.assertNotEqual(self.run_script(CI_COMMIT_TAG=tag).returncode, 0)
                self.assertFalse((self.root / "request.json").exists())

    def test_requires_token_before_contacting_host(self):
        self.assertNotEqual(self.run_script(RASTPLOY_TOKEN="").returncode, 0)
        self.assertFalse((self.root / "request.json").exists())

    def test_rejects_invalid_deployment_id(self):
        self.assertNotEqual(self.run_script(MOCK_ID="../../other-resource").returncode, 0)


if __name__ == "__main__":
    unittest.main()
