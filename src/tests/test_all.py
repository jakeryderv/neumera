import pytest
import neumera


def test_sum_as_string():
    assert neumera.sum_as_string(1, 1) == "2"
