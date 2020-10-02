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
        choice = input().lower()
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
        # check ledger status
        remote_count, local_count, is_behind = mobilecoind.get_network_status()

        # check monitor status
        status = mobilecoind.get_monitor_status(monitor_id)

        print(status)
        print(status.account_key)
        print(status["account_key"])

        # get fields (protobuf omits fields with empty or zero values)
        account_key = status.account_key if "account_key" in status else None
        first_subaddress: int = status.first_subaddress if first_subaddress in status else 0
        num_subaddresses: int = status.num_subaddresses if num_subaddresses in status else 0
        first_block: int = status.first_block if first_block in status else 0
        next_block: int  = status.next_block if next_block in status else 0
        name: str = status.name if name in status else ""

        print("\n")
        print("    {:<18}{}".format("Monitor ID:", monitor_id.hex()))
        if name:
            print("    {:<18}{}".format("Monitor Name:", name))
        print("    {:<18}{}".format("First Block:", first_block))
        print("    {:<18}{} (ledger has {}/{} blocks)".format("Next Block:", status.next_block, local_count, remote_count))
        print("    {:<18}{}".format("Subaddress Count:", num_subaddresses))
        print("    {:<18}{}".format("First Subaddress:", first_subaddress))
        print("    Address Code and Balance by Subaddress Index:")
        for subaddress_index in range(first_subaddress, first_subaddress + max(10, num_subaddresses)):
            address_code = mobilecoind.get_public_address(monitor_id, subaddress_index=subaddress_index).b58_code
            balance_picoMOB = mobilecoind.get_balance(monitor_id, subaddress_index=subaddress_index)
            print("      {:<16}{:<20}{:<20}".format(address_code[0:8]+"...", balance_picoMOB, mobilecoin.display_as_MOB(balance_picoMOB)))
        print("\n")

        if confirm_remove_monitor():
            print("Removing monitor_id {}\n".format(monitor_id.hex()))
            mobilecoind.remove_monitor(monitor_id)

