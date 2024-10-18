# -*- coding: utf-8 -*-
import subprocess
import io

from config import firefox_link_linux, firefox_link_macos

# This code is MacOS specific
class FireFoxFetcher:
    def __init__(self) -> None:
        firefox_mount_path = subprocess.run(
            ['bash', '-c', 'realpath /Volumes/Firefox*'], capture_output=True, text=True
        ).stdout.strip()

        if not firefox_mount_path:
            print('(temporary MacOS Firefox installation) Fetching')
            subprocess.run(
                ['wget', '-O', 'firefox.dmg', '--no-clobber', '--quiet', firefox_link_macos],
                capture_output=True,
            )

            print('(temporary MacOS Firefox installation) Mounting')
            subprocess.run(
                ['hdiutil', 'attach', 'firefox.dmg', '-quiet'],
                capture_output=True,
            )

        firefox_binary = subprocess.run(
            ['bash', '-c', 'realpath /Volumes/Firefox*/Firefox*.app/Contents/MacOS/firefox'],
            capture_output=True,
            text=True,
        ).stdout.strip()

        firefox_version = subprocess.run(
            [firefox_binary, '--version'],
            capture_output=True,
            text=True,
        ).stdout.strip()

        if not firefox_binary or not firefox_version:
            print(f'Is the following a binary to a web browser? {firefox_binary}')
            print('Because I could not tell. Aborting.')
            exit(0)

        self.firefox_binary = firefox_binary
        self.firefox_version = firefox_version

    def binary(self):
        return self.firefox_binary

    def version(self):
        return self.firefox_version
