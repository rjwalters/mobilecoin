#!/usr/bin/env python3
# -*- coding: utf-8 -*-

# Copyright (c) 2018-2020 MobileCoin Inc.

# display information about all active monitors and prompt the user to keep or remove each

import os,sys
sys.path.insert(1, os.path.realpath(os.path.join(os.path.pardir, "lib")))
import mobilecoin

def confirm_remove_monitor():
    valid = {"yes": True, "y": True, "ye": True,
             "no": False, "n": False}
    while True:
        sys.stdout.write("Remove this monitor? [y/N]")
        choice = raw_input().lower()
        if choice == '':
            return valid["no"]
        elif choice in valid:
            return valid[choice]
        else:
            sys.stdout.write("Please respond with 'yes' or 'no'.\n")

if __name__ == '__main__':
    # Connect to mobilecoind
    mobilecoind = mobilecoin.Client("localhost:4444", ssl=False)

    # iterate over all active monitors
    for monitor_id in mobilecoind.get_monitor_list():
        status = mobilecoind.get_monitor_status(monitor_id)
        print()
        print(status)
        print()
        if confirm_remove_monitor():
            print("Removing monitor_id {}\n".format(monitor_id.hex()))
            mobilecoind.remove_monitor(monitor_id)

