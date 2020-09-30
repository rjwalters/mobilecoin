#!/usr/bin/env python3

# Copyright (c) 2018-2020 MobileCoin Inc.

# used to allocation TestNet mobilecoins to users who sign up for our mailing list

import sys, os.path
import mobilecoin
import argparse
from mailchimp3 import MailChimp
import time

TX_RECEIPT_CHECK_INTERVAL_SECONDS = 2

def wait_for_monitor(monitor_id):
    (monitor_is_behind, next_block, remote_count, blocks_per_second) = mobilecoind.wait_for_monitor(monitor_id)
    if monitor_is_behind:
        print("#\n# waiting for the sender's monitor to process {} blocks".format(remote_count - next_block))
        while monitor_is_behind:
            blocks_remaining = (remote_count - next_block)
            if blocks_per_second > 0:
                time_remaining_seconds = blocks_remaining / blocks_per_second
                print("#    {} blocks remain ({} seconds)".format(blocks_remaining, round(time_remaining_seconds, 1)))
            else:
                print("#    {} blocks remain (? seconds)".format(blocks_remaining))
            (monitor_is_behind, next_block, remote_count, blocks_per_second) = mobilecoind.wait_for_monitor(monitor_id)
        print("# sender's  monitor has processed all {} blocks\n#".format(local_count))
    return remote_count

# generates a new master key, allocates funds, stores data at Mailchimp and triggers the welcome email
def allocate_MOB(mailchimp_member_record, amount_picoMOB):
    new_user_email = mailchimp_member_record["email_address"]
    new_user_hash = mailchimp_member_record["id"]

    # Wait for mobilecoind to sync ledger
    block_count = wait_for_monitor(sender_monitor_id)

    # abort if sender's balance is too low
    sender_balance_picoMOB = mobilecoind.get_balance(sender_monitor_id)
    if sender_balance_picoMOB < args.value * MOB:
        print("# sender's balance is running low ({} MOB)... aborting!".format(sender_balance_picoMOB/MOB))
        sys.exit()

    # create and fund a new MobileCoin TestNet account
    recipient_entropy = mobilecoind.generate_entropy()
    recipient_account_key = mobilecoind.get_account_key(recipient_entropy)
    print("# generated entropy {} for email {}".format(recipient_entropy.hex(), new_user_email))

    # no need to start the recipient from the origin block since we know we just created this account
    recipient_monitor_id = mobilecoind.add_monitor(recipient_account_key, first_block=block_count)
    recipient_public_address = mobilecoind.get_public_address(recipient_monitor_id)
    print("# adding monitor {} for {} (first block = {})".format(recipient_monitor_id.hex(), new_user_email, block_count))

    # Construct and send the MOB allocation transaction
    tx_list = mobilecoind.get_unspent_tx_output_list(sender_monitor_id)
    outlays = [{'value': amount_picoMOB, 'receiver': recipient_public_address}]

    tx_proposal = mobilecoind.generate_tx(sender_monitor_id, mobilecoind.DEFAULT_SUBADDRESS_INDEX, tx_list, outlays)

    sender_tx_receipt = mobilecoind.submit_tx(tx_proposal).sender_tx_receipt

    # Wait for the transaction to clear
    tx_status = mobilecoind.TX_STATUS_UNKNOWN
    while tx_status == mobilecoind.TX_STATUS_UNKNOWN:
        time.sleep(TX_RECEIPT_CHECK_INTERVAL_SECONDS)
        tx_status = int(mobilecoind.get_tx_status_as_sender(sender_tx_receipt))

    if tx_status != mobilecoind.TX_STATUS_VERIFIED:
        print("ERROR... Transaction failed with status {}".format(tx_status))

        # remove recipient monitor
        mobilecoind.remove_monitor(recipient_monitor_id)
        print("# removed monitor {} for {}".format(recipient_monitor_id.hex(), new_user_email))

        return 0 # no email sent

    # Check that balances are as expected
    wait_for_monitor(sender_monitor_id)
    sender_balance = mobilecoind.get_balance(sender_monitor_id)

    wait_for_monitor(recipient_monitor_id)
    recipient_balance = mobilecoind.get_balance(recipient_monitor_id)

    print("# recipient balance = {} picoMOB, sender balance = {} picoMOB".format(recipient_balance, sender_balance))

    # If the recipient's balance is not as expected, complain and do not trigger the email in Mailchimp
    if recipient_balance != amount_picoMOB:
        print(
            "ERROR... recipient balance is not correct! Entropy {} has only {} MOB. Expected {} MOB."
            .format(recipient_entropy.hex(), recipient_balance/MOB, amount_picoMOB/MOB)
        )
        email_sent = 0

    else:
        # set the entropy value at MailChimp
        data = {"merge_fields":{"ENTROPY":recipient_entropy.hex()}}
        response = mailchimp.lists.members.update(list_id, subscriber_hash=new_user_hash, data=data)

        # adding "send_key_now" tag triggers the welcome email automation!
        data = {"tags":[{"name":"has_entropy", "status":"active"},{"name":"send_key_now", "status":"active"}]}
        mailchimp.lists.members.tags.update(list_id, subscriber_hash=new_user_hash, data=data)

        print("# setting welcome email trigger for {}!".format(new_user_email))
        email_sent = 1

    # remove recipient monitor
    mobilecoind.remove_monitor(recipient_monitor_id)
    print("# removed monitor {} for {}".format(recipient_monitor_id.hex(), new_user_email))

    return email_sent

