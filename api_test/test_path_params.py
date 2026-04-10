"""Step definitions for path parameter mock features."""

from pytest_bdd import scenarios

scenarios("../features/path_params.feature")

# All steps are shared via conftest / common_steps – no extra steps needed.
