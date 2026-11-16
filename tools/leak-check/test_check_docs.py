"""Unit checks for tools/leak-check/check_docs.py."""
from __future__ import annotations

from check_docs import ip_allowed, line_errors


def test_allows_protocol_constants_and_loopback() -> None:
    assert line_errors('seed 0x04A80B38') == []
    assert line_errors('mask 0x7FFFFFFF') == []
    assert line_errors('space is 0x20') == []  # too short for the address pattern
    assert line_errors('listen on 127.0.0.1:14900') == []
    assert line_errors('client 2.2.2.0') == []
    assert line_errors('fake resolver 198.18.0.1') == []


def test_flags_code_addresses_names_and_public_ips() -> None:
    assert any('code address' in e for e in line_errors('handler at 0x2e0b88'))
    assert any('disassembler name' in e for e in line_errors('see FUN_003c9a80'))
    assert any('PTR_DAT_' in e for e in line_errors('pad PTR_DAT_007ad4b8'))
    # Neutral public DNS — not a game-server host from the scrubbed docs.
    assert any('public IPv4' in e for e in line_errors('host 8.8.8.8:53'))
    assert not ip_allowed('1.1.1.1')


def test_ip_allowlist() -> None:
    assert ip_allowed('127.0.0.1')
    assert ip_allowed('192.0.2.10')
    assert ip_allowed('198.51.100.1')
    assert ip_allowed('203.0.113.5')
    assert ip_allowed('4.0.2.11')  # client version string in the docs
    assert not ip_allowed('8.8.8.8')
    assert not ip_allowed('1.1.1.1')
