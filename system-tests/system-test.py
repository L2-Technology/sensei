#!/usr/bin/env python3
import atexit
import logging
import os
import signal
import subprocess

import sys
import time
from shutil import rmtree
from subprocess import Popen

import jsonrpc_requests
import requests
import grpc
from retrying import retry

from sensei_pb2_grpc import AdminStub, NodeStub
from sensei_pb2 import (
    CloseChannelRequest, GetStatusRequest, CreateNodeRequest, GetUnusedAddressRequest,
    GetBalanceRequest,
    ListPaymentsRequest, OpenChannelsRequest, OpenChannelRequest, ListChannelsRequest, CreateInvoiceRequest,
    PaginationRequest, PayInvoiceRequest
)

import functools

print = functools.partial(print, flush=True)

processes: [Popen] = []
OUTPUT_DIR = 'test-output'
NUM_PAYMENTS = 100
CHANNEL_BALANCE_SYNC_INTERVAL = 25
WAIT_TIMEOUT = 20
CHANNEL_VALUE_SAT = 10_000_000
INITIAL_FUNDING_SAT = 100_000_000
PAYMENT_MSAT = 4_000_000
DEBUG_ON_FAIL = True  # go into Python debugger / don't kill processes on failure
USE_RELEASE_BINARIES = False

SIGNER = "default"

logger = logging.getLogger()

os.environ['RUST_BACKTRACE'] = "1"


def kill_procs():
    global processes
    for p in processes:
        p.send_signal(signal.SIGTERM)


class Bitcoind(jsonrpc_requests.Server):
    def __init__(self, name, url, **kwargs):
        self.name = name
        self.mine_address = None
        super().__init__(url, **kwargs)

    def wait_for_ready(self):
        timeout = 5
        request_exception = None
        while timeout > 0:
            try:
                self.getblockchaininfo()
                break
            except Exception as e:
                request_exception = e
                time.sleep(1)
                timeout -= 1
        if timeout <= 0:
            if request_exception:
                raise request_exception
            raise Exception('Timeout')

    def setup(self):
        self.createwallet('default')
        # unload and reload with autoload, in case dev wants to play with it later
        self.unloadwallet('default')
        self.loadwallet('default', True)
        self.mine_address = self.getnewaddress()

    def mine(self, count=1):
        self.generatetoaddress(count, self.mine_address)
        print('at height', self.getblockchaininfo()['blocks'])

@retry(stop_max_attempt_number=5, wait_fixed=100)
def grpc_admin_client(url, metadata):
    channel = grpc.insecure_channel(url)
    stub = AdminStub(channel)
    # stub.GetStatus(GetStatusRequest(), metadata=metadata, timeout=1)
    return stub


@retry(stop_max_attempt_number=5, wait_fixed=100)
def grpc_node_client(url, metadata):
    channel = grpc.insecure_channel(url)
    stub = NodeStub(channel)
    stub.GetBalance(GetBalanceRequest(), metadata=metadata, timeout=1)
    return stub


# retry every 0.1 seconds until WAIT_TIMEOUT seconds have passed
def wait_until(name, func):
    logger.info(f'wait for {name}')
    timeout = WAIT_TIMEOUT * 10
    exc = None
    while timeout > 0:
        try:
            if func():
                break
        except Exception as e:
            exc = e
        time.sleep(0.5)
        timeout -= 1
    if timeout <= 0:
        print('Failed', name)
        if DEBUG_ON_FAIL:
            print(f'failed with exc={exc}')
            import pdb
            pdb.set_trace()
        if exc:
            raise exc
        raise Exception('Timeout')
    logger.debug(f'done {name}')