if __name__ == '__main__':
    # Parse the arguments and generate the mobilecoind client
    mobilecoind = mobilecoin.Client("localhost:4444", ssl=False)

    parser = argparse.ArgumentParser(description='provide secrets')
    parser.add_argument('-k', '--key', help='MailChimp API key', type=str, required=True)
    parser.add_argument('-s', '--sender', help='MobileCoin sender master key as hex', type=str, required=True)
    parser.add_argument('-v', '--value', help='amount to allocate in MOB (default=100)', nargs='?', const=100, type=int)
    parser.add_argument('--clean', help='remove all old monitors', action='store_true')
    args = parser.parse_args()

    print("\n# *\n# * Starting up TestNet token allocation script!\n# *\n#")

    # Set up our "bank"
    sender_entropy_bytes = bytes.fromhex(args.sender)
    sender_account_key = mobilecoind.get_account_key(sender_entropy_bytes)
    sender_monitor_id = mobilecoind.add_monitor(sender_account_key)
    sender_public_address = mobilecoind.get_public_address(sender_monitor_id)

    # clean up all old monitors -- except for sender_monitor_id
    if args.clean:
        for monitor_id in mobilecoind.get_monitor_list():
            if monitor_id != sender_monitor_id:
                print("# removing existing monitor_id {}.".format(monitor_id.hex()))
                mobilecoind.remove_monitor(monitor_id)

    # Wait for mobilecoind to get sender's current balance
    wait_for_monitor(sender_monitor_id)

    # generate the MailChimp client
    mailchimp = MailChimp(mc_api=args.key)

    # figure out the id for the list of interest
    # print(mailchimp.lists.all(get_all=True, fields="lists.name,lists.id"))
    list_id = '5f47419453' # The "MobileCoin" Audience

    # go through all the subscribers in chunks and find any who don't have an assigned entropy
    print("# * Processing all existing records")
    fields="members.id,members.email_address,members.merge_fields,members.status" # important: no spaces!

    offset = 0
    count = 200 # can be up to 1000
    emails_sent = 0
    while count > 0:
        members = mailchimp.lists.members.all(list_id, count=count, offset=offset, fields=fields)["members"]
        count = len(members)
        offset += count
        if count > 0 :
            print("# processed {} records found at MailChimp".format(offset))
        for member_record in members:
            if member_record["status"] == "subscribed" and not member_record["merge_fields"]["ENTROPY"]:
                emails_sent += allocate_MOB(member_record, args.value * MOB)
    if emails_sent > 0:
        print("# sent {} MOB to each of {} new records found at MailChimp".format(args.value, emails_sent))
    else:
        print("# no new records found.")

    print("# * Finished processing all existing records")
