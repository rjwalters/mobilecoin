#!/usr/bin/env python3

# Copyright (c) 2018-2020 MobileCoin Inc.

# used to download our mailing list and output a table with all user balances

import sys, os.path
import mobilecoin
import argparse
from mailchimp3 import MailChimp

if __name__ == '__main__':
    # Parse the arguments and generate the mobilecoind client
    mobilecoind = mobilecoin.Client("localhost:4444", ssl=False)

    parser = argparse.ArgumentParser(description='provide secrets')
    parser.add_argument('-k', '--key', help='MailChimp API key', type=str, required=True)
    parser.add_argument('--clean', help='remove all old monitors', action='store_true')
    args = parser.parse_args()

    print("\n# *\n# * Starting up TestNet member balance check script!\n# *\n#")

    # clean up all old monitors
    if args.clean:
        for monitor_id in mobilecoind.get_monitor_list():
            print("# removing existing monitor_id {}.".format(monitor_id.hex()))
            mobilecoind.remove_monitor(monitor_id)

    # generate the MailChimp client
    mailchimp = MailChimp(mc_api=args.key)

    # figure out the id for the list of interest
    # print(mailchimp.lists.all(get_all=True, fields="lists.name,lists.id"))
    list_id = '5f47419453' # The "MobileCoin" Audience

    # go through all the subscribers in chunks
    fields="members.id,members.email_address,members.merge_fields,members.status" # important: no spaces!
    offset = 0
    count = 200 # can be up to 1000
    members = []
    while count > 0:
        chunk_of_members = mailchimp.lists.members.all(list_id, count=count, offset=offset, fields=fields)["members"]
        count = len(chunk_of_members)
        offset += count
        members.extend(chunk_of_members)

    print("# mailing list contains {} members.".format(len(members)))

    for member_record in members:
        if member_record["merge_fields"]["ENTROPY"]:
            entropy = member_record["merge_fields"]["ENTROPY"]
            email = member_record["email_address"]
            try:
                entropy_bytes = bytes.fromhex(entropy)
                account_key = mobilecoind.get_account_key(entropy_bytes)
                monitor_id = mobilecoind.add_monitor(account_key)
                print("# adding monitor_id {} for {}".format(monitor_id.hex(), email))

                (monitor_is_behind, next_block, remote_count, blocks_per_second) = mobilecoind.wait_for_monitor(monitor_id)
                if monitor_is_behind:
                    print("#\n# waiting for the monitor to process {} blocks".format(remote_count - next_block))
                    while monitor_is_behind:
                        blocks_remaining = (remote_count - next_block)
                        if blocks_per_second > 0:
                            time_remaining_seconds = blocks_remaining / blocks_per_second
                            print("#    {} blocks remain ({} seconds)".format(blocks_remaining, round(time_remaining_seconds, 1)))
                        else:
                            print("#    {} blocks remain (? seconds)".format(blocks_remaining))
                        (monitor_is_behind, next_block, remote_count, blocks_per_second) = mobilecoind.wait_for_monitor(monitor_id)
                    print("# monitor has processed all {} blocks\n#".format(local_count))

                balance_picoMOB = mobilecoind.get_balance(monitor_id)
                print("{}, {}, {}, {}MOB".format(entropy, email, balance_picoMOB, mobilecoind.display_as_MOB(balance_picoMOB)))

            except:
                print("\n# ERROR: failed to get balance for {} ({})\n".format(entropy, email))
