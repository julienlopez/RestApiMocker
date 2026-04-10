"""
Shared fixtures for the API test suite.

The ``backend`` fixture builds and spawns the backend binary once per test,
using random ports so tests can run in parallel without conflicts.
It tears the process down after each scenario to guarantee a clean state.
"""

import socket
import subprocess
import sys
import time
from pathlib import Path

import pytest
import requests as http

# Import shared step definitions so pytest-bdd can discover them.
from common_steps import *  # noqa: F401, F403

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent
BACKEND_DIR = WORKSPACE_ROOT / "backend"


def _free_port() -> int:
    """Return an OS-assigned free TCP port."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def _wait_for_port(port: int, timeout: float = 15.0) -> None:
    """Block until *port* accepts a TCP connection or *timeout* is exceeded."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.5):
                return
        except OSError:
            time.sleep(0.15)
    raise TimeoutError(f"Port {port} not ready after {timeout}s")


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture()
def backend():
    """Build & start the backend, yield connection info, then stop it."""
    # 1. Build (fail fast if compilation errors)
    build = subprocess.run(
        ["cargo", "build", "-p", "backend"],
        cwd=str(WORKSPACE_ROOT),
        capture_output=True,
        text=True,
    )
    if build.returncode != 0:
        pytest.fail(f"cargo build failed:\n{build.stderr}")

    # 2. Resolve binary path
    if sys.platform == "win32":
        binary = WORKSPACE_ROOT / "target" / "debug" / "backend.exe"
    else:
        binary = WORKSPACE_ROOT / "target" / "debug" / "backend"
    assert binary.exists(), f"Binary not found at {binary}"

    # 3. Pick random ports
    public_port = _free_port()
    private_port = _free_port()

    env = {
        "PUBLIC_PORT": str(public_port),
        "PRIVATE_PORT": str(private_port),
    }

    # 4. Start the process
    proc = subprocess.Popen(
        [str(binary)],
        env={**subprocess.os.environ, **env},
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    try:
        _wait_for_port(public_port)
        _wait_for_port(private_port)
    except TimeoutError:
        proc.kill()
        stdout, stderr = proc.communicate(timeout=5)
        pytest.fail(
            f"Backend did not start in time.\nstdout: {stdout.decode()}\nstderr: {stderr.decode()}"
        )

    class _Info:
        """Lightweight bag exposed to tests."""

        def __init__(self):
            self.public_port = public_port
            self.private_port = private_port
            self.public_url = f"http://127.0.0.1:{public_port}"
            self.internal_url = f"http://127.0.0.1:{private_port}"
            self.process = proc

    info = _Info()
    yield info

    # 5. Teardown
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait(timeout=5)


@pytest.fixture()
def public_url(backend):
    return backend.public_url


@pytest.fixture()
def internal_url(backend):
    return backend.internal_url