def run():
    # ensure we sync after the last payment
    assert NUM_PAYMENTS % CHANNEL_BALANCE_SYNC_INTERVAL == 0

    atexit.register(kill_procs)
    rmtree(OUTPUT_DIR, ignore_errors=True)
    os.mkdir(OUTPUT_DIR)
    print('Starting bitcoind')
    btc = start_bitcoind()

    print('Starting nodes')
    senseid, metadata = start_senseid()

    print('Generate initial blocks')
    btc.mine(110)
    balance = btc.getbalance()
    assert balance > 0

    alice, meta_a, id_a = fund_node(btc, metadata, senseid, 1)
    bob, meta_b, id_b = fund_node(btc, metadata, senseid, 2)

    print('Create channel alice -> bob')
    oc_res = alice.OpenChannels(OpenChannelsRequest(requests=[OpenChannelRequest(counterparty_pubkey=f"{id_b}", counterparty_host_port=f"127.0.0.1:10001", amount_sats=CHANNEL_VALUE_SAT, public=True)]),
                      metadata=meta_a)

    print(oc_res)
    wait_until('channel at bob', lambda: bob.ListChannels(ListChannelsRequest(), metadata=meta_b).channels[0])
    assert not bob.ListChannels(ListChannelsRequest(), metadata=meta_b).channels[0].is_usable

    charlie, meta_c, id_c = fund_node(btc, metadata, senseid, 3)

    print('Create channel bob -> charlie')
    bob.OpenChannels(OpenChannelsRequest(requests=[OpenChannelRequest(counterparty_pubkey=f"{id_c}", counterparty_host_port=f"127.0.0.1:10002", amount_sats=CHANNEL_VALUE_SAT, public=True)]),
                    metadata=meta_b)
    wait_until('channel at charlie', lambda: charlie.ListChannels(ListChannelsRequest(), metadata=meta_c).channels[0])
    assert not charlie.ListChannels(ListChannelsRequest(), metadata=meta_c).channels[0].is_usable

    print('Confirm channels')
    btc.mine(6)

    def channel_active():
        btc.mine(1)
        alice_chans = alice.ListChannels(ListChannelsRequest(), metadata=meta_a).channels
        bob_chans = bob.ListChannels(ListChannelsRequest(), metadata=meta_b).channels
        charlie_chans = charlie.ListChannels(ListChannelsRequest(), metadata=meta_c).channels
        return (alice_chans[0].is_usable and
                bob_chans[0].is_usable and
                bob_chans[1].is_usable and
                charlie_chans[0].is_usable)

    wait_until('channels active', channel_active)

    for i in range(1, NUM_PAYMENTS + 1):
        print(f'Pay invoice {i}')
        invoice = charlie.CreateInvoice(CreateInvoiceRequest(amt_msat=PAYMENT_MSAT, description="stuff"), metadata=meta_c).invoice
        alice.PayInvoice(PayInvoiceRequest(invoice=invoice), metadata=meta_a)

        if i % CHANNEL_BALANCE_SYNC_INTERVAL == 0:
            print('*** SYNC TO CHANNEL BALANCE')
            # check within 0.5%, due to fees

            wait_until('channel balance alice',
                       lambda: assert_equal_delta(CHANNEL_VALUE_SAT * 1000 -
                                                  alice.ListChannels(ListChannelsRequest(), metadata=meta_a).channels[0].balance_msat,
                                                  i * PAYMENT_MSAT))
            wait_until('channel balance charlie',
                       lambda: assert_equal_delta(charlie.ListChannels(ListChannelsRequest(), metadata=meta_c).channels[0].balance_msat,
                                                  max(0, i * PAYMENT_MSAT)))

    def check_payments():
        pagination = PaginationRequest(page=0, take=1000)
        payment_list = alice.ListPayments(ListPaymentsRequest(pagination=pagination), metadata=meta_a)
        assert len(payment_list.payments) == NUM_PAYMENTS
        for payment in payment_list.payments:
            assert payment.origin == "invoice_outgoing"
            assert payment.status == "succeeded", payment
        return True

    wait_until('check payments', check_payments)

    print('Closing alice - bob')
    alice_channel = alice.ListChannels(ListChannelsRequest(), metadata=meta_a).channels[0]
    alice.CloseChannel(CloseChannelRequest(channel_id=alice_channel.channel_id), metadata=meta_a)

    def is_channel_closed(node, meta, is_outbound):
        btc.mine(1)
        chans = node.ListChannels(ListChannelsRequest(), metadata=meta).channels
        chans = list(filter(lambda c: c.is_outbound == is_outbound, chans))
        return len(chans) == 0

    wait_until('alice close', lambda: is_channel_closed(alice, meta_a, True))
    wait_until('bob close', lambda: is_channel_closed(bob, meta_b, False))

    def check_balance(node, meta, expected):
        btc.mine(1)
        balance = node.GetBalance(GetBalanceRequest(), metadata=meta).onchain_balance_sats
        # print(f'balance {balance} expected {expected}')
        assert_equal_delta(balance, expected)
        return True

    wait_until('alice balance',
               lambda: check_balance(alice, meta_a, INITIAL_FUNDING_SAT - (NUM_PAYMENTS * PAYMENT_MSAT) / 1000 - 1000))

    print('Force closing charlie')
    charlie_channel = charlie.ListChannels(ListChannelsRequest(), metadata=meta_c).channels[0]
    charlie.CloseChannel(CloseChannelRequest(channel_id=charlie_channel.channel_id, force=True), metadata=meta_c)

    wait_until('charlie close', lambda: is_channel_closed(charlie, meta_c, False))
    wait_until('bob close', lambda: is_channel_closed(bob, meta_b, True))

    wait_until('bob balance',
               lambda: check_balance(bob, meta_b, INITIAL_FUNDING_SAT))
    btc.mine(144)
    wait_until('charlie balance',
               lambda: check_balance(charlie, meta_c, INITIAL_FUNDING_SAT + (NUM_PAYMENTS * PAYMENT_MSAT) / 1000 - 1000))
    print('Done')


