# Security policy

Stella Scope runs on your own computer and talks to your own printers, and it can be set up to send commands to machines with heaters and motors. We take security reports seriously.

## Reporting a vulnerability

**Please do not open a public issue for a security problem.**

Report it privately in either of these ways:

- Use GitHub's private reporting: on this repository, open the **Security** tab and choose **Report a vulnerability**.
- Or email **info@3d3d.ca** with the subject `Stella Scope security`.

Please include what you found, the version (More, About), the operating system, how to reproduce it, and what you think the impact is. Do not include real printer passwords, API keys or pairing keys in your report.

We will acknowledge your report, keep you informed while we investigate and fix it, and credit you in the release notes if you wish. We ask that you give us a reasonable time to release a fix before you publish details.

## What counts

Examples we want to hear about:

- a way to make Stella Scope send a command to a printer without the person's action, or to bypass a heat or motion limit;
- a way to reach the web interface or its controls from outside the machine, or from another web page (cross-site requests);
- reading or writing files outside the places Stella Scope owns, or leaking printer passwords, access codes or pairing keys;
- a weakness in the phone link (pairing, encryption, the relay);
- anything in a download path that lets an unverified file be installed.

## Supported versions

The latest release and `main`. Older releases are not patched.

## Hardening advice for users

- Keep the web interface on `127.0.0.1` (the default). It has no login of its own; never expose it to the internet.
- Leave printer controls off unless you need them, and never enable them on a network address.
- Keep printer passwords and access codes in the files Stella Scope creates for them, readable only by you.
