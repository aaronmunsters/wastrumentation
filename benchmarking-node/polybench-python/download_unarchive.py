# -*- coding: utf-8 -*-

import os
import subprocess

def download_unarchive(archive_path, archive_file, archive_link):
    """
    Given a destination path, file name & archive link;
    download and unarchive if this hasn't happened yet.
    """

    if os.path.isdir(archive_path):
        print(f'{archive_path} found, using this as reference')
        return


    print(f'{archive_path} not found, looking for archive file')
    if not os.path.isfile(archive_file):
        print(f'{archive_file} not found, downloading it')
        subprocess.run(['wget', '-O', archive_file, '--no-clobber', '--quiet', archive_link])

    if not os.path.isfile(archive_file):
        print(f'Could not download the {archive_file} from {archive_link}, aborting...')
        exit(0)

    print(f'Extracting {archive_file} to {archive_path}')
    os.makedirs(archive_path, exist_ok=True)
    subprocess.run(['tar', '--extract', '--file', archive_file, '--directory', archive_path])