def fund_node(btc, metadata, grpc, n):
    print(f"Create node {n}")
    node_res = grpc.CreateNode(CreateNodeRequest(username=f"admin{n}", alias=f"Satoshi{n}", passphrase="pass", start=True),
                               metadata=metadata)
    node_metadata = (('macaroon', node_res.macaroon),)
    node = grpc_node_client(f'localhost:3301', metadata=node_metadata)
    address = node.GetUnusedAddress(GetUnusedAddressRequest(), metadata=node_metadata).address
    btc.sendtoaddress(address, 1)
    btc.mine(1)
    wait_until(f'balance {n}', lambda: node.GetBalance(GetBalanceRequest(), metadata=node_metadata).onchain_balance_sats > 0)
    return node, node_metadata, node_res.pubkey


def fund_root_node(btc, metadata):
    print(f"Fund root node")
    node = grpc_node_client(f'localhost:3301', metadata=metadata)
    address = node.GetUnusedAddress(GetUnusedAddressRequest(), metadata=metadata).address
    btc.sendtoaddress(address, 1)
    btc.mine(1)
    wait_until(f'balance root', lambda: node.GetBalance(GetBalanceRequest(), metadata=metadata).onchain_balance_sats > 0)
    return node


def assert_equal_delta(a, b):
    if a < b * 0.998 or a > b * 1.001:
        raise AssertionError(f'value out of range {a} vs {b}')
    return True


def start_bitcoind():
    global processes

    btc_log = open(OUTPUT_DIR + '/btc.log', 'w')
    btc_proc = Popen([
        # 'strace', '-o', '/tmp/out', '-s', '10000', '-f',
        'bitcoind', '--regtest', '--fallbackfee=0.0000001',
        '--rpcuser=user', '--rpcpassword=pass',
        '--rpcport=18883',
        f'--datadir={OUTPUT_DIR}'], stdout=btc_log)
    processes.append(btc_proc)
    btc = Bitcoind('btc-regtest', 'http://user:pass@localhost:18883')
    btc.wait_for_ready()
    btc.setup()
    return btc


def start_senseid():
    global processes

    stdout_log = open(OUTPUT_DIR + f'/node.log', 'w')
    optimization = 'release' if USE_RELEASE_BINARIES else 'debug'
    cmd = [f'target/{optimization}/senseid',
           '--network=regtest',
           '--bitcoind-rpc-host=localhost',
           '--bitcoind-rpc-port=18883',
           '--bitcoind-rpc-username=user',
           '--bitcoind-rpc-password=pass',
           f'--data-dir={OUTPUT_DIR}',
           f'--database-url=sensei.db',  # TODO should be inside OUTPUT_DIR, but that gives a PATH
           '--api-port=3301'
           ]
    p = Popen(cmd, stdout=stdout_log, stderr=subprocess.STDOUT)
    processes.append(p)
    time.sleep(2)
    requests.get('http://localhost:3301/api/v1/status')

    print("Init sensei")
    res = requests.post('http://localhost:3301/api/v1/init',
                        json={ "passphrase": "test", "username": "admin"})
    json = res.json()
    metadata = (('macaroon', 'invalid'), ('token', json['token']))
    senseid = grpc_admin_client(f'localhost:3301', metadata)
    return senseid, metadata


if __name__ == '__main__':
    logging.basicConfig(level=logging.INFO)
    run()
